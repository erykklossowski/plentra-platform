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
    let source = params.source.as_deref().unwrap_or("databento");

    let client = state.http_client.clone();
    let config = state.config.clone();

    match source {
        "databento" => {
            let api_key = match config.databento_api_key {
                Some(k) => k,
                None => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(json!({"error": "DATABENTO_API_KEY not set"})),
                    )
                        .into_response()
                }
            };
            tokio::spawn(async move {
                match backfill_databento(&api_key, &pool, days).await {
                    Ok(n) => tracing::info!("Databento backfill: {} rows written", n),
                    Err(e) => tracing::error!("Databento backfill failed: {}", e),
                }
            });
            Json(json!({
                "status": "backfill started",
                "source": "databento",
                "days": days,
                "instruments": ["TTF", "EUA", "ARA"],
                "note": "check Railway logs for progress"
            }))
            .into_response()
        }
        "stooq" => {
            Json(json!({
                "status": "deprecated",
                "message": "Stooq removed. Use source=databento."
            }))
            .into_response()
        }
        "curtailment" => {
            tokio::spawn(async move {
                match backfill_curtailment(&client, &pool, days).await {
                    Ok(n) => tracing::info!("Curtailment backfill: {} rows written", n),
                    Err(e) => tracing::error!("Curtailment backfill failed: {}", e),
                }
            });
            Json(json!({
                "status": "backfill started",
                "source": source,
                "days": days,
                "note": "check Railway logs for progress"
            }))
            .into_response()
        }
        "reserves" => {
            tokio::spawn(async move {
                match backfill_reserves(&client, &pool, days).await {
                    Ok(n) => tracing::info!("Reserves backfill: {} rows written", n),
                    Err(e) => tracing::error!("Reserves backfill failed: {}", e),
                }
            });
            Json(json!({
                "status": "backfill started",
                "source": source,
                "days": days,
                "note": "check Railway logs for progress"
            }))
            .into_response()
        }
        "databento_debug" => {
            let api_key = match config.databento_api_key {
                Some(k) => k,
                None => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(json!({"error": "DATABENTO_API_KEY not set"})),
                    )
                        .into_response()
                }
            };
            // Synchronous: fetch all stat_types for each instrument, return in response
            use databento::{
                dbn::{Schema, SType, StatMsg},
                historical::timeseries::GetRangeParams,
                HistoricalClient,
            };
            let symbols = [
                ("TTF", "TFU.FUT"),
                ("EUA_ECF", "ECF.FUT"),
                ("EUA_CKM", "CKM.FUT"),
                ("EUA_CFI", "CFI.FUT"),
                ("ARA", "ATW.FUT"),
            ];
            let mut results = serde_json::Map::new();

            for (name, symbol) in &symbols {
                let client_result = HistoricalClient::builder()
                    .key(api_key.as_str())
                    .and_then(|b| Ok(b.build()?));
                let mut client = match client_result {
                    Ok(c) => c,
                    Err(e) => {
                        results.insert(name.to_string(), json!({"error": format!("{}", e)}));
                        continue;
                    }
                };
                let decoder_result = client
                    .timeseries()
                    .get_range(
                        &GetRangeParams::builder()
                            .dataset("IFEU.IMPACT")
                            .date_time_range(
                                time::macros::datetime!(2026-03-27 00:00 UTC)
                                    ..time::macros::datetime!(2026-03-28 00:00 UTC),
                            )
                            .symbols(vec![*symbol])
                            .stype_in(SType::Parent)
                            .schema(Schema::Statistics)
                            .build(),
                    )
                    .await;
                let mut decoder = match decoder_result {
                    Ok(d) => d,
                    Err(e) => {
                        results.insert(name.to_string(), json!({"error": format!("{}", e)}));
                        continue;
                    }
                };
                // Collect unique (stat_type, price) pairs, skip NaN and 0.0
                let mut by_stat: std::collections::BTreeMap<u16, Vec<f64>> =
                    std::collections::BTreeMap::new();
                while let Ok(Some(msg)) = decoder.decode_record::<StatMsg>().await {
                    let price = msg.price_f64();
                    if price.is_nan() || price == 0.0 {
                        continue;
                    }
                    by_stat.entry(msg.stat_type).or_default().push(price);
                }
                let stat_summary: serde_json::Map<String, serde_json::Value> = by_stat
                    .iter()
                    .map(|(st, prices)| {
                        let first = prices[0];
                        let count = prices.len();
                        (
                            format!("stat_type_{}", st),
                            json!({"price": first, "count": count}),
                        )
                    })
                    .collect();
                results.insert(name.to_string(), json!(stat_summary));
            }

            Json(json!({
                "status": "done",
                "date": "2026-03-27",
                "results": results
            }))
            .into_response()
        }
        "databento_ohlcv" => {
            let api_key = match config.databento_api_key {
                Some(k) => k,
                None => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(json!({"error": "DATABENTO_API_KEY not set"})),
                    )
                        .into_response()
                }
            };
            let today = Utc::now().date_naive();
            let start_date = today - chrono::Duration::days(days);
            match crate::fetchers::databento::fetch_ohlcv(&api_key, start_date, today).await {
                Err(e) => {
                    tracing::error!("OHLCV backfill failed: {}", e);
                    Json(json!({ "status": "error", "error": format!("{}", e) })).into_response()
                }
                Ok(bars) => {
                    let mut written = 0usize;
                    for b in &bars {
                        if crate::db::writer::upsert_fuel_ohlcv(&pool, b).await.is_ok() {
                            written += 1;
                        }
                    }
                    Json(json!({
                        "status": "done",
                        "bars_fetched": bars.len(),
                        "bars_written": written,
                        "days": days
                    })).into_response()
                }
            }
        }
        "recalculate_spreads" => {
            let app_state = state.clone();
            tokio::spawn(async move {
                if let Some(ref pool) = app_state.db {
                    let today = Utc::now().date_naive();
                    let mut date = today - chrono::Duration::days(days);
                    let mut success = 0usize;
                    let mut skipped = 0usize;
                    while date <= today {
                        match crate::analytics::css::run_css(pool, date).await {
                            Ok(_) => success += 1,
                            Err(e) => {
                                // Expected for weekends/holidays/missing data
                                tracing::debug!("CSS skipped {}: {}", date, e);
                                skipped += 1;
                            }
                        }
                        date += chrono::Duration::days(1);
                    }
                    tracing::info!(
                        "Spread recalc: {} calculated, {} skipped",
                        success,
                        skipped
                    );
                }
            });
            Json(json!({ "status": "recalculation started", "days": days })).into_response()
        }
        "ohlcv_status" => {
            // Diagnostic: show what's in fuel_ohlcv table
            let rows = sqlx::query_as::<_, (String, i64, Option<String>, Option<String>)>(
                r#"
                SELECT ticker, COUNT(*) as cnt,
                       MIN(date)::text, MAX(date)::text
                FROM fuel_ohlcv
                GROUP BY ticker ORDER BY ticker
                "#,
            )
            .fetch_all(&pool)
            .await
            .unwrap_or_default();

            let summary: Vec<serde_json::Value> = rows.iter().map(|(ticker, cnt, min_d, max_d)| {
                json!({
                    "ticker": ticker,
                    "rows": cnt,
                    "min_date": min_d,
                    "max_date": max_d,
                })
            }).collect();

            // Sample raw_symbols
            let samples = sqlx::query_as::<_, (String, String, f64)>(
                r#"
                SELECT DISTINCT ON (ticker) ticker, raw_symbol, close
                FROM fuel_ohlcv
                ORDER BY ticker, date DESC
                "#,
            )
            .fetch_all(&pool)
            .await
            .unwrap_or_default();

            let sample_list: Vec<serde_json::Value> = samples.iter().map(|(t, sym, price)| {
                json!({"ticker": t, "raw_symbol": sym, "close": price})
            }).collect();

            Json(json!({
                "status": "done",
                "summary": summary,
                "latest_samples": sample_list,
            }))
            .into_response()
        }
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "unknown source, use: databento, databento_debug, databento_ohlcv, recalculate_spreads, ohlcv_status, curtailment, reserves"})),
            )
                .into_response()
        }
    }
}

async fn backfill_databento(
    api_key: &str,
    pool: &PgPool,
    days: i64,
) -> anyhow::Result<usize> {
    let records = crate::fetchers::databento::fetch_history(api_key, days).await?;

    let mut written = 0usize;
    for (ts, name, price, unit) in &records {
        match crate::db::writer::write_fuel_price(pool, *ts, name, *price, unit, "DATABENTO").await
        {
            Ok(()) => written += 1,
            Err(e) => tracing::warn!("Backfill write failed for {} at {}: {}", name, ts, e),
        }
    }
    tracing::info!("Backfill: {}/{} rows written", written, records.len());
    Ok(written)
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

#[derive(Deserialize)]
pub struct RefreshParams {
    pub token: String,
    pub route: Option<String>,
}

pub async fn refresh_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<RefreshParams>,
) -> impl IntoResponse {
    let expected = std::env::var("BACKFILL_TOKEN").unwrap_or_else(|_| "change-me".to_string());
    if params.token != expected {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "invalid token"})),
        )
            .into_response();
    }

    match params.route.as_deref() {
        Some("all") | None => {
            state.cache.clear();
            Json(json!({"status": "cache invalidated", "route": "all"})).into_response()
        }
        Some(route) => {
            state.cache.invalidate(route);
            Json(json!({"status": "cache invalidated", "route": route})).into_response()
        }
    }
}
