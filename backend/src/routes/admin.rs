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
            // Use symbology resolve to discover correct symbols, then try fetching stats
            let candidates: Vec<(&str, &str, &str)> = vec![
                // TTF Natural Gas candidates
                ("IFEU.IMPACT", "TFM.FUT", "TTF?"),
                ("IFEU.IMPACT", "TTF.FUT", "TTF?"),
                ("IFEU.IMPACT", "TFU.FUT", "TTF?"),
                ("IFEU.IMPACT", "TF.FUT", "TTF?"),
                ("IFEU.IMPACT", "TTFM.FUT", "TTF?"),
                ("NDEX.IMPACT", "TFM.FUT", "TTF?"),
                ("NDEX.IMPACT", "TTF.FUT", "TTF?"),
                // EUA Carbon candidates
                ("IFEU.IMPACT", "ECF.FUT", "EUA?"),
                ("IFEU.IMPACT", "CKM.FUT", "EUA?"),
                ("IFEU.IMPACT", "EUA.FUT", "EUA?"),
                ("IFEU.IMPACT", "CFI.FUT", "EUA?"),
                ("NDEX.IMPACT", "ECF.FUT", "EUA?"),
                // ARA Coal candidates
                ("IFEU.IMPACT", "ATW.FUT", "ARA?"),
                ("IFEU.IMPACT", "MTF.FUT", "ARA?"),
            ];
            let resp_candidates: Vec<String> = candidates.iter().map(|(d,s,l)| format!("{} {}/{}", l, d, s)).collect();
            tokio::spawn(async move {
                use databento::{
                    dbn::SType,
                    historical::symbology::ResolveParams,
                    HistoricalClient,
                };
                // Group candidates by dataset
                for dataset in &["IFEU.IMPACT", "NDEX.IMPACT"] {
                    let syms: Vec<&str> = candidates.iter()
                        .filter(|(d, _, _)| d == dataset)
                        .map(|(_, s, _)| *s)
                        .collect();
                    if syms.is_empty() { continue; }

                    tracing::info!("DATABENTO_DEBUG: resolving {} symbols on {}", syms.len(), dataset);

                    let client_result = HistoricalClient::builder()
                        .key(api_key.as_str())
                        .and_then(|b| Ok(b.build()?));
                    let mut client = match client_result {
                        Ok(c) => c,
                        Err(e) => {
                            tracing::error!("DATABENTO_DEBUG: client build failed: {}", e);
                            continue;
                        }
                    };

                    let sym_strings: Vec<String> = syms.iter().map(|s| s.to_string()).collect();
                    let resolve_result = client.symbology().resolve(
                        &ResolveParams::builder()
                            .dataset(dataset.to_string())
                            .symbols(sym_strings)
                            .stype_in(SType::Parent)
                            .stype_out(SType::InstrumentId)
                            .date_range(
                                time::macros::date!(2026-03-27)
                                    ..time::macros::date!(2026-03-28)
                            )
                            .build(),
                    ).await;

                    match resolve_result {
                        Ok(resolution) => {
                            tracing::info!(
                                "DATABENTO_DEBUG: resolve OK on {} — result: {:?}",
                                dataset, resolution
                            );
                        }
                        Err(e) => {
                            tracing::warn!(
                                "DATABENTO_DEBUG: resolve failed on {}: {}",
                                dataset, e
                            );
                        }
                    }
                }

                // Also try fetching stats for each candidate individually
                for (dataset, symbol, label) in &candidates {
                    tracing::info!("DATABENTO_DEBUG: trying stats {} {} / {}", label, dataset, symbol);
                    match crate::fetchers::databento::debug_print_stats(&api_key, dataset, symbol).await {
                        Ok(()) => tracing::info!("DATABENTO_DEBUG: {} {} / {} — OK", label, dataset, symbol),
                        Err(e) => tracing::warn!("DATABENTO_DEBUG: {} {} / {} — {}", label, dataset, symbol, e),
                    }
                }
                tracing::info!("DATABENTO_DEBUG: all candidates tested — check logs above");
            });
            Json(json!({
                "status": "debug started",
                "note": "check Railway logs for symbol resolution results",
                "candidates": resp_candidates
            }))
            .into_response()
        }
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "unknown source, use: databento, databento_debug, curtailment, reserves"})),
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
