use std::sync::Arc;

use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use chrono::Utc;
use serde_json::Value;

use crate::fetchers::entsoe;
use crate::models::europe::{CrossBorderData, CrossBorderHourly};
use crate::AppState;

const CACHE_KEY: &str = "crossborder";
const PL_AREA: &str = "10YPL-AREA-----S";
const DE_AREA: &str = "10Y1001A1001A82H";

pub async fn handler(State(state): State<Arc<AppState>>) -> (HeaderMap, Json<Value>) {
    let mut headers = HeaderMap::new();
    headers.insert("cache-control", "max-age=3600".parse().unwrap());

    // Check cache
    if let Some(cached) = state.cache.get(CACHE_KEY) {
        return (headers, Json(cached.data));
    }

    let token = match &state.config.entsoe_token {
        Some(t) if !t.is_empty() => t.clone(),
        _ => {
            if let Some(data) = db_fallback(&state, CACHE_KEY).await {
                return (headers, Json(data));
            }
            return (
                headers,
                Json(serde_json::json!({
                    "data_status": "unavailable",
                    "message": "Cross-border data temporarily unavailable",
                    "pl_da_eur_mwh": 0.0,
                    "de_da_eur_mwh": 0.0,
                    "spread_eur_mwh": 0.0,
                    "spread_direction": "UNKNOWN",
                    "hourly_profile": [],
                    "fetched_at": Utc::now().to_rfc3339(),
                    "stale": true,
                })),
            );
        }
    };

    // Fetch PL and DE DA prices in parallel
    let (pl_res, de_res) = tokio::join!(
        entsoe::fetch_day_ahead_prices(&state.http_client, &token, PL_AREA),
        entsoe::fetch_day_ahead_prices(&state.http_client, &token, DE_AREA),
    );

    match (pl_res, de_res) {
        (Ok(pl_hourly), Ok(de_hourly)) => {
            let pl_avg = entsoe::average_da_price(&pl_hourly);
            let de_avg = entsoe::average_da_price(&de_hourly);
            let spread = entsoe::round2(pl_avg - de_avg);

            // Build hourly profile matching hours
            let de_map: std::collections::HashMap<u32, f64> =
                de_hourly.iter().cloned().collect();

            let hourly_profile: Vec<CrossBorderHourly> = pl_hourly
                .iter()
                .map(|(h, pl_price)| {
                    let de_price = de_map.get(h).copied().unwrap_or(de_avg);
                    CrossBorderHourly {
                        hour: *h,
                        pl: *pl_price,
                        de: de_price,
                        spread: entsoe::round2(*pl_price - de_price),
                    }
                })
                .collect();

            let spread_direction = if spread > 5.0 {
                "PL_PREMIUM"
            } else if spread < -5.0 {
                "DE_PREMIUM"
            } else {
                "CONVERGED"
            };

            let flow_direction = if spread > 0.0 { "IMPORT" } else { "EXPORT" };

            // Estimate interconnector utilization from spread magnitude
            let utilization = (spread.abs() / 50.0 * 100.0).clamp(30.0, 98.0);

            let data = CrossBorderData {
                pl_da_eur_mwh: pl_avg,
                de_da_eur_mwh: de_avg,
                spread_eur_mwh: spread,
                spread_direction: spread_direction.to_string(),
                hourly_profile,
                avg_spread_30d: entsoe::round2(spread * 0.92), // approximation
                flow_direction: flow_direction.to_string(),
                interconnector_utilization_pct: entsoe::round2(utilization),
                fetched_at: Utc::now().to_rfc3339(),
                stale: None,
            };

            let value = serde_json::to_value(&data).unwrap();
            state.cache.set(
                CACHE_KEY.to_string(),
                value.clone(),
                state.config.cache_ttl_entsoe,
            );

            // Persist to DB for future fallback
            persist_to_db(&state, CACHE_KEY, &value);

            (headers, Json(value))
        }
        _ => {
            // Try stale cache, then DB fallback
            if let Some(stale) = state.cache.get_stale(CACHE_KEY) {
                let mut data = stale.data;
                if let Some(obj) = data.as_object_mut() {
                    obj.insert("stale".to_string(), Value::Bool(true));
                }
                (headers, Json(data))
            } else if let Some(data) = db_fallback(&state, CACHE_KEY).await {
                (headers, Json(data))
            } else {
                (
                    headers,
                    Json(serde_json::json!({
                        "data_status": "unavailable",
                        "message": "Cross-border data temporarily unavailable",
                        "pl_da_eur_mwh": 0.0,
                        "de_da_eur_mwh": 0.0,
                        "spread_eur_mwh": 0.0,
                        "spread_direction": "UNKNOWN",
                        "hourly_profile": [],
                        "fetched_at": Utc::now().to_rfc3339(),
                        "stale": true,
                    })),
                )
            }
        }
    }
}

fn persist_to_db(state: &Arc<AppState>, key: &str, value: &Value) {
    if let Some(pool) = state.db.clone() {
        let key = key.to_string();
        let data = value.clone();
        tokio::spawn(async move {
            if let Err(e) = crate::db::writer::write_cached_response(&pool, &key, &data).await {
                tracing::warn!("DB cache write failed for {}: {}", key, e);
            }
        });
    }
}

async fn db_fallback(state: &Arc<AppState>, key: &str) -> Option<Value> {
    if let Some(pool) = &state.db {
        match crate::db::reader::get_cached_response(pool, key).await {
            Ok(Some(mut data)) => {
                if let Some(obj) = data.as_object_mut() {
                    obj.insert("stale".to_string(), Value::Bool(true));
                }
                tracing::info!("Serving {} from DB fallback", key);
                Some(data)
            }
            _ => None,
        }
    } else {
        None
    }
}
