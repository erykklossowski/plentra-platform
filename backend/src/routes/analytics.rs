use std::sync::Arc;

use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use chrono::Utc;
use serde_json::{json, Value};

use crate::AppState;

fn headers_cached() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("cache-control", "max-age=3600".parse().unwrap());
    headers
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

// ──────────────────────── /api/analytics/spreads ────────────────────────
// CSS/CDS analytics: 90-day history with rolling stats, monthly seasonality, positive days.

pub async fn get_spread_analytics(
    State(state): State<Arc<AppState>>,
) -> (HeaderMap, Json<Value>) {
    let pool = match &state.db {
        Some(p) => p,
        None => {
            return (
                headers_cached(),
                Json(json!({"error": "db not connected"})),
            )
        }
    };

    let (history, seasonality, positive_days) = tokio::join!(
        fetch_spread_history(pool),
        fetch_spread_seasonality(pool),
        fetch_spread_positive_days(pool),
    );

    (
        headers_cached(),
        Json(json!({
            "generated_at": Utc::now().to_rfc3339(),
            "history_90d":   history.unwrap_or_default(),
            "seasonality":   seasonality.unwrap_or_default(),
            "positive_days": positive_days.unwrap_or_default(),
        })),
    )
}

async fn fetch_spread_history(pool: &sqlx::PgPool) -> anyhow::Result<Vec<Value>> {
    let rows = sqlx::query_as::<_, (
        chrono::NaiveDate,
        String,
        f64,
        Option<f64>,
        Option<f64>,
        Option<f64>,
    )>(
        r#"SELECT
               date,
               spread_type,
               value,
               AVG(value) OVER (
                   PARTITION BY spread_type
                   ORDER BY date
                   ROWS BETWEEN 6 PRECEDING AND CURRENT ROW
               ) AS rolling_7d_avg,
               AVG(value) OVER (
                   PARTITION BY spread_type
                   ORDER BY date
                   ROWS BETWEEN 29 PRECEDING AND CURRENT ROW
               ) AS rolling_30d_avg,
               STDDEV(value) OVER (
                   PARTITION BY spread_type
                   ORDER BY date
                   ROWS BETWEEN 29 PRECEDING AND CURRENT ROW
               ) AS rolling_30d_stddev
           FROM calculated_spreads
           WHERE date >= CURRENT_DATE - INTERVAL '90 days'
           ORDER BY date ASC, spread_type ASC"#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .iter()
        .map(|(date, spread_type, value, avg7, avg30, std30)| {
            json!({
                "date":              date.to_string(),
                "spread_type":       spread_type,
                "value":             round2(*value),
                "rolling_7d_avg":    avg7.map(round2),
                "rolling_30d_avg":   avg30.map(round2),
                "rolling_30d_stddev": std30.map(round2),
            })
        })
        .collect())
}

async fn fetch_spread_seasonality(pool: &sqlx::PgPool) -> anyhow::Result<Vec<Value>> {
    let rows = sqlx::query_as::<_, (
        String,
        chrono::NaiveDate,
        Option<f64>,
        Option<f64>,
        Option<f64>,
        Option<f64>,
        Option<f64>,
        Option<f64>,
        i64,
    )>(
        r#"SELECT
               spread_type,
               DATE_TRUNC('month', date)::date AS month,
               MIN(value)                                                AS min_val,
               PERCENTILE_CONT(0.25) WITHIN GROUP (ORDER BY value)      AS q1,
               PERCENTILE_CONT(0.50) WITHIN GROUP (ORDER BY value)      AS median,
               PERCENTILE_CONT(0.75) WITHIN GROUP (ORDER BY value)      AS q3,
               MAX(value)                                                AS max_val,
               AVG(value)                                                AS mean_val,
               COUNT(*)                                                  AS n_days
           FROM calculated_spreads
           WHERE date >= CURRENT_DATE - INTERVAL '365 days'
           GROUP BY spread_type, DATE_TRUNC('month', date)
           ORDER BY spread_type, month ASC"#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .iter()
        .map(|(spread_type, month, min_val, q1, median, q3, max_val, mean_val, n_days)| {
            json!({
                "spread_type": spread_type,
                "month":       month.to_string(),
                "min":         min_val.map(round2),
                "q1":          q1.map(round2),
                "median":      median.map(round2),
                "q3":          q3.map(round2),
                "max":         max_val.map(round2),
                "mean":        mean_val.map(round2),
                "n_days":      n_days,
            })
        })
        .collect())
}

async fn fetch_spread_positive_days(pool: &sqlx::PgPool) -> anyhow::Result<Vec<Value>> {
    let rows = sqlx::query_as::<_, (chrono::NaiveDate, String, i64, i64)>(
        r#"SELECT
               DATE_TRUNC('month', date)::date AS month,
               spread_type,
               COUNT(*) FILTER (WHERE value > 0) AS positive_days,
               COUNT(*)                           AS total_days
           FROM calculated_spreads
           WHERE date >= CURRENT_DATE - INTERVAL '365 days'
           GROUP BY DATE_TRUNC('month', date), spread_type
           ORDER BY month ASC, spread_type"#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .iter()
        .map(|(month, spread_type, positive, total)| {
            let pct = if *total > 0 {
                round2(100.0 * *positive as f64 / *total as f64)
            } else {
                0.0
            };
            json!({
                "month":         month.to_string(),
                "spread_type":   spread_type,
                "positive_days": positive,
                "total_days":    total,
                "positive_pct":  pct,
            })
        })
        .collect())
}

// ──────────────────────── /api/analytics/evening ────────────────────────
// DA evening peak (17-21h CET) decomposition into 4 components.
//
// Tunable constants:
const EUR_PLN_RATE: f64 = 4.27;
const PASS_THROUGH: f64 = 0.65;
const OZE_SCALE_PLN_MWH: f64 = 15.0;

pub async fn get_evening_decomposition(
    State(state): State<Arc<AppState>>,
) -> (HeaderMap, Json<Value>) {
    let pool = match &state.db {
        Some(p) => p,
        None => {
            return (
                headers_cached(),
                Json(json!({"error": "db not connected"})),
            )
        }
    };

    let rows = sqlx::query_as::<_, (
        chrono::NaiveDate,  // date
        Option<f64>,        // evening_avg_pln
        Option<f64>,        // baseline_pln
        Option<f64>,        // css_value (EUR)
        Option<f64>,        // oze_mw midday
        Option<f64>,        // oze_30d_avg
    )>(
        r#"WITH evening_cen AS (
               SELECT
                   ts::date                    AS date,
                   AVG(cen_pln)                AS evening_avg_pln,
                   COUNT(*)                    AS hour_count
               FROM price_hourly
               WHERE source   = 'PSE'
                 AND product  = 'DA'
                 AND cen_pln IS NOT NULL
                 AND EXTRACT(HOUR FROM ts) BETWEEN 15 AND 19
               GROUP BY ts::date
               HAVING COUNT(*) >= 4
           ),
           baseline AS (
               SELECT
                   date,
                   evening_avg_pln,
                   AVG(evening_avg_pln) OVER (
                       ORDER BY date
                       ROWS BETWEEN 7 PRECEDING AND 1 PRECEDING
                   ) AS baseline_pln
               FROM evening_cen
           ),
           oze_midday AS (
               SELECT
                   ts::date AS date,
                   SUM(value_mw) FILTER (WHERE source_type IN ('WIND','PV')) AS oze_mw
               FROM generation_hourly
               WHERE EXTRACT(HOUR FROM ts) BETWEEN 11 AND 14
               GROUP BY ts::date
           )
           SELECT
               b.date,
               b.evening_avg_pln,
               b.baseline_pln,
               cs.value,
               o.oze_mw,
               AVG(o.oze_mw) OVER (ORDER BY b.date ROWS BETWEEN 29 PRECEDING AND CURRENT ROW)
                   AS oze_30d_avg
           FROM baseline b
           LEFT JOIN calculated_spreads cs
               ON cs.date = b.date AND cs.spread_type = 'rolling_3m_css'
           LEFT JOIN oze_midday o
               ON o.date = b.date
           WHERE b.date >= CURRENT_DATE - INTERVAL '90 days'
           ORDER BY b.date ASC"#,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    // Compute decomposition for each day
    let mut decomposition: Vec<Value> = Vec::with_capacity(rows.len());
    let mut css_contrib_sum = 0.0f64;
    let mut css_contrib_count = 0usize;

    for (date, evening_avg, baseline, css_eur, oze_mw, oze_30d_avg) in &rows {
        let evening = evening_avg.unwrap_or(0.0);
        let base = baseline.unwrap_or(evening);

        // delta_fuel: CSS contribution (EUR → PLN, apply pass-through)
        // Floor at zero: fuel costs always add to price, never subtract.
        // When CSS is negative, CCGT is uncompetitive — effect goes into residual.
        let delta_fuel = css_eur
            .map(|css| (css * EUR_PLN_RATE * PASS_THROUGH).max(0.0))
            .unwrap_or(0.0);

        // delta_oze: normalized duck curve × scale factor
        let delta_oze = match (oze_mw, oze_30d_avg) {
            (Some(mw), Some(avg)) if *avg > 0.0 => (mw / avg) * OZE_SCALE_PLN_MWH,
            _ => 0.0,
        };

        let residual = evening - base - delta_fuel - delta_oze;

        // Track CSS contribution % for summary
        if base.abs() > 1.0 {
            css_contrib_sum += (delta_fuel / base) * 100.0;
            css_contrib_count += 1;
        }

        decomposition.push(json!({
            "date":            date.to_string(),
            "evening_avg_pln": round2(evening),
            "baseline_pln":    round2(base),
            "delta_fuel_pln":  round2(delta_fuel),
            "delta_oze_pln":   round2(delta_oze),
            "residual_pln":    round2(residual),
        }));
    }

    let avg_css_pct = if css_contrib_count > 0 {
        round2(css_contrib_sum / css_contrib_count as f64)
    } else {
        0.0
    };

    (
        headers_cached(),
        Json(json!({
            "generated_at": Utc::now().to_rfc3339(),
            "days":         decomposition.len(),
            "constants": {
                "eur_pln_rate":      EUR_PLN_RATE,
                "pass_through":      PASS_THROUGH,
                "oze_scale_pln_mwh": OZE_SCALE_PLN_MWH,
            },
            "summary": {
                "avg_css_contribution_pct": avg_css_pct,
            },
            "decomposition": decomposition,
        })),
    )
}
