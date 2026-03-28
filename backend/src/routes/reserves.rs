use std::sync::Arc;

use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use chrono::Utc;
use serde_json::{json, Value};

use crate::fetchers::pse::{
    build_monthly_avg_history, daily_avg_reserve_price, fetch_pse,
    thirteen_months_ago, today_warsaw, ReservePriceRecord,
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

    // 13-month history — fetch day-by-day (PSE max 100 records/request, 24/day for cmbp-tp)
    // To avoid ~400 API calls, sample weekly (every 7th day)
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
        "history_13m": history_13m,
        "source": "PSE CMBP-TP",
        "fetched_at": Utc::now().to_rfc3339(),
    });

    state
        .cache
        .set("pse_reserves".to_string(), result.clone(), 3600);

    (headers, Json(result))
}

/// Fetch reserve prices sampled every 7 days to avoid too many API calls.
/// For 13 months (~395 days), this makes ~57 requests instead of 395.
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
        date += chrono::Duration::days(7); // sample weekly
    }

    all_records
}
