mod cache;
mod config;
mod fetchers;
mod models;
mod routes;

use std::sync::Arc;

use axum::{routing::get, Router};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

pub struct AppState {
    pub config: config::Config,
    pub cache: cache::Cache,
    pub http_client: reqwest::Client,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = config::Config::from_env();
    let port = config.port;

    let origins: Vec<_> = config
        .allowed_origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();

    let state = Arc::new(AppState {
        config,
        cache: cache::Cache::new(),
        http_client: reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (compatible; PlentraBot/1.0)")
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("failed to build HTTP client"),
    });

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([axum::http::Method::GET, axum::http::Method::OPTIONS])
        .allow_headers([axum::http::header::CONTENT_TYPE]);

    let app = Router::new()
        .route("/health", get(routes::health::handler))
        .route("/api/fuels", get(routes::fuels::handler))
        .route("/api/spreads", get(routes::spreads::handler))
        .route("/api/summary", get(routes::summary::handler))
        .route("/api/residual", get(routes::residual::handler))
        .route("/api/prices", get(routes::prices::handler))
        .route("/api/crossborder", get(routes::crossborder::handler))
        .route("/api/reserves", get(routes::reserves::handler))
        .route("/api/curtailment", get(routes::curtailment::handler))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("0.0.0.0:{port}");
    tracing::info!("Plentra backend listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
