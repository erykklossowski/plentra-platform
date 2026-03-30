use std::sync::Arc;

use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use chrono::Utc;
use serde_json::Value;

use crate::models::fuel::FuelData;
use crate::AppState;

const CACHE_KEY: &str = "fuels";

pub async fn handler(State(state): State<Arc<AppState>>) -> (HeaderMap, Json<Value>) {
    let mut headers = HeaderMap::new();
    headers.insert("cache-control", "max-age=900".parse().unwrap());

    // 1. DashMap cache
    if let Some(cached) = state.cache.get(CACHE_KEY) {
        return (headers, Json(cached.data));
    }

    // 2. Read latest prices from TimescaleDB
    if let Some(pool) = &state.db {
        let (ttf, eua, ara) = tokio::join!(
            crate::db::reader::get_latest_fuel_price(pool, "TTF"),
            crate::db::reader::get_latest_fuel_price(pool, "EUA"),
            crate::db::reader::get_latest_fuel_price(pool, "ARA"),
        );

        let ttf_db = ttf.ok().flatten();
        let eua_db = eua.ok().flatten();
        let ara_db = ara.ok().flatten();

        if let (Some(ttf_v), Some(eua_v), Some(ara_v)) = (ttf_db, eua_db, ara_db) {
            // Get sparklines from DB
            let (ttf_hist, ara_hist, eua_hist) = tokio::join!(
                crate::db::reader::get_fuel_sparkline(pool, "TTF", 30),
                crate::db::reader::get_fuel_sparkline(pool, "ARA", 30),
                crate::db::reader::get_fuel_sparkline(pool, "EUA", 30),
            );

            let ttf_hist = ttf_hist.ok().filter(|v| v.len() >= 2).unwrap_or_default();
            let ara_hist = ara_hist.ok().filter(|v| v.len() >= 2).unwrap_or_default();
            let eua_hist = eua_hist.ok().filter(|v| v.len() >= 2).unwrap_or_default();

            let fuel_data = FuelData {
                ttf_eur_mwh: ttf_v,
                ttf_change_pct: crate::fetchers::databento::mom_delta_pct(&ttf_hist),
                ttf_history_30d: ttf_hist,
                ara_usd_tonne: ara_v,
                ara_change_pct: crate::fetchers::databento::mom_delta_pct(&ara_hist),
                ara_history_30d: ara_hist,
                eua_eur_tonne: eua_v,
                eua_change_pct: crate::fetchers::databento::mom_delta_pct(&eua_hist),
                eua_history_30d: eua_hist,
                fetched_at: Utc::now().to_rfc3339(),
                stale: None,
            };

            let value = serde_json::to_value(&fuel_data).unwrap();
            state
                .cache
                .set(CACHE_KEY.to_string(), value.clone(), state.config.cache_ttl_fuels);

            // Background: persist cached response for other routes' DB fallback
            let pool_c = pool.clone();
            let cached_value = value.clone();
            tokio::spawn(async move {
                let _ = crate::db::writer::write_cached_response(&pool_c, CACHE_KEY, &cached_value).await;
            });

            // Spawn background Databento refresh to keep data current
            if let Some(ref key) = state.config.databento_api_key {
                let key_c = key.clone();
                let state_c = state.clone();
                tokio::spawn(async move {
                    background_databento_refresh(&key_c, &state_c).await;
                });
            }

            return (headers, Json(value));
        }
    }

    // 3. DB empty — live Databento fetch
    if let Some(ref key) = state.config.databento_api_key {
        let settlements = crate::fetchers::databento::fetch_today(key).await;
        if !settlements.is_empty() {
            let find = |name: &str| {
                settlements.iter().find(|(n, _, _)| *n == name).map(|(_, p, _)| *p)
            };
            let ttf_v = find("TTF").unwrap_or(0.0);
            let eua_v = find("EUA").unwrap_or(0.0);
            let ara_v = find("ARA").unwrap_or(0.0);

            let fuel_data = FuelData {
                ttf_eur_mwh: ttf_v,
                ttf_change_pct: 0.0,
                ttf_history_30d: vec![],
                ara_usd_tonne: ara_v,
                ara_change_pct: 0.0,
                ara_history_30d: vec![],
                eua_eur_tonne: eua_v,
                eua_change_pct: 0.0,
                eua_history_30d: vec![],
                fetched_at: Utc::now().to_rfc3339(),
                stale: None,
            };

            let value = serde_json::to_value(&fuel_data).unwrap();
            state
                .cache
                .set(CACHE_KEY.to_string(), value.clone(), state.config.cache_ttl_fuels);

            return (headers, Json(value));
        }
    }

    // 4. Try stale cache or DB cached response
    if let Some(stale) = state.cache.get_stale(CACHE_KEY) {
        let mut data = stale.data;
        if let Some(obj) = data.as_object_mut() {
            obj.insert("stale".to_string(), Value::Bool(true));
        }
        return (headers, Json(data));
    }

    if let Some(data) = db_fallback(&state, CACHE_KEY).await {
        return (headers, Json(data));
    }

    // 5. All sources failed — graceful empty (NEVER 500)
    tracing::warn!("All fuel data sources unavailable");
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

async fn background_databento_refresh(api_key: &str, state: &Arc<AppState>) {
    let settlements = crate::fetchers::databento::fetch_today(api_key).await;
    if settlements.is_empty() {
        return;
    }
    state.cache.invalidate(CACHE_KEY);
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
