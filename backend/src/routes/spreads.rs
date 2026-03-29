use std::sync::Arc;

use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use chrono::Utc;
use serde_json::Value;

use crate::models::fuel::FuelData;
use crate::models::spread::{SpreadData, SpreadHistoryEntry};
use crate::AppState;

// Phase 1: hardcoded Polish day-ahead price (EUR/MWh)
// TODO Phase 2: fetch from ENTSO-E Transparency Platform
const RDN_EUR_MWH: f64 = 85.0;

// Phase 1: hardcoded EUR/USD exchange rate
// TODO Phase 3: add live FX
const EUR_USD: f64 = 1.08;

const CACHE_KEY: &str = "spreads";

fn calculate_css(rdn: f64, ttf_eur_mwh: f64, eua_eur_tonne: f64) -> f64 {
    rdn - (ttf_eur_mwh / 0.60) - (eua_eur_tonne * 0.202)
}

fn calculate_cds(rdn: f64, ara_usd_tonne: f64, eua_eur_tonne: f64, efficiency: f64) -> f64 {
    let ara_eur_tonne = ara_usd_tonne / EUR_USD;
    let ara_eur_gj = ara_eur_tonne / 29.31;
    rdn - (ara_eur_gj / efficiency) - (eua_eur_tonne * 0.341)
}

fn dispatch_signal(css: f64, cds: f64) -> &'static str {
    if css > 0.0 && css > cds {
        "GAS_MARGINAL"
    } else if cds > 0.0 && cds > css {
        "COAL_MARGINAL"
    } else {
        "NEGATIVE_SPREADS"
    }
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

pub async fn handler(State(state): State<Arc<AppState>>) -> (HeaderMap, Json<Value>) {
    let mut headers = HeaderMap::new();
    headers.insert("cache-control", "max-age=900".parse().unwrap());

    // Check cache
    if let Some(cached) = state.cache.get(CACHE_KEY) {
        return (headers, Json(cached.data));
    }

    // Get fuel data from cache, stale cache, or DB fallback
    let fuel_value = if let Some(cached) = state.cache.get("fuels") {
        cached.data
    } else if let Some(stale) = state.cache.get_stale("fuels") {
        stale.data
    } else if let Some(pool) = &state.db {
        match crate::db::reader::get_cached_response(pool, "fuels").await {
            Ok(Some(v)) => v,
            _ => {
                if let Some(data) = db_fallback(&state, CACHE_KEY).await {
                    return (headers, Json(data));
                }
                return (
                    headers,
                    Json(serde_json::json!({
                        "data_status": "unavailable",
                        "message": "Spread data temporarily unavailable",
                        "css_spot": null,
                        "cds_spot_eta42": null,
                        "cds_spot_eta34": null,
                        "dispatch_signal": "UNKNOWN",
                        "history_30d": [],
                        "fetched_at": Utc::now().to_rfc3339(),
                        "stale": true,
                    })),
                );
            }
        }
    } else {
        return (
            headers,
            Json(serde_json::json!({
                "data_status": "unavailable",
                "message": "Spread data temporarily unavailable",
                "css_spot": null,
                "cds_spot_eta42": null,
                "cds_spot_eta34": null,
                "dispatch_signal": "UNKNOWN",
                "history_30d": [],
                "fetched_at": Utc::now().to_rfc3339(),
                "stale": true,
            })),
        );
    };

    let fuel: FuelData = serde_json::from_value(fuel_value).unwrap();

    let css_spot = round2(calculate_css(RDN_EUR_MWH, fuel.ttf_eur_mwh, fuel.eua_eur_tonne));
    let cds_spot_eta42 =
        round2(calculate_cds(RDN_EUR_MWH, fuel.ara_usd_tonne, fuel.eua_eur_tonne, 0.42));
    let cds_spot_eta34 =
        round2(calculate_cds(RDN_EUR_MWH, fuel.ara_usd_tonne, fuel.eua_eur_tonne, 0.34));

    // Build 30-day history
    let len = fuel
        .ttf_history_30d
        .len()
        .min(fuel.ara_history_30d.len())
        .min(fuel.eua_history_30d.len());
    let history: Vec<SpreadHistoryEntry> = (0..len)
        .map(|i| {
            let ttf = fuel.ttf_history_30d[i];
            let ara = fuel.ara_history_30d[i];
            let eua = fuel.eua_history_30d[i];
            SpreadHistoryEntry {
                date: format!("day-{}", i + 1), // Phase 1: simplified date labels
                css: round2(calculate_css(RDN_EUR_MWH, ttf, eua)),
                cds_42: round2(calculate_cds(RDN_EUR_MWH, ara, eua, 0.42)),
            }
        })
        .collect();

    // Calculate MoM percentage changes from history arrays
    let css_history: Vec<f64> = history.iter().map(|h| h.css).collect();
    let cds_history: Vec<f64> = history.iter().map(|h| h.cds_42).collect();
    let css_pct_change = crate::fetchers::databento::mom_delta_pct(&css_history);
    let cds_pct_change = crate::fetchers::databento::mom_delta_pct(&cds_history);

    let carbon_impact = round2(-fuel.eua_eur_tonne * 0.202);

    let spread_data = SpreadData {
        css_spot,
        css_spot_pct_change: css_pct_change,
        cds_spot_eta34,
        cds_spot_eta42,
        cds_spot_pct_change: cds_pct_change,
        css_term_y1: round2(css_spot * 0.95), // approximate term value
        cds_term_y1: None,
        baseload_profitability_eur_mwh: round2(css_spot.max(0.0)),
        peak_load_advantage_eur_mwh: round2(css_spot * 1.4), // peak premium estimate
        carbon_impact_factor: carbon_impact,
        dispatch_signal: dispatch_signal(css_spot, cds_spot_eta42).to_string(),
        history_30d: history,
        fetched_at: Utc::now().to_rfc3339(),
        stale: None,
    };

    let value = serde_json::to_value(&spread_data).unwrap();
    state
        .cache
        .set(CACHE_KEY.to_string(), value.clone(), state.config.cache_ttl_fuels);

    // Persist to DB for future fallback
    if let Some(pool) = state.db.clone() {
        let cached_value = value.clone();
        tokio::spawn(async move {
            if let Err(e) = crate::db::writer::write_cached_response(&pool, CACHE_KEY, &cached_value).await {
                tracing::warn!("DB cache write failed for spreads: {}", e);
            }
        });
    }

    (headers, Json(value))
}

async fn db_fallback(state: &Arc<AppState>, key: &str) -> Option<serde_json::Value> {
    if let Some(pool) = &state.db {
        match crate::db::reader::get_cached_response(pool, key).await {
            Ok(Some(mut data)) => {
                if let Some(obj) = data.as_object_mut() {
                    obj.insert("stale".to_string(), serde_json::Value::Bool(true));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_css_calculation() {
        // css = 85.0 - (34.20 / 0.60) - (68.15 * 0.202)
        // css = 85.0 - 57.0 - 13.7663 = 14.2337
        let css = calculate_css(85.0, 34.20, 68.15);
        assert!((css - 14.23).abs() < 0.1);
    }

    #[test]
    fn test_cds_calculation() {
        // ara_eur_tonne = 112.50 / 1.08 = 104.1667
        // ara_eur_gj = 104.1667 / 29.31 = 3.5539
        // cds = 85.0 - (3.5539 / 0.42) - (68.15 * 0.341)
        // cds = 85.0 - 8.4617 - 23.2392 = 53.30
        let cds = calculate_cds(85.0, 112.50, 68.15, 0.42);
        assert!((cds - 53.30).abs() < 0.1);
    }

    #[test]
    fn test_dispatch_signal() {
        assert_eq!(dispatch_signal(14.0, -32.0), "GAS_MARGINAL");
        assert_eq!(dispatch_signal(-5.0, 10.0), "COAL_MARGINAL");
        assert_eq!(dispatch_signal(-5.0, -10.0), "NEGATIVE_SPREADS");
    }
}
