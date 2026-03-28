use std::sync::Arc;

use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use chrono::{Datelike, Utc};
use serde_json::Value;

use crate::fetchers::entsoe;
use crate::models::residual::{HeatmapEntry, HourlyProfileEntry, ResidualData};
use crate::AppState;

const CACHE_KEY: &str = "residual";

const MONTHS: [&str; 12] = [
    "JAN", "FEB", "MAR", "APR", "MAY", "JUN",
    "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
];

// Seasonal adjustment factors for heatmap extrapolation (demand relative to average)
const SEASONAL_FACTORS: [f64; 12] = [
    1.15, 1.10, 1.0, 0.90, 0.80, 0.75,
    0.70, 0.72, 0.85, 0.95, 1.05, 1.12,
];

pub async fn handler(State(state): State<Arc<AppState>>) -> (HeaderMap, Json<Value>) {
    let mut headers = HeaderMap::new();
    headers.insert("cache-control", "max-age=3600".parse().unwrap());

    // Check cache
    if let Some(cached) = state.cache.get(CACHE_KEY) {
        return (headers, Json(cached.data));
    }

    // Check ENTSO-E token
    let token = match &state.config.entsoe_token {
        Some(t) if !t.is_empty() => t.clone(),
        _ => {
            return (
                headers,
                Json(serde_json::json!({
                    "error": "ENTSO-E API not configured. Set ENTSOE_TOKEN environment variable.",
                    "timestamp": Utc::now().to_rfc3339()
                })),
            );
        }
    };

    // Fetch all data in parallel
    let (gen_res, load_res, hourly_gen_res, hourly_load_res) = tokio::join!(
        entsoe::fetch_actual_generation(&state.http_client, &token),
        entsoe::fetch_actual_load(&state.http_client, &token),
        entsoe::fetch_hourly_generation(&state.http_client, &token),
        entsoe::fetch_hourly_load(&state.http_client, &token),
    );

    match (gen_res, load_res) {
        (Ok(gen), Ok(load_mw)) => {
            let wind_mw = gen.wind_mw();
            let solar_mw = gen.solar_mw();
            let renewable_mw = gen.total_renewable_mw();

            let current_month = Utc::now().month();
            let residual_gw = entsoe::calculate_residual_demand_gw(load_mw, wind_mw, solar_mw);
            let must_run_gw = entsoe::calculate_must_run_floor_gw(&gen, current_month);
            let stability_margin = entsoe::round2(residual_gw - must_run_gw);

            let residual_mw = load_mw - wind_mw - solar_mw;
            let must_run_mw = must_run_gw * 1000.0;

            let (cri_value, cri_level) =
                entsoe::calculate_cri(load_mw, residual_mw, must_run_mw, renewable_mw);
            let congestion_pct = entsoe::calculate_congestion_probability(cri_value);

            // Build hourly profile
            let hourly_profile = build_hourly_profile(
                &hourly_gen_res.unwrap_or_default(),
                &hourly_load_res.unwrap_or_default(),
                residual_gw,
                must_run_gw,
                current_month,
            );

            // Build heatmap from hourly profile
            let heatmap_data = build_heatmap(&hourly_profile);

            // Calculate correlation from hourly data
            let residual_vals: Vec<f64> = hourly_profile.iter().map(|h| h.residual_gw).collect();
            let must_run_vals: Vec<f64> = hourly_profile.iter().map(|h| h.must_run_gw).collect();
            let (r, r2, p) = entsoe::calculate_correlation(&residual_vals, &must_run_vals);

            // Estimate curtailment
            let curtailment_mwh: f64 = hourly_profile
                .iter()
                .map(|h| {
                    let excess = (h.must_run_gw - h.residual_gw).max(0.0) * 1000.0; // MW
                    excess // 1 hour * MW = MWh
                })
                .sum();

            let day_of_year = Utc::now().ordinal() as f64;
            let ytd_curtailment_gwh = entsoe::round2(curtailment_mwh * day_of_year / 1000.0);
            let forecast_curtailment_gwh = entsoe::round2(ytd_curtailment_gwh * 365.0 / day_of_year);

            let wind_ratio = if renewable_mw > 0.0 { wind_mw / renewable_mw } else { 0.5 };
            let wind_reduction_gwh = entsoe::round2(ytd_curtailment_gwh * wind_ratio);
            let solar_reduction_gwh = entsoe::round2(ytd_curtailment_gwh * (1.0 - wind_ratio));

            let data = ResidualData {
                current_residual_gw: residual_gw,
                must_run_floor_gw: must_run_gw,
                stability_margin_gw: stability_margin,
                congestion_probability_pct: congestion_pct,
                cri_value,
                cri_level,
                hourly_profile,
                heatmap_data,
                ytd_curtailment_gwh,
                forecast_curtailment_gwh,
                wind_reduction_gwh,
                solar_reduction_gwh,
                correlation_r: r,
                correlation_r2: r2,
                correlation_p: p,
                fetched_at: Utc::now().to_rfc3339(),
                stale: None,
            };

            let value = serde_json::to_value(&data).unwrap();
            state.cache.set(
                CACHE_KEY.to_string(),
                value.clone(),
                state.config.cache_ttl_entsoe,
            );

            (headers, Json(value))
        }
        _ => {
            // Fetch failed — try stale cache
            if let Some(stale) = state.cache.get_stale(CACHE_KEY) {
                let mut data = stale.data;
                if let Some(obj) = data.as_object_mut() {
                    obj.insert("stale".to_string(), Value::Bool(true));
                }
                (headers, Json(data))
            } else {
                (
                    headers,
                    Json(serde_json::json!({
                        "error": "Failed to fetch ENTSO-E data and no cache available",
                        "timestamp": Utc::now().to_rfc3339()
                    })),
                )
            }
        }
    }
}

fn build_hourly_profile(
    hourly_gen: &[(u32, entsoe::GenerationByType)],
    hourly_load: &[(u32, f64)],
    fallback_residual: f64,
    fallback_must_run: f64,
    month: u32,
) -> Vec<HourlyProfileEntry> {
    let load_map: std::collections::HashMap<u32, f64> =
        hourly_load.iter().map(|(h, v)| (*h, *v)).collect();

    if hourly_gen.is_empty() {
        // Generate synthetic profile based on current values
        return (0..24)
            .map(|h| {
                let factor = 1.0 + (h as f64 * std::f64::consts::PI / 12.0).sin() * 0.15;
                HourlyProfileEntry {
                    hour: h,
                    residual_gw: entsoe::round2(fallback_residual * factor),
                    must_run_gw: fallback_must_run,
                }
            })
            .collect();
    }

    (0..24)
        .map(|h| {
            let gen = hourly_gen
                .iter()
                .find(|(hour, _)| *hour == h)
                .map(|(_, g)| g);

            let load_mw = load_map.get(&h).copied().unwrap_or(fallback_residual * 1000.0);

            if let Some(g) = gen {
                let wind = g.wind_mw();
                let solar = g.solar_mw();
                let residual = entsoe::calculate_residual_demand_gw(load_mw, wind, solar);
                let must_run = entsoe::calculate_must_run_floor_gw(g, month);
                HourlyProfileEntry {
                    hour: h,
                    residual_gw: residual,
                    must_run_gw: must_run,
                }
            } else {
                HourlyProfileEntry {
                    hour: h,
                    residual_gw: fallback_residual,
                    must_run_gw: fallback_must_run,
                }
            }
        })
        .collect()
}

fn build_heatmap(hourly_profile: &[HourlyProfileEntry]) -> Vec<HeatmapEntry> {
    let mut heatmap = Vec::with_capacity(288);

    for (month_idx, month) in MONTHS.iter().enumerate() {
        let factor = SEASONAL_FACTORS[month_idx];
        for entry in hourly_profile {
            heatmap.push(HeatmapEntry {
                month: month.to_string(),
                hour: entry.hour,
                value: entsoe::round2(entry.residual_gw * factor),
            });
        }
    }

    heatmap
}
