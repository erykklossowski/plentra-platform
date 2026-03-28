use std::sync::Arc;

use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use chrono::Utc;
use serde_json::{json, Value};

use crate::models::fuel::FuelData;
use crate::models::spread::SpreadData;
use crate::AppState;

pub async fn handler(State(state): State<Arc<AppState>>) -> (HeaderMap, Json<Value>) {
    let mut headers = HeaderMap::new();
    headers.insert("cache-control", "max-age=900".parse().unwrap());

    // Check cache
    if let Some(cached) = state.cache.get("summary") {
        return (headers, Json(cached.data));
    }

    // Get fuel and spread data from their caches (or they'll be populated by the routes)
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

    let summary = json!({
        "retrospective_text": format!(
            "{month_name} saw continued volatility in European energy markets. \
             TTF natural gas prices reflected supply-demand balancing amid varying LNG import levels. \
             EUA carbon permits maintained upward pressure on generation costs. \
             Clean spark spreads remained positive, supporting gas-fired generation dispatch, \
             while clean dark spreads stayed negative, indicating challenging economics for coal-fired units."
        ),
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
        "fetched_at": now.to_rfc3339()
    });

    state
        .cache
        .set("summary".to_string(), summary.clone(), state.config.cache_ttl_fuels);

    (headers, Json(summary))
}
