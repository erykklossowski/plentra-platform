use std::sync::Arc;

use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use chrono::Utc;
use serde_json::{json, Value};

use crate::fetchers::pse::{
    aggregate_curtailment_daily, aggregate_to_hourly, estimate_ytd_gwh,
    estimate_ytd_gwh_field, fetch_pse, fetch_pse_date_range, today_warsaw,
    date_days_ago, DailyCurtailment, PozRedozeRecord,
};
use crate::AppState;

use chrono_tz::Europe::Warsaw;

pub async fn handler(State(state): State<Arc<AppState>>) -> (HeaderMap, Json<Value>) {
    let mut headers = HeaderMap::new();
    headers.insert("cache-control", "max-age=600".parse().unwrap());

    // Check fresh cache first
    if let Some(cached) = state.cache.get("pse_curtailment") {
        return (headers, Json(cached.data));
    }

    let today = today_warsaw();
    let date_30d_ago = date_days_ago(30);

    // Fetch today's curtailment (single day — 96 records, fits in one request)
    let today_filter = format!(
        "business_date ge '{}' and business_date le '{}'",
        today, today
    );
    let today_records: Vec<PozRedozeRecord> = fetch_pse(
        &state.http_client,
        "poze-redoze",
        &today_filter,
    )
    .await
    .unwrap_or_default();

    // Fetch 30-day rolling window day-by-day (PSE returns max 100 records per request)
    let rolling_records: Vec<PozRedozeRecord> = fetch_pse_date_range(
        &state.http_client,
        "poze-redoze",
        &date_30d_ago,
        &today,
    )
    .await;

    // Aggregate today
    let today_agg = aggregate_curtailment_daily(&today_records, &today);

    // Aggregate each day in rolling window
    let mut dates: Vec<String> = rolling_records
        .iter()
        .map(|r| r.business_date.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    dates.sort();

    let daily_30d: Vec<DailyCurtailment> = dates
        .iter()
        .map(|date| aggregate_curtailment_daily(&rolling_records, date))
        .collect();

    // Try YTD from DB, fall back to estimation
    let ytd_json = if let Some(pool) = &state.db {
        crate::db::reader::get_curtailment_ytd(pool).await.ok()
    } else {
        None
    };

    let (ytd_total_gwh, ytd_wind_gwh, ytd_solar_gwh, ytd_network_gwh, ytd_balance_gwh, ytd_is_estimate, ytd_source) =
        if let Some(ref ytd) = ytd_json {
            (
                ytd["ytd_total_gwh"].as_f64().unwrap_or(0.0),
                ytd["ytd_wind_gwh"].as_f64().unwrap_or(0.0),
                ytd["ytd_solar_gwh"].as_f64().unwrap_or(0.0),
                ytd["ytd_network_gwh"].as_f64().unwrap_or(0.0),
                ytd["ytd_balance_gwh"].as_f64().unwrap_or(0.0),
                false,
                "PSE POZE-REDOZE via TimescaleDB",
            )
        } else {
            (
                estimate_ytd_gwh(&daily_30d),
                estimate_ytd_gwh_field(&daily_30d, |d| d.wi_balance_mwh + d.wi_network_mwh),
                estimate_ytd_gwh_field(&daily_30d, |d| d.pv_balance_mwh + d.pv_network_mwh),
                estimate_ytd_gwh_field(&daily_30d, |d| d.wi_network_mwh + d.pv_network_mwh),
                estimate_ytd_gwh_field(&daily_30d, |d| d.wi_balance_mwh + d.pv_balance_mwh),
                true,
                "PSE POZE-REDOZE (estimated)",
            )
        };

    // Hourly profile: aggregate 15-min → hourly for today
    let hourly_profile = aggregate_to_hourly(&today_records);

    let result = json!({
        "today_total_mwh": today_agg.total_mwh,
        "today_wind_balance_mwh": today_agg.wi_balance_mwh,
        "today_wind_network_mwh": today_agg.wi_network_mwh,
        "today_pv_balance_mwh": today_agg.pv_balance_mwh,
        "today_pv_network_mwh": today_agg.pv_network_mwh,
        "ytd_total_gwh": ytd_total_gwh,
        "ytd_wind_gwh": ytd_wind_gwh,
        "ytd_solar_gwh": ytd_solar_gwh,
        "ytd_network_gwh": ytd_network_gwh,
        "ytd_balance_gwh": ytd_balance_gwh,
        "hourly_profile": hourly_profile,
        "daily_30d": daily_30d,
        "is_estimate": ytd_is_estimate,
        "source": ytd_source,
        "fetched_at": Utc::now().to_rfc3339(),
    });

    state
        .cache
        .set("pse_curtailment".to_string(), result.clone(), 600);

    // Background: persist to TimescaleDB
    if let Some(pool) = state.db.clone() {
        let records = rolling_records.clone();
        tokio::spawn(async move {
            if let Err(e) = persist_curtailment(&pool, &records).await {
                tracing::warn!("DB write failed for curtailment: {}", e);
            }
        });
    }

    (headers, Json(result))
}

fn parse_pse_dtime_utc(dtime: &str) -> Option<chrono::DateTime<Utc>> {
    chrono::NaiveDateTime::parse_from_str(dtime, "%Y-%m-%d %H:%M:%S")
        .ok()
        .and_then(|ndt| ndt.and_local_timezone(Warsaw).single())
        .map(|dt| dt.with_timezone(&Utc))
}

async fn persist_curtailment(
    pool: &sqlx::PgPool,
    records: &[PozRedozeRecord],
) -> anyhow::Result<()> {
    use crate::db::writer::write_curtailment_batch;

    let batch: Vec<_> = records
        .iter()
        .filter_map(|r| {
            parse_pse_dtime_utc(&r.dtime).map(|ts| {
                (
                    ts,
                    r.wi_red_balance.unwrap_or(0.0),
                    r.wi_red_network.unwrap_or(0.0),
                    r.pv_red_balance.unwrap_or(0.0),
                    r.pv_red_network.unwrap_or(0.0),
                )
            })
        })
        .collect();

    let count = write_curtailment_batch(pool, &batch).await?;
    tracing::debug!("Persisted {} curtailment records to DB", count);
    Ok(())
}
