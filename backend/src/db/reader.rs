use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct FuelHistory {
    pub ts: DateTime<Utc>,
    pub ticker: String,
    pub close: f64,
    pub unit: String,
}

/// Last N days of daily close prices for a ticker.
pub async fn get_fuel_history(
    pool: &PgPool,
    ticker: &str,
    days: i64,
) -> anyhow::Result<Vec<FuelHistory>> {
    let rows = sqlx::query_as::<_, FuelHistory>(
        "SELECT ts, ticker, close, unit
         FROM fuel_daily
         WHERE ticker = $1
           AND ts >= NOW() - make_interval(days => $2)
         ORDER BY ts ASC",
    )
    .bind(ticker)
    .bind(days as i32)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Latest fuel price for a ticker (most recent row).
pub async fn get_latest_fuel_price(
    pool: &PgPool,
    ticker: &str,
) -> anyhow::Result<Option<f64>> {
    let row: Option<(f64,)> = sqlx::query_as(
        "SELECT close FROM fuel_daily WHERE ticker = $1 ORDER BY ts DESC LIMIT 1",
    )
    .bind(ticker)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(c,)| c))
}

/// Most recent N days of fuel close prices for sparkline rendering.
/// Reads from fuel_ohlcv, picking the highest-volume contract per date.
pub async fn get_fuel_sparkline(
    pool: &PgPool,
    ticker: &str,
    days: i64,
) -> anyhow::Result<Vec<f64>> {
    let rows: Vec<(f64,)> = sqlx::query_as(
        r#"
        SELECT close FROM (
            SELECT DISTINCT ON (date) date, close
            FROM fuel_ohlcv
            WHERE ticker = $1
              AND date >= CURRENT_DATE - make_interval(days => $2)
              AND close > 0 AND close < 1000000
            ORDER BY date ASC, volume DESC
        ) sub
        ORDER BY date ASC
        "#,
    )
    .bind(ticker)
    .bind(days as i32)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|(c,)| c).collect())
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PriceMonthly {
    pub month: DateTime<Utc>,
    pub source: String,
    pub product: String,
    pub avg_eur: Option<f64>,
    pub avg_pln: Option<f64>,
    pub sample_count: Option<i64>,
}

/// Monthly average prices for the last N months.
pub async fn get_price_monthly(
    pool: &PgPool,
    source: &str,
    product: &str,
    months: i64,
) -> anyhow::Result<Vec<PriceMonthly>> {
    let rows = sqlx::query_as::<_, PriceMonthly>(
        "SELECT month, source, product,
                avg_eur, avg_pln, sample_count
         FROM price_monthly
         WHERE source = $1
           AND product = $2
           AND month >= NOW() - make_interval(months => $3)
         ORDER BY month ASC",
    )
    .bind(source)
    .bind(product)
    .bind(months as i32)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CurtailmentDaily {
    pub day: DateTime<Utc>,
    pub wi_balance_mwh: Option<f64>,
    pub wi_network_mwh: Option<f64>,
    pub pv_balance_mwh: Option<f64>,
    pub pv_network_mwh: Option<f64>,
    pub total_mwh: Option<f64>,
}

/// Daily curtailment for the last N days.
pub async fn get_curtailment_daily(
    pool: &PgPool,
    days: i64,
) -> anyhow::Result<Vec<CurtailmentDaily>> {
    let rows = sqlx::query_as::<_, CurtailmentDaily>(
        "SELECT day, wi_balance_mwh, wi_network_mwh,
                pv_balance_mwh, pv_network_mwh, total_mwh
         FROM curtailment_daily
         WHERE day >= NOW() - make_interval(days => $1)
         ORDER BY day ASC",
    )
    .bind(days as i32)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

type YtdRow = (Option<f64>, Option<f64>, Option<f64>, Option<f64>, Option<f64>);

/// YTD curtailment totals, by cause.
pub async fn get_curtailment_ytd(pool: &PgPool) -> anyhow::Result<serde_json::Value> {
    let row: YtdRow = sqlx::query_as(
        "SELECT
             COALESCE(SUM(wi_balance_mwh), 0),
             COALESCE(SUM(wi_network_mwh), 0),
             COALESCE(SUM(pv_balance_mwh), 0),
             COALESCE(SUM(pv_network_mwh), 0),
             COALESCE(SUM(total_mwh),      0)
         FROM curtailment_daily
         WHERE day >= date_trunc('year', NOW())",
    )
    .fetch_one(pool)
    .await?;

    let wi_bal = row.0.unwrap_or(0.0);
    let wi_net = row.1.unwrap_or(0.0);
    let pv_bal = row.2.unwrap_or(0.0);
    let pv_net = row.3.unwrap_or(0.0);
    let total = row.4.unwrap_or(0.0);

    Ok(serde_json::json!({
        "ytd_wi_balance_gwh":  wi_bal / 1000.0,
        "ytd_wi_network_gwh":  wi_net / 1000.0,
        "ytd_pv_balance_gwh":  pv_bal / 1000.0,
        "ytd_pv_network_gwh":  pv_net / 1000.0,
        "ytd_total_gwh":       total / 1000.0,
        "ytd_wind_gwh":        (wi_bal + wi_net) / 1000.0,
        "ytd_solar_gwh":       (pv_bal + pv_net) / 1000.0,
        "ytd_network_gwh":     (wi_net + pv_net) / 1000.0,
        "ytd_balance_gwh":     (wi_bal + pv_bal) / 1000.0,
        "is_estimate":         false,
        "source":              "PSE POZE-REDOZE via TimescaleDB",
    }))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ReserveMonthly {
    pub month: DateTime<Utc>,
    pub afrr_d: Option<f64>,
    pub afrr_g: Option<f64>,
    pub mfrrd_d: Option<f64>,
    pub mfrrd_g: Option<f64>,
    pub fcr_d: Option<f64>,
    pub fcr_g: Option<f64>,
    pub rr_g: Option<f64>,
}

/// Monthly average reserve prices for the last N months.
pub async fn get_reserve_prices_monthly(
    pool: &PgPool,
    months: i64,
) -> anyhow::Result<Vec<ReserveMonthly>> {
    let rows = sqlx::query_as::<_, ReserveMonthly>(
        "SELECT month, afrr_d, afrr_g, mfrrd_d, mfrrd_g, fcr_d, fcr_g, rr_g
         FROM reserve_prices_monthly
         WHERE month >= NOW() - make_interval(months => $1)
         ORDER BY month ASC",
    )
    .bind(months as i32)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Latest recorded timestamp for a given data source.
/// Used to determine whether backfill is needed.
pub async fn get_latest_ts(
    pool: &PgPool,
    table: &str,
    filter_col: &str,
    filter_val: &str,
) -> anyhow::Result<Option<DateTime<Utc>>> {
    // table and filter_col are internal constants, not user input
    let query = format!("SELECT MAX(ts) FROM {} WHERE {} = $1", table, filter_col);
    let ts: (Option<DateTime<Utc>>,) = sqlx::query_as(&query)
        .bind(filter_val)
        .fetch_one(pool)
        .await?;
    Ok(ts.0)
}

/// Read a cached API response from the persistent api_cache table.
pub async fn get_cached_response(
    pool: &PgPool,
    key: &str,
) -> anyhow::Result<Option<serde_json::Value>> {
    let row: Option<(serde_json::Value,)> = sqlx::query_as(
        "SELECT data FROM api_cache WHERE key = $1",
    )
    .bind(key)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(data,)| data))
}
