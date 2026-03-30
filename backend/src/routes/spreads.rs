use std::sync::Arc;

use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use serde_json::{json, Value};

use sqlx;

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

    // Try to build spreads from calculated_spreads (forward-looking DB data)
    let db_spread = if let Some(pool) = &state.db {
        build_spread_from_db(pool).await
    } else {
        None
    };

    let spread_data = if let Some(sd) = db_spread {
        sd
    } else {
        // Fallback: compute inline from fuel spot prices (old method)
        let fuel_value = if let Some(cached) = state.cache.get("fuels") {
            Some(cached.data)
        } else if let Some(stale) = state.cache.get_stale("fuels") {
            Some(stale.data)
        } else if let Some(pool) = &state.db {
            crate::db::reader::get_cached_response(pool, "fuels")
                .await
                .ok()
                .flatten()
        } else {
            None
        };

        match fuel_value.and_then(|v| serde_json::from_value::<FuelData>(v).ok()) {
            Some(fuel) => build_spread_from_fuel(&fuel),
            None => {
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

/// Build SpreadData from calculated_spreads table (forward-looking CSS + CDS).
async fn build_spread_from_db(pool: &sqlx::PgPool) -> Option<SpreadData> {
    // Fetch last 30 days of CSS and CDS
    let rows = sqlx::query_as::<_, (chrono::NaiveDate, String, f64, f64)>(
        r#"
        SELECT date, spread_type, value, carbon_price
        FROM calculated_spreads
        WHERE date >= CURRENT_DATE - INTERVAL '30 days'
          AND spread_type IN ('rolling_3m_css', 'rolling_3m_cds')
        ORDER BY date ASC
        "#,
    )
    .fetch_all(pool)
    .await
    .ok()?;

    if rows.is_empty() {
        return None;
    }

    // Pivot into per-date map
    let mut date_map: std::collections::BTreeMap<
        chrono::NaiveDate,
        (Option<f64>, Option<f64>, Option<f64>),
    > = std::collections::BTreeMap::new();
    for (date, spread_type, value, carbon_price) in &rows {
        let entry = date_map.entry(*date).or_insert((None, None, None));
        match spread_type.as_str() {
            "rolling_3m_css" => {
                entry.0 = Some(*value);
                entry.2 = Some(*carbon_price);
            }
            "rolling_3m_cds" => entry.1 = Some(*value),
            _ => {}
        }
    }

    let history: Vec<SpreadHistoryEntry> = date_map
        .iter()
        .filter_map(|(date, (css, cds, _))| {
            Some(SpreadHistoryEntry {
                date: date.to_string(),
                css: round2((*css)?),
                cds_42: round2(cds.unwrap_or(0.0)),
            })
        })
        .collect();

    if history.is_empty() {
        return None;
    }

    let latest_css = history.last().map(|h| h.css).unwrap_or(0.0);
    let latest_cds = history.last().map(|h| h.cds_42).unwrap_or(0.0);

    // MoM changes from 30-day series
    let css_series: Vec<f64> = history.iter().map(|h| h.css).collect();
    let cds_series: Vec<f64> = history.iter().map(|h| h.cds_42).collect();
    let css_pct = crate::fetchers::databento::mom_delta_pct(&css_series);
    let cds_pct = crate::fetchers::databento::mom_delta_pct(&cds_series);

    // Carbon impact from latest forward EUA price
    let latest_carbon = date_map.values().rev().find_map(|(_, _, cp)| *cp).unwrap_or(0.0);
    let carbon_impact = round2(crate::analytics::css::carbon_impact_factor(latest_carbon));

    Some(SpreadData {
        css_spot: round2(latest_css),
        css_spot_pct_change: css_pct,
        cds_spot_eta34: round2(latest_cds * 0.81), // approximate eta34/eta42 ratio
        cds_spot_eta42: round2(latest_cds),
        cds_spot_pct_change: cds_pct,
        css_term_y1: round2(latest_css),
        cds_term_y1: Some(round2(latest_cds)),
        baseload_profitability_eur_mwh: round2(latest_css.max(0.0)),
        peak_load_advantage_eur_mwh: round2(latest_css * 1.4),
        carbon_impact_factor: carbon_impact,
        dispatch_signal: dispatch_signal(latest_css, latest_cds).to_string(),
        history_30d: history,
        fetched_at: Utc::now().to_rfc3339(),
        stale: None,
    })
}

/// Fallback: build SpreadData from spot fuel prices (old inline method).
fn build_spread_from_fuel(fuel: &FuelData) -> SpreadData {
    let css_spot = round2(calculate_css(RDN_EUR_MWH, fuel.ttf_eur_mwh, fuel.eua_eur_tonne));
    let cds_spot_eta42 =
        round2(calculate_cds(RDN_EUR_MWH, fuel.ara_usd_tonne, fuel.eua_eur_tonne, 0.42));
    let cds_spot_eta34 =
        round2(calculate_cds(RDN_EUR_MWH, fuel.ara_usd_tonne, fuel.eua_eur_tonne, 0.34));

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
                date: format!("day-{}", i + 1),
                css: round2(calculate_css(RDN_EUR_MWH, ttf, eua)),
                cds_42: round2(calculate_cds(RDN_EUR_MWH, ara, eua, 0.42)),
            }
        })
        .collect();

    let css_history: Vec<f64> = history.iter().map(|h| h.css).collect();
    let cds_history: Vec<f64> = history.iter().map(|h| h.cds_42).collect();
    let css_pct = crate::fetchers::databento::mom_delta_pct(&css_history);
    let cds_pct = crate::fetchers::databento::mom_delta_pct(&cds_history);
    let carbon_impact = round2(-fuel.eua_eur_tonne * 0.202);

    SpreadData {
        css_spot,
        css_spot_pct_change: css_pct,
        cds_spot_eta34,
        cds_spot_eta42,
        cds_spot_pct_change: cds_pct,
        css_term_y1: round2(css_spot * 0.95),
        cds_term_y1: None,
        baseload_profitability_eur_mwh: round2(css_spot.max(0.0)),
        peak_load_advantage_eur_mwh: round2(css_spot * 1.4),
        carbon_impact_factor: carbon_impact,
        dispatch_signal: dispatch_signal(css_spot, cds_spot_eta42).to_string(),
        history_30d: history,
        fetched_at: Utc::now().to_rfc3339(),
        stale: None,
    }
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

/// GET /api/spreads/css — rolling 3-month clean spark spread from calculated_spreads.
pub async fn get_css(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let Some(ref pool) = state.db else {
        return Json(json!({ "error": "database not connected" })).into_response();
    };

    let rows = sqlx::query_as::<_, (chrono::NaiveDate, f64, f64, f64, f64, Vec<String>, Vec<String>, String)>(
        r#"
        SELECT date, value, power_avg, gas_avg, carbon_price,
               power_symbols, gas_symbols, carbon_symbol
        FROM calculated_spreads
        WHERE spread_type = 'rolling_3m_css'
          AND date >= CURRENT_DATE - INTERVAL '90 days'
        ORDER BY date ASC
        "#,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let series: Vec<_> = rows
        .iter()
        .map(|r| {
            json!({
                "date":          r.0.to_string(),
                "css":           r.1,
                "power_avg":     r.2,
                "gas_avg":       r.3,
                "carbon_price":  r.4,
                "power_symbols": r.5,
                "gas_symbols":   r.6,
                "carbon_symbol": r.7,
            })
        })
        .collect();

    Json(json!({
        "spread_type": "rolling_3m_css",
        "latest": rows.last().map(|r| json!({
            "date":  r.0.to_string(),
            "value": r.1,
        })),
        "series": series,
    }))
    .into_response()
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
