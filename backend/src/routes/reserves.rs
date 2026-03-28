use std::sync::Arc;

use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use chrono::Utc;
use serde_json::{json, Value};

use crate::fetchers::pse::{
    build_monthly_avg_history, daily_avg_reserve_price, fetch_pse,
    date_days_ago, thirteen_months_ago, today_warsaw, round2,
    ReservePriceRecord,
};
use crate::AppState;

pub async fn handler(State(state): State<Arc<AppState>>) -> (HeaderMap, Json<Value>) {
    let mut headers = HeaderMap::new();
    headers.insert("cache-control", "max-age=3600".parse().unwrap());

    // Check fresh cache first
    if let Some(cached) = state.cache.get("pse_reserves") {
        return (headers, Json(cached.data));
    }

    let today = today_warsaw();
    let date_13m_ago = thirteen_months_ago();
    let date_30d_ago = date_days_ago(30);

    // Today's prices (single day — 24 records, fits in one request)
    let today_filter = format!(
        "business_date ge '{}' and business_date le '{}'",
        today, today
    );
    let price_records: Vec<ReservePriceRecord> = fetch_pse(
        &state.http_client,
        "cmbp-tp",
        &today_filter,
    )
    .await
    .unwrap_or_default();

    // 30-day daily history (for dual-axis chart vs CSS)
    let daily_30d_records = fetch_reserves_daily(
        &state.http_client,
        &date_30d_ago,
        &today,
    )
    .await;

    // Build daily averages for the 30-day window
    let daily_30d = build_daily_avg_history(&daily_30d_records);

    // 13-month history — sample weekly to limit API calls
    let history_records = fetch_reserves_sampled(
        &state.http_client,
        &date_13m_ago,
        &today,
    )
    .await;

    let history_13m = build_monthly_avg_history(&history_records);

    let avg = |f: fn(&ReservePriceRecord) -> Option<f64>| -> f64 {
        daily_avg_reserve_price(&price_records, &today, f)
    };

    let result = json!({
        "date": today,
        "prices": {
            "afrr_d_pln_mw": avg(|r| r.afrr_d),
            "afrr_g_pln_mw": avg(|r| r.afrr_g),
            "mfrrd_d_pln_mw": avg(|r| r.mfrrd_d),
            "mfrrd_g_pln_mw": avg(|r| r.mfrrd_g),
            "fcr_d_pln_mw": avg(|r| r.fcr_d),
            "fcr_g_pln_mw": avg(|r| r.fcr_g),
            "rr_g_pln_mw": avg(|r| r.rr_g),
        },
        "daily_30d": daily_30d,
        "history_13m": history_13m,
        "source": "PSE CMBP-TP",
        "fetched_at": Utc::now().to_rfc3339(),
    });

    state
        .cache
        .set("pse_reserves".to_string(), result.clone(), 3600);

    (headers, Json(result))
}

/// Fetch reserve prices for each day in a range (one API call per day).
async fn fetch_reserves_daily(
    client: &reqwest::Client,
    start_date: &str,
    end_date: &str,
) -> Vec<ReservePriceRecord> {
    let start = chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d");
    let end = chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d");

    let (start, end) = match (start, end) {
        (Ok(s), Ok(e)) => (s, e),
        _ => return vec![],
    };

    let mut all_records: Vec<ReservePriceRecord> = Vec::new();
    let mut date = start;

    while date <= end {
        let date_str = date.to_string();
        let filter = format!(
            "business_date ge '{}' and business_date le '{}'",
            date_str, date_str
        );
        match fetch_pse::<ReservePriceRecord>(client, "cmbp-tp", &filter).await {
            Ok(records) => all_records.extend(records),
            Err(e) => {
                tracing::warn!("PSE reserves daily fetch failed for {}: {}", date_str, e);
            }
        }
        date += chrono::Duration::days(1);
    }

    all_records
}

/// Fetch reserve prices sampled every 7 days to avoid too many API calls.
async fn fetch_reserves_sampled(
    client: &reqwest::Client,
    start_date: &str,
    end_date: &str,
) -> Vec<ReservePriceRecord> {
    let start = chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d");
    let end = chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d");

    let (start, end) = match (start, end) {
        (Ok(s), Ok(e)) => (s, e),
        _ => return vec![],
    };

    let mut all_records: Vec<ReservePriceRecord> = Vec::new();
    let mut date = start;

    while date <= end {
        let date_str = date.to_string();
        let filter = format!(
            "business_date ge '{}' and business_date le '{}'",
            date_str, date_str
        );
        match fetch_pse::<ReservePriceRecord>(client, "cmbp-tp", &filter).await {
            Ok(records) => all_records.extend(records),
            Err(e) => {
                tracing::warn!("PSE reserves fetch failed for {}: {}", date_str, e);
            }
        }
        date += chrono::Duration::days(7);
    }

    all_records
}

/// Build daily average reserve prices from hourly records.
fn build_daily_avg_history(records: &[ReservePriceRecord]) -> Vec<serde_json::Value> {
    let mut days: std::collections::BTreeMap<String, Vec<&ReservePriceRecord>> =
        std::collections::BTreeMap::new();

    for r in records {
        days.entry(r.business_date.clone()).or_default().push(r);
    }

    days.iter()
        .map(|(date, recs)| {
            let avg_field = |f: fn(&ReservePriceRecord) -> Option<f64>| -> f64 {
                let vals: Vec<f64> = recs.iter().filter_map(|r| f(r)).collect();
                if vals.is_empty() {
                    return 0.0;
                }
                round2(vals.iter().sum::<f64>() / vals.len() as f64)
            };
            json!({
                "date": date,
                "afrr_g": avg_field(|r| r.afrr_g),
                "afrr_d": avg_field(|r| r.afrr_d),
                "mfrrd_g": avg_field(|r| r.mfrrd_g),
                "fcr_g": avg_field(|r| r.fcr_g),
            })
        })
        .collect()
}
