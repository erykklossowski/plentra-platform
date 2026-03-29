use std::sync::Arc;

use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use chrono::Utc;
use serde_json::{json, Value};

use crate::fetchers::stooq;
use crate::models::fuel::FuelData;
use crate::models::spread::{SpreadData, SpreadHistoryEntry};
use crate::services::retrospective::{
    build_retrospective_prompt, generate_retrospective, RetrospectiveInput,
};
use crate::AppState;


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
    let (retrospective_text, retrospective_generated_at, retrospective_stale, retro_is_fallback) =
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

    // Only cache when we have a real LLM-generated retrospective
    if !retro_is_fallback {
        state
            .cache
            .set("summary".to_string(), summary.clone(), state.config.cache_ttl_fuels);
    }

    (headers, Json(summary))
}

/// Returns `(text, generated_at, is_stale, is_fallback)`.
/// `is_fallback = true` means we couldn't generate a real retrospective; caller must not cache.
async fn build_retrospective_text(
    state: &Arc<AppState>,
    fuel_opt: &Option<FuelData>,
    spread_opt: &Option<SpreadData>,
    _month_name: &str,
) -> (String, Option<String>, bool, bool) {
    // Check if we have a cached retrospective that's still fresh
    if let Some(cached) = state.cache.get("retrospective") {
        let text = cached.data["text"].as_str().unwrap_or("").to_string();
        let gen_at = cached.data["generated_at"].as_str().map(|s| s.to_string());
        return (text, gen_at, false, false);
    }

    // No API key — return stale cache if available, otherwise empty
    let api_key = match &state.config.anthropic_api_key {
        Some(key) if !key.is_empty() => key.clone(),
        _ => {
            if let Some(stale) = state.cache.get_stale("retrospective") {
                let text = stale.data["text"].as_str().unwrap_or("").to_string();
                let gen_at = stale.data["generated_at"].as_str().map(|s| s.to_string());
                return (text, gen_at, true, false);
            }
            return ("".to_string(), None, false, true);
        }
    };

    // Resolve fuel data — use cached value or actively fetch from Stooq
    let fuel: FuelData = if let Some(f) = fuel_opt.clone() {
        f
    } else {
        let (ttf_res, ara_res, eua_res) = tokio::join!(
            stooq::fetch_ttf(&state.http_client),
            stooq::fetch_ara(&state.http_client),
            stooq::fetch_eua(&state.http_client),
        );
        match (ttf_res, ara_res, eua_res) {
            (Ok(ttf), Ok(ara), Ok(eua)) => {
                let fd = FuelData {
                    ttf_eur_mwh: ttf.current_price,
                    ttf_change_pct: ttf.change_pct,
                    ttf_history_30d: ttf.history_30d,
                    ara_usd_tonne: ara.current_price,
                    ara_change_pct: ara.change_pct,
                    ara_history_30d: ara.history_30d,
                    eua_eur_tonne: eua.current_price,
                    eua_change_pct: eua.change_pct,
                    eua_history_30d: eua.history_30d,
                    fetched_at: Utc::now().to_rfc3339(),
                    stale: None,
                };
                let v = serde_json::to_value(&fd).unwrap();
                state.cache.set("fuels".to_string(), v, state.config.cache_ttl_fuels);
                fd
            }
            _ => {
                tracing::warn!("summary: fuel fetch failed on cold start — no retrospective");
                if let Some(stale) = state.cache.get_stale("retrospective") {
                    let text = stale.data["text"].as_str().unwrap_or("").to_string();
                    let gen_at = stale.data["generated_at"].as_str().map(|s| s.to_string());
                    return (text, gen_at, true, false);
                }
                return ("".to_string(), None, false, true);
            }
        }
    };

    // Resolve spread data — compute from fuel if not cached
    let spread: SpreadData = if let Some(s) = spread_opt.clone() {
        s
    } else {
        const RDN: f64 = 85.0;
        const EUR_USD: f64 = 1.08;
        let css = ((RDN - (fuel.ttf_eur_mwh / 0.60) - (fuel.eua_eur_tonne * 0.202)) * 100.0).round() / 100.0;
        let ara_eur_gj = (fuel.ara_usd_tonne / EUR_USD) / 29.31;
        let cds42 = ((RDN - (ara_eur_gj / 0.42) - (fuel.eua_eur_tonne * 0.341)) * 100.0).round() / 100.0;
        let cds34 = ((RDN - (ara_eur_gj / 0.34) - (fuel.eua_eur_tonne * 0.341)) * 100.0).round() / 100.0;
        let dispatch_signal = if css > 0.0 && css > cds42 {
            "GAS_MARGINAL"
        } else if cds42 > 0.0 && cds42 > css {
            "COAL_MARGINAL"
        } else {
            "NEGATIVE_SPREADS"
        };
        let len = fuel.ttf_history_30d.len().min(fuel.ara_history_30d.len()).min(fuel.eua_history_30d.len());
        let history: Vec<SpreadHistoryEntry> = (0..len).map(|i| {
            let t = fuel.ttf_history_30d[i];
            let a = fuel.ara_history_30d[i];
            let e = fuel.eua_history_30d[i];
            let ag = (a / EUR_USD) / 29.31;
            SpreadHistoryEntry {
                date: format!("day-{}", i + 1),
                css: ((RDN - (t / 0.60) - (e * 0.202)) * 100.0).round() / 100.0,
                cds_42: ((RDN - (ag / 0.42) - (e * 0.341)) * 100.0).round() / 100.0,
            }
        }).collect();
        let sd = SpreadData {
            css_spot: css,
            css_spot_pct_change: 0.0,
            cds_spot_eta34: cds34,
            cds_spot_eta42: cds42,
            cds_spot_pct_change: 0.0,
            css_term_y1: (css * 0.95 * 100.0).round() / 100.0,
            cds_term_y1: None,
            baseload_profitability_eur_mwh: css.max(0.0),
            peak_load_advantage_eur_mwh: (css * 1.4 * 100.0).round() / 100.0,
            carbon_impact_factor: (-fuel.eua_eur_tonne * 0.202 * 100.0).round() / 100.0,
            dispatch_signal: dispatch_signal.to_string(),
            history_30d: history,
            fetched_at: Utc::now().to_rfc3339(),
            stale: None,
        };
        let v = serde_json::to_value(&sd).unwrap();
        state.cache.set("spreads".to_string(), v, state.config.cache_ttl_fuels);
        sd
    };

    // Gather optional contextual data from caches
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

    let f = &fuel;
    let s = &spread;

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
            state.cache.set(
                "retrospective".to_string(),
                json!({ "text": text, "generated_at": gen_at }),
                43200, // 12h — refresh twice a day
            );
            (text, Some(gen_at), false, false)
        }
        Err(e) => {
            tracing::warn!("Claude API error: {e}");
            // Try stale cache
            if let Some(stale) = state.cache.get_stale("retrospective") {
                let text = stale.data["text"].as_str().unwrap_or("").to_string();
                let gen_at = stale.data["generated_at"].as_str().map(|s| s.to_string());
                (text, gen_at, true, false)
            } else {
                ("".to_string(), None, false, true)
            }
        }
    }
}
