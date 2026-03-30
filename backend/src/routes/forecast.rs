use std::sync::Arc;

use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use chrono::Utc;
use serde_json::{json, Value};

use crate::AppState;

const CACHE_KEY: &str = "forecast";

pub async fn handler(State(state): State<Arc<AppState>>) -> (HeaderMap, Json<Value>) {
    let mut headers = HeaderMap::new();
    headers.insert("cache-control", "max-age=3600".parse().unwrap());

    // Check cache
    if let Some(cached) = state.cache.get(CACHE_KEY) {
        return (headers, Json(cached.data));
    }

    // DB fallback
    if let Some(pool) = &state.db {
        if let Ok(Some(mut data)) = crate::db::reader::get_cached_response(pool, CACHE_KEY).await {
            if let Some(obj) = data.as_object_mut() {
                obj.insert("stale".to_string(), Value::Bool(true));
            }
            return (headers, Json(data));
        }
    }

    let pool = match &state.db {
        Some(p) => p,
        None => {
            return (
                headers,
                Json(json!({
                    "data_status": "unavailable",
                    "message": "Database required for forecasting",
                })),
            );
        }
    };

    // Fetch last 90 days of daily fuel prices from DB
    let ttf_history = crate::db::reader::get_fuel_sparkline(pool, "TTF", 90)
        .await
        .unwrap_or_default();
    let ara_history = crate::db::reader::get_fuel_sparkline(pool, "ARA", 90)
        .await
        .unwrap_or_default();
    let eua_history = crate::db::reader::get_fuel_sparkline(pool, "EUA", 90)
        .await
        .unwrap_or_default();

    // ETS forecast: 14-day horizon
    let ttf_forecast =
        crate::analytics::forecast::forecast_fuel_ets("TTF", &ttf_history, 14).ok();
    let ara_forecast =
        crate::analytics::forecast::forecast_fuel_ets("ARA", &ara_history, 14).ok();
    let eua_forecast =
        crate::analytics::forecast::forecast_fuel_ets("EUA", &eua_history, 14).ok();

    // MSTL decomposition of TTF
    let ttf_decomp = if ttf_history.len() >= 14 {
        crate::analytics::decomposition::decompose_daily(&ttf_history).ok()
    } else {
        None
    };

    // Changepoint detection on EUA
    let eua_changepoints = if eua_history.len() >= 30 {
        let dates: Vec<chrono::NaiveDate> = (0..eua_history.len())
            .map(|i| {
                Utc::now().date_naive()
                    - chrono::Duration::days((eua_history.len() - 1 - i) as i64)
            })
            .collect();
        crate::analytics::changepoint::detect_changepoints(&eua_history, &dates).ok()
    } else {
        None
    };

    let forecast_to_json = |f: &crate::analytics::forecast::FuelForecast| -> Value {
        json!({
            "ticker": f.ticker,
            "horizon_days": f.horizon_days,
            "last_historical": f.last_historical,
            "training_points": f.training_points,
            "point_forecast": f.point_forecast,
            "lower_80": f.lower_80,
            "upper_80": f.upper_80,
            "lower_95": f.lower_95,
            "upper_95": f.upper_95,
        })
    };

    let response = json!({
        "generated_at": Utc::now().to_rfc3339(),
        "data_status": "ok",
        "fuel_forecasts": {
            "ttf": ttf_forecast.as_ref().map(&forecast_to_json),
            "ara": ara_forecast.as_ref().map(&forecast_to_json),
            "eua": eua_forecast.as_ref().map(&forecast_to_json),
        },
        "decomposition": ttf_decomp.as_ref().map(|d| json!({
            "ticker": "TTF",
            "series_len": d.series_len,
            "trend": d.trend,
            "seasonal_7d": d.seasonal_7d,
            "residual": d.residual,
        })),
        "changepoint_alerts": eua_changepoints.and_then(|c| {
            c.alert_message.map(|msg| json!({
                "ticker": "EUA",
                "alert": true,
                "message": msg,
                "latest_break_index": c.latest_break,
            }))
        })
    });

    // Cache for 1 hour
    state
        .cache
        .set(CACHE_KEY.to_string(), response.clone(), 3600);

    // Persist to DB
    if let Some(pool) = state.db.clone() {
        let data = response.clone();
        tokio::spawn(async move {
            if let Err(e) = crate::db::writer::write_cached_response(&pool, CACHE_KEY, &data).await
            {
                tracing::warn!("DB cache write failed for forecast: {}", e);
            }
        });
    }

    (headers, Json(response))
}
