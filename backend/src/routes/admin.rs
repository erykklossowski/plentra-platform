use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use chrono_tz::Europe::Warsaw;
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;

use crate::fetchers::pse::{fetch_pse, PozRedozeRecord, ReservePriceRecord};
use crate::fetchers::stooq;
use crate::AppState;

#[derive(Deserialize)]
pub struct BackfillParams {
    pub token: String,
    pub source: Option<String>,
    pub days: Option<i64>,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<BackfillParams>,
) -> impl IntoResponse {
    let expected = std::env::var("BACKFILL_TOKEN").unwrap_or_else(|_| "change-me".to_string());
    if params.token != expected {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "invalid token"})),
        )
            .into_response();
    }

    let pool = match &state.db {
        Some(p) => p.clone(),
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({"error": "no database"})),
            )
                .into_response()
        }
    };

    let days = params.days.unwrap_or(30).min(730);
    let source = params.source.as_deref().unwrap_or("stooq");

    let client = state.http_client.clone();

    match source {
        "stooq" => {
            tokio::spawn(async move {
                match backfill_stooq(&client, &pool, days).await {
                    Ok(n) => tracing::info!("Stooq backfill: {} rows written", n),
                    Err(e) => tracing::error!("Stooq backfill failed: {}", e),
                }
            });
        }
        "curtailment" => {
            tokio::spawn(async move {
                match backfill_curtailment(&client, &pool, days).await {
                    Ok(n) => tracing::info!("Curtailment backfill: {} rows written", n),
                    Err(e) => tracing::error!("Curtailment backfill failed: {}", e),
                }
            });
        }
        "reserves" => {
            tokio::spawn(async move {
                match backfill_reserves(&client, &pool, days).await {
                    Ok(n) => tracing::info!("Reserves backfill: {} rows written", n),
                    Err(e) => tracing::error!("Reserves backfill failed: {}", e),
                }
            });
        }
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "unknown source, use: stooq, curtailment, reserves"})),
            )
                .into_response()
        }
    }

    Json(json!({
        "status": "backfill started",
        "source": source,
        "days": days,
        "note": "check Railway logs for progress"
    }))
    .into_response()
}

async fn backfill_stooq(
    client: &reqwest::Client,
    pool: &PgPool,
    days: i64,
) -> anyhow::Result<usize> {
    let tickers = [
        ("tg.f", "TTF", "EUR/MWh"),
        ("ck.f", "EUA", "EUR/t"),
        ("lu.f", "ARA", "USD/t"),
    ];

    let mut total = 0usize;
    for (stooq_sym, ticker, unit) in &tickers {
        tracing::info!("Backfilling {} from Stooq ({} days)...", ticker, days);

        let rows = stooq::fetch_history_csv(client, stooq_sym, days as u64).await?;
        let batch: Vec<_> = rows
            .iter()
            .map(|(ts, close)| (*ts, *ticker, *close, *unit, "STOOQ"))
            .collect();
        total += crate::db::writer::write_fuel_prices_batch(pool, &batch).await?;

        // Rate limit
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    Ok(total)
}

fn parse_pse_dtime_utc(dtime: &str) -> Option<chrono::DateTime<Utc>> {
    chrono::NaiveDateTime::parse_from_str(dtime, "%Y-%m-%d %H:%M:%S")
        .ok()
        .and_then(|ndt| ndt.and_local_timezone(Warsaw).single())
        .map(|dt| dt.with_timezone(&Utc))
}

async fn backfill_curtailment(
    client: &reqwest::Client,
    pool: &PgPool,
    days: i64,
) -> anyhow::Result<usize> {
    use chrono::Duration;
    let mut total = 0usize;

    for day_offset in (0..days).rev() {
        let date = (Utc::now() - Duration::days(day_offset))
            .date_naive()
            .to_string();

        let filter = format!(
            "business_date ge '{}' and business_date le '{}'",
            date, date
        );
        let records: Vec<PozRedozeRecord> =
            fetch_pse(client, "poze-redoze", &filter).await.unwrap_or_default();

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

        total += crate::db::writer::write_curtailment_batch(pool, &batch).await?;

        if day_offset % 10 == 0 {
            tracing::info!("Curtailment backfill: {} days remaining", day_offset);
        }

        // Rate limit: 1 request per second to PSE API
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    }

    Ok(total)
}

async fn backfill_reserves(
    client: &reqwest::Client,
    pool: &PgPool,
    days: i64,
) -> anyhow::Result<usize> {
    use chrono::Duration;
    let mut total = 0usize;

    for day_offset in (0..days).rev() {
        let date = (Utc::now() - Duration::days(day_offset))
            .date_naive()
            .to_string();

        let filter = format!(
            "business_date ge '{}' and business_date le '{}'",
            date, date
        );
        let records: Vec<ReservePriceRecord> =
            fetch_pse(client, "cmbp-tp", &filter).await.unwrap_or_default();

        for r in &records {
            if let Some(ts) = parse_pse_dtime_utc(&r.dtime) {
                crate::db::writer::write_reserve_prices(
                    pool, ts, r.afrr_d, r.afrr_g, r.mfrrd_d, r.mfrrd_g, r.fcr_d, r.fcr_g,
                    r.rr_g,
                )
                .await?;
                total += 1;
            }
        }

        if day_offset % 10 == 0 {
            tracing::info!("Reserves backfill: {} days remaining", day_offset);
        }

        // Rate limit
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    }

    Ok(total)
}
