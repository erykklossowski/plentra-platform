use std::sync::Arc;

use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use chrono::Utc;
use serde_json::Value;

use sqlx::PgPool;

use crate::fetchers::stooq;
use crate::models::fuel::FuelData;
use crate::AppState;

const CACHE_KEY: &str = "fuels";

pub async fn handler(State(state): State<Arc<AppState>>) -> (HeaderMap, Json<Value>) {
    let mut headers = HeaderMap::new();
    headers.insert("cache-control", "max-age=900".parse().unwrap());

    // Check cache
    if let Some(cached) = state.cache.get(CACHE_KEY) {
        return (headers, Json(cached.data));
    }

    // Fetch all three in parallel
    let (ttf_res, ara_res, eua_res) = tokio::join!(
        stooq::fetch_ttf(&state.http_client),
        stooq::fetch_ara(&state.http_client),
        stooq::fetch_eua(&state.http_client),
    );

    match (ttf_res, ara_res, eua_res) {
        (Ok(ttf), Ok(ara), Ok(eua)) => {
            // Try real history from DB, fall back to synthetic
            let (ttf_hist, ara_hist, eua_hist) = if let Some(pool) = &state.db {
                let (t, a, e) = tokio::join!(
                    crate::db::reader::get_fuel_sparkline(pool, "TTF", 30),
                    crate::db::reader::get_fuel_sparkline(pool, "ARA", 30),
                    crate::db::reader::get_fuel_sparkline(pool, "EUA", 30),
                );
                (
                    t.ok().filter(|v| v.len() >= 7).unwrap_or(ttf.history_30d.clone()),
                    a.ok().filter(|v| v.len() >= 7).unwrap_or(ara.history_30d.clone()),
                    e.ok().filter(|v| v.len() >= 7).unwrap_or(eua.history_30d.clone()),
                )
            } else {
                (ttf.history_30d.clone(), ara.history_30d.clone(), eua.history_30d.clone())
            };

            let fuel_data = FuelData {
                ttf_eur_mwh: ttf.current_price,
                ttf_change_pct: ttf.change_pct,
                ttf_history_30d: ttf_hist,
                ara_usd_tonne: ara.current_price,
                ara_change_pct: ara.change_pct,
                ara_history_30d: ara_hist,
                eua_eur_tonne: eua.current_price,
                eua_change_pct: eua.change_pct,
                eua_history_30d: eua_hist,
                fetched_at: Utc::now().to_rfc3339(),
                stale: None,
            };

            let value = serde_json::to_value(&fuel_data).unwrap();
            state
                .cache
                .set(CACHE_KEY.to_string(), value.clone(), state.config.cache_ttl_fuels);

            // Background: persist to TimescaleDB (non-blocking)
            if let Some(pool) = state.db.clone() {
                let fuel = fuel_data.clone();
                let cached_value = value.clone();
                tokio::spawn(async move {
                    if let Err(e) = persist_fuels(&pool, &fuel).await {
                        tracing::warn!("DB write failed for fuels: {}", e);
                    }
                    if let Err(e) = crate::db::writer::write_cached_response(&pool, CACHE_KEY, &cached_value).await {
                        tracing::warn!("DB cache write failed for fuels: {}", e);
                    }
                });
            }

            (headers, Json(value))
        }
        _ => {
            // At least one fetch failed — try stale cache
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
                        "message": "Fuel data temporarily unavailable",
                        "ttf_eur_mwh": 0.0,
                        "ttf_change_pct": 0.0,
                        "ttf_history_30d": [],
                        "ara_usd_tonne": 0.0,
                        "ara_change_pct": 0.0,
                        "ara_history_30d": [],
                        "eua_eur_tonne": 0.0,
                        "eua_change_pct": 0.0,
                        "eua_history_30d": [],
                        "fetched_at": Utc::now().to_rfc3339(),
                        "stale": true,
                    })),
                )
            }
        }
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

async fn persist_fuels(pool: &PgPool, data: &FuelData) -> anyhow::Result<()> {
    use crate::db::writer::write_fuel_price;

    let today = Utc::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();

    write_fuel_price(pool, today, "TTF", data.ttf_eur_mwh, "EUR/MWh", "STOOQ").await?;
    write_fuel_price(pool, today, "ARA", data.ara_usd_tonne, "USD/t", "STOOQ").await?;
    write_fuel_price(pool, today, "EUA", data.eua_eur_tonne, "EUR/t", "STOOQ").await?;

    tracing::debug!("Persisted fuel prices to DB");
    Ok(())
}
