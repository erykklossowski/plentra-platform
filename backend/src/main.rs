mod analytics;
mod cache;
mod config;
mod db;
mod fetchers;
mod models;
mod routes;
mod services;

use std::sync::Arc;

use axum::{http::HeaderValue, routing::{get, post}, Router};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

pub struct AppState {
    pub config: config::Config,
    pub cache: cache::Cache,
    pub http_client: reqwest::Client,
    pub db: Option<sqlx::PgPool>,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = config::Config::from_env();
    let port = config.port;

    // Database: optional — app works without it
    let db = if let Some(ref url) = config.database_url {
        match db::pool::connect(url).await {
            Ok(pool) => {
                tracing::info!("TimescaleDB connected");
                Some(pool)
            }
            Err(e) => {
                tracing::warn!("DB connection failed, running without persistence: {}", e);
                None
            }
        }
    } else {
        tracing::info!("DATABASE_URL not set, running without persistence");
        None
    };

    let state = Arc::new(AppState {
        config,
        cache: cache::Cache::new(),
        http_client: reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("failed to build HTTP client"),
        db,
    });

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list([
            HeaderValue::from_static("http://localhost:3000"),
            HeaderValue::from_static("https://frontend-gamma-pink-76.vercel.app"),
            HeaderValue::from_static("https://plentra.vercel.app"),
        ]))
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST, axum::http::Method::OPTIONS])
        .allow_headers(tower_http::cors::Any);

    // Databento daily fuel scheduler at 19:30 UTC
    {
        let sched_state = state.clone();
        tokio::spawn(async move {
            // Verify API key on startup
            if let Some(ref key) = sched_state.config.databento_api_key {
                match fetchers::databento::verify_api_key(key).await {
                    Ok(()) => {}
                    Err(e) => tracing::warn!("Databento API key check failed: {}", e),
                }
            }

            loop {
                let now = chrono::Utc::now();
                let target = now
                    .date_naive()
                    .and_hms_opt(19, 30, 0)
                    .unwrap()
                    .and_utc();
                let next_run = if now < target {
                    target
                } else {
                    (now.date_naive() + chrono::Duration::days(1))
                        .and_hms_opt(19, 30, 0)
                        .unwrap()
                        .and_utc()
                };

                let secs = (next_run - now).num_seconds().max(0) as u64;
                tracing::info!(
                    "Fuel scheduler: sleeping {}s until {}",
                    secs,
                    next_run.format("%Y-%m-%d %H:%M UTC")
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(secs)).await;

                let Some(ref api_key) = sched_state.config.databento_api_key else {
                    tracing::warn!("Fuel scheduler: DATABENTO_API_KEY not set, skipping");
                    continue;
                };

                tracing::info!("Fuel scheduler: running Databento fetch");
                let settlements = fetchers::databento::fetch_today(api_key).await;

                if settlements.is_empty() {
                    tracing::warn!("Fuel scheduler: no settlements — weekend or holiday");
                    continue;
                }

                // Invalidate caches
                sched_state.cache.invalidate("fuels");
                sched_state.cache.invalidate("summary");
                sched_state.cache.invalidate("forecast");
                sched_state.cache.invalidate("spreads");

                // Persist to TimescaleDB
                if let Some(ref pool) = sched_state.db {
                    let ts = chrono::Utc::now()
                        .date_naive()
                        .and_hms_opt(17, 30, 0)
                        .unwrap()
                        .and_utc();

                    for (name, price, unit) in &settlements {
                        match crate::db::writer::write_fuel_price(
                            pool, ts, name, *price, unit, "DATABENTO",
                        )
                        .await
                        {
                            Ok(()) => tracing::info!(
                                "Fuel scheduler: wrote {} {:.4} {}",
                                name,
                                price,
                                unit
                            ),
                            Err(e) => tracing::warn!(
                                "Fuel scheduler: DB write failed for {}: {}",
                                name,
                                e
                            ),
                        }
                    }
                }
            }
        });
    }

    let app = Router::new()
        .route("/health", get(routes::health::handler))
        .route("/api/fuels", get(routes::fuels::handler))
        .route("/api/spreads", get(routes::spreads::handler))
        .route("/api/summary", get(routes::summary::handler))
        .route("/api/residual", get(routes::residual::handler))
        .route("/api/prices", get(routes::prices::handler))
        .route("/api/crossborder", get(routes::crossborder::handler))
        .route("/api/generation", get(routes::generation::handler))
        .route("/api/europe", get(routes::europe::handler))
        .route("/api/reserves", get(routes::reserves::handler))
        .route("/api/curtailment", get(routes::curtailment::handler))
        .route("/api/forecast", get(routes::forecast::handler))
        .route("/api/history/fuels", get(routes::history::fuels_handler))
        .route("/api/history/spreads", get(routes::history::spreads_handler))
        .route("/api/history/curtailment", get(routes::history::curtailment_handler))
        .route("/api/history/reserves", get(routes::history::reserves_handler))
        .route("/api/history/prices", get(routes::history::prices_handler))
        .route("/admin/backfill", get(routes::admin::handler))
        .route("/admin/refresh", post(routes::admin::refresh_handler))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("0.0.0.0:{port}");
    tracing::info!("Plentra backend listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
