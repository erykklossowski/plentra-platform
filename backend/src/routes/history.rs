use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::Json;
use chrono::Utc;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::AppState;

#[derive(Deserialize)]
pub struct HistoryParams {
    pub ticker: Option<String>,
    pub source: Option<String>,
    pub product: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub resolution: Option<String>,
}

fn resolution_to_bucket(res: &str) -> &'static str {
    match res {
        "15min" => "15 minutes",
        "hourly" => "1 hour",
        "weekly" => "1 week",
        "monthly" => "1 month",
        _ => "1 day", // default: daily
    }
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

fn headers_cached() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("cache-control", "max-age=3600".parse().unwrap());
    headers
}

fn empty_history(ticker: &str, resolution: &str, from: &str, to: &str) -> Value {
    json!({
        "ticker": ticker,
        "resolution": resolution,
        "from": from,
        "to": to,
        "points": [],
        "point_count": 0,
        "source": "TimescaleDB",
    })
}

// ──────────────────────── /api/history/fuels ────────────────────────

pub async fn fuels_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HistoryParams>,
) -> (HeaderMap, Json<Value>) {
    let ticker = params.ticker.as_deref().unwrap_or("TTF");
    let resolution = params.resolution.as_deref().unwrap_or("daily");
    let bucket = resolution_to_bucket(resolution);
    let from = params.from.as_deref().unwrap_or("2025-01-01");
    let to = params
        .to
        .as_deref()
        .map(|s| s.to_string())
        .unwrap_or_else(|| Utc::now().date_naive().to_string());

    let pool = match &state.db {
        Some(p) => p,
        None => return (headers_cached(), Json(empty_history(ticker, resolution, from, &to))),
    };

    // Query fuel_ohlcv: pick the highest-volume contract per date for front-month price
    let rows = sqlx::query_as::<_, (chrono::NaiveDate, f64)>(
        r#"
        SELECT DISTINCT ON (date)
            date, close
        FROM fuel_ohlcv
        WHERE ticker = $1
          AND date >= $2::date
          AND date <= $3::date
          AND close > 0
          AND close < 1000000
        ORDER BY date ASC, volume DESC
        "#,
    )
    .bind(ticker)
    .bind(from)
    .bind(to.as_str())
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let points: Vec<Value> = rows
        .iter()
        .map(|(date, close)| {
            json!({
                "ts": date.and_hms_opt(17, 30, 0).unwrap().and_utc().to_rfc3339(),
                "avg": round2(*close),
                "min": round2(*close),
                "max": round2(*close),
            })
        })
        .collect();

    let count = points.len();
    (
        headers_cached(),
        Json(json!({
            "ticker": ticker,
            "resolution": resolution,
            "from": from,
            "to": to,
            "points": points,
            "point_count": count,
            "source": "TimescaleDB",
        })),
    )
}

// ──────────────────────── /api/history/spreads ────────────────────────

pub async fn spreads_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HistoryParams>,
) -> (HeaderMap, Json<Value>) {
    let resolution = params.resolution.as_deref().unwrap_or("daily");
    let from = params.from.as_deref().unwrap_or("2025-01-01");
    let to = params
        .to
        .as_deref()
        .map(|s| s.to_string())
        .unwrap_or_else(|| Utc::now().date_naive().to_string());

    let pool = match &state.db {
        Some(p) => p,
        None => return (headers_cached(), Json(empty_history("spreads", resolution, from, &to))),
    };

    // Read pre-computed CSS and CDS from calculated_spreads (forward-looking).
    let rows = sqlx::query_as::<_, (chrono::NaiveDate, String, f64, f64)>(
        r#"
        SELECT date, spread_type, value, carbon_price
        FROM calculated_spreads
        WHERE date >= $1::date AND date <= $2::date
          AND spread_type IN ('rolling_3m_css', 'rolling_3m_cds')
        ORDER BY date ASC
        "#,
    )
    .bind(from)
    .bind(to.as_str())
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    // Pivot CSS + CDS rows into per-date points
    let mut date_map: std::collections::BTreeMap<chrono::NaiveDate, (Option<f64>, Option<f64>, Option<f64>)> =
        std::collections::BTreeMap::new();
    for (date, spread_type, value, carbon_price) in &rows {
        let entry = date_map.entry(*date).or_insert((None, None, None));
        match spread_type.as_str() {
            "rolling_3m_css" => {
                entry.0 = Some(round2(*value));
                // Carbon impact factor derived from forward EUA
                entry.2 = Some(round2(crate::analytics::css::carbon_impact_factor(*carbon_price)));
            }
            "rolling_3m_cds" => entry.1 = Some(round2(*value)),
            _ => {}
        }
    }

    let points: Vec<Value> = date_map
        .iter()
        .filter_map(|(date, (css, cds, _cif))| {
            // Only emit points that have at least CSS
            let css_val = (*css)?;
            Some(json!({
                "ts": date.and_hms_opt(17, 30, 0).unwrap().and_utc().to_rfc3339(),
                "css": css_val,
                "cds_42": cds.unwrap_or(0.0),
            }))
        })
        .collect();

    let count = points.len();
    (
        headers_cached(),
        Json(json!({
            "ticker": "spreads",
            "resolution": resolution,
            "from": from,
            "to": to,
            "points": points,
            "point_count": count,
            "source": "TimescaleDB",
        })),
    )
}

// ──────────────────────── /api/history/curtailment ────────────────────────

pub async fn curtailment_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HistoryParams>,
) -> (HeaderMap, Json<Value>) {
    let resolution = params.resolution.as_deref().unwrap_or("daily");
    let bucket = resolution_to_bucket(resolution);
    let from = params.from.as_deref().unwrap_or("2026-01-01");
    let to = params
        .to
        .as_deref()
        .map(|s| s.to_string())
        .unwrap_or_else(|| Utc::now().date_naive().to_string());

    let pool = match &state.db {
        Some(p) => p,
        None => return (headers_cached(), Json(empty_history("curtailment", resolution, from, &to))),
    };

    let rows = sqlx::query_as::<_, (Option<chrono::DateTime<Utc>>, Option<f64>, Option<f64>, Option<f64>, Option<f64>)>(
        r#"SELECT
               time_bucket($1::interval, ts) AS bucket,
               SUM(wi_balance_mw) * 0.25 AS wi_balance_mwh,
               SUM(wi_network_mw) * 0.25 AS wi_network_mwh,
               SUM(pv_balance_mw) * 0.25 AS pv_balance_mwh,
               SUM(pv_network_mw) * 0.25 AS pv_network_mwh
           FROM curtailment_15min
           WHERE ts >= $2::date AND ts <= $3::date
           GROUP BY bucket
           ORDER BY bucket ASC"#,
    )
    .bind(bucket)
    .bind(from)
    .bind(to.as_str())
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let points: Vec<Value> = rows
        .iter()
        .map(|(ts, wi_b, wi_n, pv_b, pv_n)| {
            json!({
                "ts": ts.map(|t| t.to_rfc3339()),
                "wi_balance_mwh": wi_b.map(round2).unwrap_or(0.0),
                "wi_network_mwh": wi_n.map(round2).unwrap_or(0.0),
                "pv_balance_mwh": pv_b.map(round2).unwrap_or(0.0),
                "pv_network_mwh": pv_n.map(round2).unwrap_or(0.0),
            })
        })
        .collect();

    let count = points.len();
    (
        headers_cached(),
        Json(json!({
            "ticker": "curtailment",
            "resolution": resolution,
            "from": from,
            "to": to,
            "points": points,
            "point_count": count,
            "source": "TimescaleDB",
        })),
    )
}

// ──────────────────────── /api/history/reserves ────────────────────────

pub async fn reserves_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HistoryParams>,
) -> (HeaderMap, Json<Value>) {
    let product = params.product.as_deref().unwrap_or("afrr_g");
    let resolution = params.resolution.as_deref().unwrap_or("daily");
    let bucket = resolution_to_bucket(resolution);
    let from = params.from.as_deref().unwrap_or("2025-01-01");
    let to = params
        .to
        .as_deref()
        .map(|s| s.to_string())
        .unwrap_or_else(|| Utc::now().date_naive().to_string());

    let pool = match &state.db {
        Some(p) => p,
        None => return (headers_cached(), Json(empty_history(product, resolution, from, &to))),
    };

    // Map product name to column — use a safe whitelist
    let column = match product {
        "afrr_d" => "afrr_d_pln_mw",
        "afrr_g" => "afrr_g_pln_mw",
        "mfrrd_d" => "mfrrd_d_pln_mw",
        "mfrrd_g" => "mfrrd_g_pln_mw",
        "fcr_d" => "fcr_d_pln_mw",
        "fcr_g" => "fcr_g_pln_mw",
        "rr_g" => "rr_g_pln_mw",
        _ => "afrr_g_pln_mw",
    };

    // We need to use format! since column names can't be parameterized in SQL
    let query = format!(
        r#"SELECT
               time_bucket($1::interval, ts) AS bucket,
               AVG({col}) AS avg_val,
               MIN({col}) AS min_val,
               MAX({col}) AS max_val
           FROM reserve_prices_hourly
           WHERE ts >= $2::date AND ts <= $3::date
           GROUP BY bucket
           ORDER BY bucket ASC"#,
        col = column
    );

    let rows = sqlx::query_as::<_, (Option<chrono::DateTime<Utc>>, Option<f64>, Option<f64>, Option<f64>)>(
        &query,
    )
    .bind(bucket)
    .bind(from)
    .bind(to.as_str())
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let points: Vec<Value> = rows
        .iter()
        .map(|(ts, avg, min, max)| {
            json!({
                "ts": ts.map(|t| t.to_rfc3339()),
                "avg": avg.map(round2),
                "min": min.map(round2),
                "max": max.map(round2),
            })
        })
        .collect();

    let count = points.len();
    (
        headers_cached(),
        Json(json!({
            "ticker": product,
            "resolution": resolution,
            "from": from,
            "to": to,
            "points": points,
            "point_count": count,
            "source": "TimescaleDB",
        })),
    )
}

// ──────────────────────── /api/history/prices ────────────────────────

pub async fn prices_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HistoryParams>,
) -> (HeaderMap, Json<Value>) {
    let source = params.source.as_deref().unwrap_or("PSE");
    let resolution = params.resolution.as_deref().unwrap_or("daily");
    let bucket = resolution_to_bucket(resolution);
    let from = params.from.as_deref().unwrap_or("2025-01-01");
    let to = params
        .to
        .as_deref()
        .map(|s| s.to_string())
        .unwrap_or_else(|| Utc::now().date_naive().to_string());

    let pool = match &state.db {
        Some(p) => p,
        None => return (headers_cached(), Json(empty_history(source, resolution, from, &to))),
    };

    // PSE source: return three price series (CEN, CKOEB, SDAC)
    if source == "PSE" {
        let rows = sqlx::query_as::<_, (Option<chrono::DateTime<Utc>>, Option<f64>, Option<f64>, Option<f64>)>(
            r#"SELECT
                   time_bucket($1::interval, ts) AS bucket,
                   AVG(cen_pln)   AS avg_cen,
                   AVG(ckoeb_pln) AS avg_ckoeb,
                   AVG(csdac_pln) AS avg_sdac
               FROM price_hourly
               WHERE source = 'PSE'
                 AND product = 'DA'
                 AND ts >= $2::date
                 AND ts <= $3::date
               GROUP BY bucket
               ORDER BY bucket ASC"#,
        )
        .bind(bucket)
        .bind(from)
        .bind(to.as_str())
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        let cen: Vec<Value> = rows.iter().map(|(ts, cen, _, _)| json!({"ts": ts.map(|t| t.to_rfc3339()), "value": cen.map(round2)})).collect();
        let ckoeb: Vec<Value> = rows.iter().map(|(ts, _, ckoeb, _)| json!({"ts": ts.map(|t| t.to_rfc3339()), "value": ckoeb.map(round2)})).collect();
        let sdac: Vec<Value> = rows.iter().map(|(ts, _, _, sdac)| json!({"ts": ts.map(|t| t.to_rfc3339()), "value": sdac.map(round2)})).collect();

        let count = rows.len();
        return (
            headers_cached(),
            Json(json!({
                "ticker": "PSE_DA",
                "resolution": resolution,
                "from": from,
                "to": to,
                "point_count": count,
                "series": {
                    "cen": cen,
                    "ckoeb": ckoeb,
                    "sdac": sdac,
                },
                "source": "PSE api.raporty.pse.pl",
            })),
        );
    }

    // Other sources: legacy EUR-based query
    let rows = sqlx::query_as::<_, (Option<chrono::DateTime<Utc>>, Option<f64>, Option<f64>, Option<f64>)>(
        r#"SELECT
               time_bucket($1::interval, ts) AS bucket,
               AVG(value_eur_mwh) AS avg_val,
               MIN(value_eur_mwh) AS min_val,
               MAX(value_eur_mwh) AS max_val
           FROM price_hourly
           WHERE source = $2
             AND ts >= $3::date
             AND ts <= $4::date
           GROUP BY bucket
           ORDER BY bucket ASC"#,
    )
    .bind(bucket)
    .bind(source)
    .bind(from)
    .bind(to.as_str())
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let points: Vec<Value> = rows
        .iter()
        .map(|(ts, avg, min, max)| {
            json!({
                "ts": ts.map(|t| t.to_rfc3339()),
                "avg": avg.map(round2),
                "min": min.map(round2),
                "max": max.map(round2),
            })
        })
        .collect();

    let count = points.len();
    (
        headers_cached(),
        Json(json!({
            "ticker": source,
            "resolution": resolution,
            "from": from,
            "to": to,
            "points": points,
            "point_count": count,
            "source": "TimescaleDB",
        })),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolution_to_bucket_mapping() {
        assert_eq!(resolution_to_bucket("daily"), "1 day");
        assert_eq!(resolution_to_bucket("hourly"), "1 hour");
        assert_eq!(resolution_to_bucket("monthly"), "1 month");
        assert_eq!(resolution_to_bucket("weekly"), "1 week");
        assert_eq!(resolution_to_bucket("15min"), "15 minutes");
        assert_eq!(resolution_to_bucket("garbage"), "1 day"); // default
    }
}
