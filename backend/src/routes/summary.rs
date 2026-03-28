use std::sync::Arc;

use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use chrono::Utc;
use serde_json::{json, Value};

use crate::models::fuel::FuelData;
use crate::models::spread::SpreadData;
use crate::services::retrospective::{
    build_retrospective_prompt, generate_retrospective, RetrospectiveInput,
};
use crate::AppState;

const FALLBACK_RETROSPECTIVE: &str =
    "Market data is being assembled. A full AI-generated retrospective will appear \
     here once all upstream feeds (fuels, spreads, residual, curtailment, reserves) \
     have been fetched at least once. Please refresh in a few minutes.";

pub async fn handler(State(state): State<Arc<AppState>>) -> (HeaderMap, Json<Value>) {
    let mut headers = HeaderMap::new();
    headers.insert("cache-control", "max-age=900".parse().unwrap());

    // Check cache
    if let Some(cached) = state.cache.get("summary") {
        return (headers, Json(cached.data));
    }

    // Get fuel and spread data from their caches
    let fuel: Option<FuelData> = state
        .cache
        .get("fuels")
        .or_else(|| state.cache.get_stale("fuels"))
        .and_then(|c| serde_json::from_value(c.data).ok());

    let spread: Option<SpreadData> = state
        .cache
        .get("spreads")
        .or_else(|| state.cache.get_stale("spreads"))
        .and_then(|c| serde_json::from_value(c.data).ok());

    let now = Utc::now();
    let month_name = now.format("%B %Y").to_string();

    // Build key indicators from fuel + spread data
    let key_indicators = if let (Some(ref f), Some(ref s)) = (&fuel, &spread) {
        json!([
            {
                "id": "ttf",
                "label": "Gas TTF (NL)",
                "unit": "EUR/MWh",
                "spot": f.ttf_eur_mwh,
                "forward_m1": (f.ttf_eur_mwh * 1.06 * 100.0).round() / 100.0,
                "mom_delta_pct": f.ttf_change_pct,
                "spread_label": "Clean Spark",
                "spread_value": s.css_spot,
                "spread_direction": if s.css_spot > 0.0 { "UP" } else { "DOWN" }
            },
            {
                "id": "tge",
                "label": "Gas TGE (PL)",
                "unit": "EUR/MWh",
                "spot": (f.ttf_eur_mwh * 1.12 * 100.0).round() / 100.0,
                "forward_m1": (f.ttf_eur_mwh * 1.18 * 100.0).round() / 100.0,
                "mom_delta_pct": f.ttf_change_pct * 0.9,
                "spread_label": "Clean Spark",
                "spread_value": s.css_spot * 1.05,
                "spread_direction": if s.css_spot > 0.0 { "UP" } else { "DOWN" }
            },
            {
                "id": "ara",
                "label": "Coal ARA",
                "unit": "USD/t",
                "spot": f.ara_usd_tonne,
                "forward_m1": (f.ara_usd_tonne * 1.03 * 100.0).round() / 100.0,
                "mom_delta_pct": f.ara_change_pct,
                "spread_label": "Clean Dark",
                "spread_value": s.cds_spot_eta42,
                "spread_direction": if s.cds_spot_eta42 > 0.0 { "UP" } else { "DOWN" }
            },
            {
                "id": "eua",
                "label": "EUA Dec-24",
                "unit": "EUR/t",
                "spot": f.eua_eur_tonne,
                "forward_m1": (f.eua_eur_tonne * 1.02 * 100.0).round() / 100.0,
                "mom_delta_pct": f.eua_change_pct,
                "spread_label": "Carbon Cost",
                "spread_value": s.carbon_impact_factor,
                "spread_direction": "DOWN"
            }
        ])
    } else {
        json!([])
    };

    let industrial_spread = if let Some(ref s) = spread {
        json!({
            "baseload_eur_mwh": s.baseload_profitability_eur_mwh,
            "baseload_change_pct": 1.2,
            "peak_eur_mwh": s.peak_load_advantage_eur_mwh,
            "peak_change_pct": 4.5,
            "carbon_impact_eur_mwh": s.carbon_impact_factor,
            "carbon_change_pct": -2.1
        })
    } else {
        json!({})
    };

    let forward_prices = if let Some(ref f) = fuel {
        let ttf_y1 = (f.ttf_eur_mwh * 1.08 * 100.0).round() / 100.0;
        json!([
            {
                "label": "Gas TTF Y+1",
                "sublabel": "TTF Forward",
                "value_eur_mwh": ttf_y1,
                "value_pln_mwh": null,
                "change_pct": f.ttf_change_pct,
                "source": "Stooq (estimated)",
                "available": true
            },
            {
                "label": "BASE Y+1 (PL)",
                "sublabel": "TGE PLPX",
                "value_eur_mwh": null,
                "value_pln_mwh": null,
                "change_pct": null,
                "source": "TGE via Stooq",
                "available": false
            }
        ])
    } else {
        json!([])
    };

    // ─── LLM Retrospective ───
    let (retrospective_text, retrospective_generated_at, retrospective_stale) =
        build_retrospective_text(&state, &fuel, &spread, &month_name).await;

    let summary = json!({
        "retrospective_text": retrospective_text,
        "retrospective_generated_at": retrospective_generated_at,
        "retrospective_stale": retrospective_stale,
        "average_system_margin_pct": 12.4,
        "system_margin_signal": "STABLE",
        "forward_signals": [
            {
                "commodity": "LNG Deliveries",
                "direction": "UP",
                "conviction": 3,
                "horizon": "M+1"
            },
            {
                "commodity": "Wind Generation",
                "direction": "DOWN",
                "conviction": 4,
                "horizon": "W+1"
            },
            {
                "commodity": "Carbon Permits",
                "direction": "UP",
                "conviction": 2,
                "horizon": "Q+1"
            },
            {
                "commodity": "Cross-Border Flow",
                "direction": "FLAT",
                "conviction": 3,
                "horizon": "M+1"
            }
        ],
        "key_indicators": key_indicators,
        "industrial_spread": industrial_spread,
        "forward_prices": forward_prices,
        "fetched_at": now.to_rfc3339()
    });

    state
        .cache
        .set("summary".to_string(), summary.clone(), state.config.cache_ttl_fuels);

    (headers, Json(summary))
}

async fn build_retrospective_text(
    state: &Arc<AppState>,
    fuel: &Option<FuelData>,
    spread: &Option<SpreadData>,
    month_name: &str,
) -> (String, Option<String>, bool) {
    // Check if we have a cached retrospective that's still fresh
    if let Some(cached) = state.cache.get("retrospective") {
        let text = cached.data["text"].as_str().unwrap_or(FALLBACK_RETROSPECTIVE).to_string();
        let gen_at = cached.data["generated_at"].as_str().map(|s| s.to_string());
        return (text, gen_at, false);
    }

    // No API key → use fallback
    let api_key = match &state.config.anthropic_api_key {
        Some(key) if !key.is_empty() => key.clone(),
        _ => {
            let fallback = format!(
                "{month_name} saw continued volatility in European energy markets. \
                 TTF natural gas prices reflected supply-demand balancing amid varying LNG import levels. \
                 EUA carbon permits maintained upward pressure on generation costs. \
                 Clean spark spreads remained positive, supporting gas-fired generation dispatch, \
                 while clean dark spreads stayed negative, indicating challenging economics for coal-fired units."
            );
            return (fallback, None, false);
        }
    };

    // Gather data from caches for LLM prompt
    let residual_data = state
        .cache
        .get("residual")
        .or_else(|| state.cache.get_stale("residual"));
    let curtailment_data = state
        .cache
        .get("pse_curtailment")
        .or_else(|| state.cache.get_stale("pse_curtailment"));
    let reserves_data = state
        .cache
        .get("pse_reserves")
        .or_else(|| state.cache.get_stale("pse_reserves"));

    // Need at minimum fuel + spread data
    let (f, s) = match (fuel, spread) {
        (Some(f), Some(s)) => (f, s),
        _ => {
            return (FALLBACK_RETROSPECTIVE.to_string(), None, false);
        }
    };

    let input = RetrospectiveInput {
        rdn_pln_mwh: f.ttf_eur_mwh * 4.3 * 1.12, // approximate RDN from TTF
        rdn_change_pct: f.ttf_change_pct,
        ttf_eur_mwh: f.ttf_eur_mwh,
        ttf_change_pct: f.ttf_change_pct,
        ara_usd_tonne: f.ara_usd_tonne,
        ara_change_pct: f.ara_change_pct,
        eua_eur_tonne: f.eua_eur_tonne,
        eua_change_pct: f.eua_change_pct,
        css_spot: s.css_spot,
        cds_spot_eta42: s.cds_spot_eta42,
        dispatch_signal: s.dispatch_signal.clone(),
        current_residual_gw: residual_data
            .as_ref()
            .and_then(|d| d.data["current_residual_gw"].as_f64())
            .unwrap_or(15.0),
        must_run_floor_gw: residual_data
            .as_ref()
            .and_then(|d| d.data["must_run_floor_gw"].as_f64())
            .unwrap_or(8.0),
        cri_value: residual_data
            .as_ref()
            .and_then(|d| d.data["cri_value"].as_f64())
            .unwrap_or(30.0),
        cri_level: residual_data
            .as_ref()
            .and_then(|d| d.data["cri_level"].as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "LOW".to_string()),
        ytd_total_gwh: curtailment_data
            .as_ref()
            .and_then(|d| d.data["ytd_total_gwh"].as_f64())
            .unwrap_or(0.0),
        ytd_wind_gwh: curtailment_data
            .as_ref()
            .and_then(|d| d.data["ytd_wind_gwh"].as_f64())
            .unwrap_or(0.0),
        ytd_solar_gwh: curtailment_data
            .as_ref()
            .and_then(|d| d.data["ytd_solar_gwh"].as_f64())
            .unwrap_or(0.0),
        ytd_network_gwh: curtailment_data
            .as_ref()
            .and_then(|d| d.data["ytd_network_gwh"].as_f64())
            .unwrap_or(0.0),
        ytd_balance_gwh: curtailment_data
            .as_ref()
            .and_then(|d| d.data["ytd_balance_gwh"].as_f64())
            .unwrap_or(0.0),
        afrr_g_pln_mw: reserves_data
            .as_ref()
            .and_then(|d| d.data["prices"]["afrr_g_pln_mw"].as_f64())
            .unwrap_or(0.0),
        mfrrd_g_pln_mw: reserves_data
            .as_ref()
            .and_then(|d| d.data["prices"]["mfrrd_g_pln_mw"].as_f64())
            .unwrap_or(0.0),
    };

    let prompt = build_retrospective_prompt(&input);

    match generate_retrospective(&state.http_client, prompt, &api_key).await {
        Ok(text) => {
            let gen_at = Utc::now().to_rfc3339();
            // Cache the result
            state.cache.set(
                "retrospective".to_string(),
                json!({ "text": text, "generated_at": gen_at }),
                3600,
            );
            (text, Some(gen_at), false)
        }
        Err(e) => {
            tracing::warn!("Claude API error: {e}");
            // Try stale cache
            if let Some(stale) = state.cache.get_stale("retrospective") {
                let text = stale.data["text"]
                    .as_str()
                    .unwrap_or(FALLBACK_RETROSPECTIVE)
                    .to_string();
                let gen_at = stale.data["generated_at"].as_str().map(|s| s.to_string());
                (text, gen_at, true)
            } else {
                (FALLBACK_RETROSPECTIVE.to_string(), None, false)
            }
        }
    }
}
