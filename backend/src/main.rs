mod cache;
mod config;
mod db;
mod fetchers;
mod models;
mod routes;
mod services;

use std::sync::Arc;

use axum::{http::HeaderValue, routing::get, Router};
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
        .allow_methods([axum::http::Method::GET, axum::http::Method::OPTIONS])
        .allow_headers(tower_http::cors::Any);

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
        .route("/admin/backfill", get(routes::admin::handler))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("0.0.0.0:{port}");
    tracing::info!("Plentra backend listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
