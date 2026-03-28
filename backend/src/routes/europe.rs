use std::sync::Arc;

use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use chrono::Utc;
use serde_json::Value;

use crate::fetchers::entsoe;
use crate::models::europe::{EURankingEntry, EuropeData, ExtremePriceEntry};
use crate::AppState;

const CACHE_KEY: &str = "europe";

pub async fn handler(State(state): State<Arc<AppState>>) -> (HeaderMap, Json<Value>) {
    let mut headers = HeaderMap::new();
    headers.insert("cache-control", "max-age=3600".parse().unwrap());

    // Check cache
    if let Some(cached) = state.cache.get(CACHE_KEY) {
        return (headers, Json(cached.data));
    }

    let token = match &state.config.entsoe_token {
        Some(t) if !t.is_empty() => t.clone(),
        _ => {
            return (
                headers,
                Json(serde_json::json!({
                    "error": "ENTSO-E API not configured",
                    "timestamp": Utc::now().to_rfc3339()
                })),
            );
        }
    };

    // Fetch DA prices for all EU zones
    let results = entsoe::fetch_eu_day_ahead_prices(&state.http_client, &token).await;

    // Build price entries from successful fetches
    let mut entries: Vec<(String, String, f64)> = Vec::new();
    for (code, name, result) in results {
        match result {
            Ok(hourly) => {
                let avg = entsoe::average_da_price(&hourly);
                if avg > 0.0 {
                    entries.push((code, name, avg));
                }
            }
            Err(e) => {
                tracing::warn!("Failed to fetch DA prices for {code}: {e}");
            }
        }
    }

    if entries.is_empty() {
        return (
            headers,
            Json(serde_json::json!({
                "error": "No European DA price data available",
                "timestamp": Utc::now().to_rfc3339()
            })),
        );
    }

    // Sort by price descending (most expensive first)
    entries.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    let max_price = entries.first().map(|e| e.2).unwrap_or(1.0);
    let sum: f64 = entries.iter().map(|e| e.2).sum();
    let eu_average = entsoe::round2(sum / entries.len() as f64);

    let mut poland_rank = 0u32;
    let mut poland_price = 0.0;

    let rankings: Vec<EURankingEntry> = entries
        .iter()
        .enumerate()
        .map(|(i, (code, name, price))| {
            let rank = (i + 1) as u32;
            let is_focus = code == "PL";
            if is_focus {
                poland_rank = rank;
                poland_price = *price;
            }
            EURankingEntry {
                rank,
                country_code: code.clone(),
                country_name: name.clone(),
                da_price_eur_mwh: *price,
                bar_pct: entsoe::round2(*price / max_price * 100.0),
                is_focus,
            }
        })
        .collect();

    let cheapest = entries.last().map(|(c, _, p)| ExtremePriceEntry {
        code: c.clone(),
        price: *p,
    }).unwrap_or(ExtremePriceEntry { code: "N/A".to_string(), price: 0.0 });

    let most_expensive = entries.first().map(|(c, _, p)| ExtremePriceEntry {
        code: c.clone(),
        price: *p,
    }).unwrap_or(ExtremePriceEntry { code: "N/A".to_string(), price: 0.0 });

    let data = EuropeData {
        rankings,
        poland_rank,
        poland_price,
        eu_average,
        cheapest,
        most_expensive,
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
