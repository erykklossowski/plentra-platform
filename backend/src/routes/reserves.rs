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

    // Today's prices (cmbp-tp has 24 hourly records per day)
    let price_records: Vec<ReservePriceRecord> = fetch_pse(
        &state.http_client,
        "cmbp-tp",
        &format!(
            "business_date ge '{}' and business_date le '{}'",
            today, today
        ),
        100,
    )
    .await
    .unwrap_or_default();

    // 13-month history for trend charts
    let history_records: Vec<ReservePriceRecord> = fetch_pse(
        &state.http_client,
        "cmbp-tp",
        &format!(
            "business_date ge '{}' and business_date le '{}'",
            date_13m_ago, today
        ),
        5000,
    )
    .await
    .unwrap_or_default();

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
