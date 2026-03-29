use std::sync::Arc;

use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use chrono::Utc;
use serde_json::Value;

use crate::fetchers::entsoe;
use crate::models::fuel::FuelData;
use crate::models::generation::{GenerationData, JKZEntry};
use crate::models::spread::SpreadData;
use crate::AppState;

const CACHE_KEY: &str = "generation";

/// Standard Polish generation technologies with their parameters
struct TechSpec {
    name: &'static str,
    fuel: FuelType,
    efficiency: f64,
    emission_factor: f64, // tCO2/MWh_el
}

enum FuelType {
    Gas,
    HardCoal,
    Lignite,
    Wind,
    Solar,
}

const TECHS: &[TechSpec] = &[
    TechSpec { name: "CCGT", fuel: FuelType::Gas, efficiency: 0.60, emission_factor: 0.202 },
    TechSpec { name: "Hard Coal (η=42%)", fuel: FuelType::HardCoal, efficiency: 0.42, emission_factor: 0.341 },
    TechSpec { name: "Hard Coal (η=34%)", fuel: FuelType::HardCoal, efficiency: 0.34, emission_factor: 0.341 },
    TechSpec { name: "Lignite", fuel: FuelType::Lignite, efficiency: 0.38, emission_factor: 0.364 },
    TechSpec { name: "Wind Onshore", fuel: FuelType::Wind, efficiency: 1.0, emission_factor: 0.0 },
    TechSpec { name: "Solar PV", fuel: FuelType::Solar, efficiency: 1.0, emission_factor: 0.0 },
];

fn build_jkz_table(
    ttf_eur_mwh: f64,
    ara_eur_tonne: f64,
    eua_eur_tonne: f64,
    da_price: f64,
) -> Vec<JKZEntry> {
    // Convert ARA coal to EUR/GJ: 1 tonne coal = 29.31 GJ
    let ara_eur_gj = ara_eur_tonne / 29.31;

    TECHS
        .iter()
        .map(|tech| {
            let fuel_cost = match tech.fuel {
                FuelType::Gas => ttf_eur_mwh / tech.efficiency,
                FuelType::HardCoal => ara_eur_gj / tech.efficiency,
                FuelType::Lignite => {
                    // Lignite has no market price; use typical Polish cost ~5 EUR/GJ
                    5.0 / tech.efficiency
                }
                FuelType::Wind | FuelType::Solar => 0.0,
            };

            let co2_cost = eua_eur_tonne * tech.emission_factor;
            let jkz = entsoe::round2(fuel_cost + co2_cost);
            let clean_spread = entsoe::round2(da_price - jkz);

            let dispatch_status = if matches!(tech.fuel, FuelType::Wind | FuelType::Solar) {
                "MUST_RUN".to_string()
            } else if clean_spread > 0.0 {
                "IN_MERIT".to_string()
            } else {
                "OUT_OF_MERIT".to_string()
            };

            JKZEntry {
                technology: tech.name.to_string(),
                efficiency: tech.efficiency,
                emission_factor: tech.emission_factor,
                fuel_cost_eur_mwh: entsoe::round2(fuel_cost),
                co2_cost_eur_mwh: entsoe::round2(co2_cost),
                jkz_eur_mwh: jkz,
                clean_spread_eur_mwh: clean_spread,
                dispatch_status,
            }
        })
        .collect()
}

pub async fn handler(State(state): State<Arc<AppState>>) -> (HeaderMap, Json<Value>) {
    let mut headers = HeaderMap::new();
    headers.insert("cache-control", "max-age=900".parse().unwrap());

    // Check cache
    if let Some(cached) = state.cache.get(CACHE_KEY) {
        return (headers, Json(cached.data));
    }

    // Get fuel data — try cache, stale cache, then DB fallback
    let fuel_value = if let Some(cached) = state.cache.get("fuels").or_else(|| state.cache.get_stale("fuels")) {
        Some(cached.data)
    } else if let Some(pool) = &state.db {
        tracing::info!("generation: fuel cache empty, trying DB fallback");
        match crate::db::reader::get_cached_response(pool, "fuels").await {
            Ok(Some(v)) => Some(v),
            _ => None,
        }
    } else {
        None
    };

    let fuel: FuelData = match fuel_value.and_then(|v| serde_json::from_value(v).ok()) {
        Some(f) => f,
        None => {
            // Last resort: serve DB-cached generation response
            if let Some(pool) = &state.db {
                if let Ok(Some(mut data)) = crate::db::reader::get_cached_response(pool, CACHE_KEY).await {
                    if let Some(obj) = data.as_object_mut() {
                        obj.insert("stale".to_string(), Value::Bool(true));
                    }
                    tracing::info!("Serving generation from DB fallback");
                    return (headers, Json(data));
                }
            }
            return (
                headers,
                Json(serde_json::json!({
                    "data_status": "unavailable",
                    "message": "Generation data temporarily unavailable",
                    "jkz_table": [],
                    "dispatch_signal": "UNKNOWN",
                    "css_spot": null,
                    "cds_spot_eta42": null,
                    "css_history_30d": [],
                    "cds_history_30d": [],
                    "eur_usd_rate": 0.0,
                    "rdn_eur_mwh": 0.0,
                    "fetched_at": Utc::now().to_rfc3339(),
                    "stale": true,
                })),
            );
        }
    };

    // Get spread data
    let spread_value = state.cache.get("spreads").or_else(|| state.cache.get_stale("spreads"));
    let spread: Option<SpreadData> =
        spread_value.and_then(|c| serde_json::from_value(c.data).ok());

    // EUR/USD rate — hardcoded fallback (previously fetched from Stooq, now deprecated)
    let eur_usd = 1.08;

    // Get DA price from ENTSO-E, fall back to DB-cached value
    let da_price = if let Some(token) = &state.config.entsoe_token {
        if !token.is_empty() {
            match entsoe::fetch_day_ahead_prices(&state.http_client, token, "10YPL-AREA-----S")
                .await
            {
                Ok(hourly) => entsoe::average_da_price(&hourly),
                Err(_) => da_price_from_db(&state).await,
            }
        } else {
            da_price_from_db(&state).await
        }
    } else {
        da_price_from_db(&state).await
    };

    // Convert ARA from USD to EUR
    let ara_eur_tonne = fuel.ara_usd_tonne / eur_usd;

    let jkz_table = build_jkz_table(fuel.ttf_eur_mwh, ara_eur_tonne, fuel.eua_eur_tonne, da_price);

    // Derive dispatch signal from JKZ
    let gas_jkz = jkz_table.iter().find(|j| j.technology == "CCGT").map(|j| j.jkz_eur_mwh).unwrap_or(0.0);
    let coal_jkz = jkz_table.iter().find(|j| j.technology.starts_with("Hard Coal (η=42%)")).map(|j| j.jkz_eur_mwh).unwrap_or(0.0);
    let css = da_price - gas_jkz;
    let cds = da_price - coal_jkz;

    let dispatch_signal = if css > 0.0 && css > cds {
        "GAS_MARGINAL"
    } else if cds > 0.0 && cds > css {
        "COAL_MARGINAL"
    } else {
        "NEGATIVE_SPREADS"
    };

    let (css_history, cds_history) = if let Some(ref s) = spread {
        (
            s.history_30d.iter().map(|h| h.css).collect(),
            s.history_30d.iter().map(|h| h.cds_42).collect(),
        )
    } else {
        (vec![], vec![])
    };

    let data = GenerationData {
        jkz_table,
        dispatch_signal: dispatch_signal.to_string(),
        css_spot: spread.as_ref().map(|s| s.css_spot).unwrap_or(entsoe::round2(css)),
        cds_spot_eta42: spread.as_ref().map(|s| s.cds_spot_eta42).unwrap_or(entsoe::round2(cds)),
        css_history_30d: css_history,
        cds_history_30d: cds_history,
        eur_usd_rate: entsoe::round2(eur_usd),
        rdn_eur_mwh: da_price,
        fetched_at: Utc::now().to_rfc3339(),
        stale: None,
    };

    let value = serde_json::to_value(&data).unwrap();
    state
        .cache
        .set(CACHE_KEY.to_string(), value.clone(), state.config.cache_ttl_fuels);

    // Persist to DB for future fallback
    if let Some(pool) = state.db.clone() {
        let key = CACHE_KEY.to_string();
        let data = value.clone();
        tokio::spawn(async move {
            if let Err(e) = crate::db::writer::write_cached_response(&pool, &key, &data).await {
                tracing::warn!("DB cache write failed for {}: {}", key, e);
            }
        });
    }

    (headers, Json(value))
}

/// Get DA price from the last cached generation response in DB.
async fn da_price_from_db(state: &Arc<AppState>) -> f64 {
    if let Some(pool) = &state.db {
        if let Ok(Some(data)) = crate::db::reader::get_cached_response(pool, CACHE_KEY).await {
            if let Some(price) = data.get("rdn_eur_mwh").and_then(|v| v.as_f64()) {
                return price;
            }
        }
    }
    0.0
}
