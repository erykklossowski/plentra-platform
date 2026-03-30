use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;

use super::models::FuelOhlcv;

/// Write hourly electricity price.
pub async fn write_price_hourly(
    pool: &PgPool,
    ts: DateTime<Utc>,
    source: &str,
    product: &str,
    value_eur: Option<f64>,
    value_pln: Option<f64>,
    is_forecast: bool,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO price_hourly
         (ts, source, product, value_eur_mwh, value_pln_mwh, is_forecast)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT DO NOTHING",
    )
    .bind(ts)
    .bind(source)
    .bind(product)
    .bind(value_eur)
    .bind(value_pln)
    .bind(is_forecast)
    .execute(pool)
    .await?;
    Ok(())
}

/// Write 15-min curtailment record.
pub async fn write_curtailment(
    pool: &PgPool,
    ts: DateTime<Utc>,
    wi_balance: f64,
    wi_network: f64,
    pv_balance: f64,
    pv_network: f64,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO curtailment_15min
         (ts, wi_balance_mw, wi_network_mw, pv_balance_mw, pv_network_mw)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT DO NOTHING",
    )
    .bind(ts)
    .bind(wi_balance)
    .bind(wi_network)
    .bind(pv_balance)
    .bind(pv_network)
    .execute(pool)
    .await?;
    Ok(())
}

/// Write a batch of curtailment records in a single transaction.
pub async fn write_curtailment_batch(
    pool: &PgPool,
    rows: &[(DateTime<Utc>, f64, f64, f64, f64)],
) -> anyhow::Result<usize> {
    let mut tx = pool.begin().await?;
    let mut count = 0usize;
    for (ts, wib, win, pvb, pvn) in rows {
        let result = sqlx::query(
            "INSERT INTO curtailment_15min
             (ts, wi_balance_mw, wi_network_mw, pv_balance_mw, pv_network_mw)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT DO NOTHING",
        )
        .bind(ts)
        .bind(wib)
        .bind(win)
        .bind(pvb)
        .bind(pvn)
        .execute(&mut *tx)
        .await?;
        count += result.rows_affected() as usize;
    }
    tx.commit().await?;
    Ok(count)
}

/// Write hourly reserve capacity prices.
#[allow(clippy::too_many_arguments)]
pub async fn write_reserve_prices(
    pool: &PgPool,
    ts: DateTime<Utc>,
    afrr_d: Option<f64>,
    afrr_g: Option<f64>,
    mfrrd_d: Option<f64>,
    mfrrd_g: Option<f64>,
    fcr_d: Option<f64>,
    fcr_g: Option<f64>,
    rr_g: Option<f64>,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO reserve_prices_hourly
         (ts, afrr_d_pln_mw, afrr_g_pln_mw, mfrrd_d_pln_mw,
          mfrrd_g_pln_mw, fcr_d_pln_mw, fcr_g_pln_mw, rr_g_pln_mw)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         ON CONFLICT DO NOTHING",
    )
    .bind(ts)
    .bind(afrr_d)
    .bind(afrr_g)
    .bind(mfrrd_d)
    .bind(mfrrd_g)
    .bind(fcr_d)
    .bind(fcr_g)
    .bind(rr_g)
    .execute(pool)
    .await?;
    Ok(())
}

/// Upsert one hour of PSE electricity prices (CEN, CKOEB, SDAC).
/// Uses source='PSE', product='DA' as the unique key alongside ts.
/// Also populates value_pln_mwh with CEN for backward compat with price_monthly aggregate.
pub async fn upsert_pse_hourly_price(
    pool: &PgPool,
    r: &crate::fetchers::pse::PseHourlyPrice,
) -> anyhow::Result<()> {
    sqlx::query(
        r#"INSERT INTO price_hourly
            (ts, source, product, value_pln_mwh, cen_pln, ckoeb_pln, csdac_pln, is_forecast)
        VALUES ($1, 'PSE', 'DA', $2, $2, $3, $4, false)
        ON CONFLICT (ts, source, product) DO UPDATE SET
            value_pln_mwh = COALESCE(EXCLUDED.value_pln_mwh, price_hourly.value_pln_mwh),
            cen_pln       = COALESCE(EXCLUDED.cen_pln,       price_hourly.cen_pln),
            ckoeb_pln     = COALESCE(EXCLUDED.ckoeb_pln,     price_hourly.ckoeb_pln),
            csdac_pln     = COALESCE(EXCLUDED.csdac_pln,     price_hourly.csdac_pln)
        "#,
    )
    .bind(r.ts)
    .bind(r.cen_pln)
    .bind(r.ckoeb_pln)
    .bind(r.csdac_pln)
    .execute(pool)
    .await?;
    Ok(())
}

/// Persist an API response to the api_cache table (upsert).
pub async fn write_cached_response(
    pool: &PgPool,
    key: &str,
    data: &serde_json::Value,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO api_cache (key, data, updated_at)
         VALUES ($1, $2, NOW())
         ON CONFLICT (key) DO UPDATE SET data = $2, updated_at = NOW()",
    )
    .bind(key)
    .bind(data)
    .execute(pool)
    .await?;
    Ok(())
}

/// Write hourly generation by source type.
pub async fn write_generation(
    pool: &PgPool,
    ts: DateTime<Utc>,
    source_type: &str,
    value_mw: f64,
    is_forecast: bool,
    data_source: &str,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO generation_hourly
         (ts, source_type, value_mw, is_forecast, data_source)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (ts, source_type) DO UPDATE
             SET value_mw = EXCLUDED.value_mw,
                 data_source = EXCLUDED.data_source",
    )
    .bind(ts)
    .bind(source_type)
    .bind(value_mw)
    .bind(is_forecast)
    .bind(data_source)
    .execute(pool)
    .await?;
    Ok(())
}

/// Write a batch of generation records in a single transaction.
/// Each tuple: (ts, source_type, value_mw, is_forecast, data_source).
pub async fn write_generation_batch(
    pool: &PgPool,
    rows: &[(DateTime<Utc>, &str, f64, bool, &str)],
) -> anyhow::Result<usize> {
    let mut tx = pool.begin().await?;
    let mut count = 0usize;
    for (ts, source_type, value_mw, is_forecast, data_source) in rows {
        let result = sqlx::query(
            "INSERT INTO generation_hourly
             (ts, source_type, value_mw, is_forecast, data_source)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (ts, source_type) DO UPDATE
                 SET value_mw = EXCLUDED.value_mw,
                     data_source = EXCLUDED.data_source",
        )
        .bind(ts)
        .bind(source_type)
        .bind(value_mw)
        .bind(is_forecast)
        .bind(data_source)
        .execute(&mut *tx)
        .await?;
        count += result.rows_affected() as usize;
    }
    tx.commit().await?;
    Ok(count)
}

/// Upsert a single OHLCV bar into fuel_ohlcv.
pub async fn upsert_fuel_ohlcv(pool: &PgPool, r: &FuelOhlcv) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO fuel_ohlcv
            (date, instrument_id, dataset, ticker, raw_symbol, unit,
             open, high, low, close, volume)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
        ON CONFLICT (date, instrument_id, dataset) DO UPDATE SET
            raw_symbol = EXCLUDED.raw_symbol,
            open       = EXCLUDED.open,
            high       = EXCLUDED.high,
            low        = EXCLUDED.low,
            close      = EXCLUDED.close,
            volume     = EXCLUDED.volume
        "#,
    )
    .bind(r.date)
    .bind(r.instrument_id)
    .bind(&r.dataset)
    .bind(&r.ticker)
    .bind(&r.raw_symbol)
    .bind(&r.unit)
    .bind(r.open)
    .bind(r.high)
    .bind(r.low)
    .bind(r.close)
    .bind(r.volume)
    .execute(pool)
    .await?;
    Ok(())
}

/// Upsert a calculated spread (e.g. rolling 3-month CSS).
#[allow(clippy::too_many_arguments)]
pub async fn upsert_spread(
    pool: &PgPool,
    date: NaiveDate,
    spread_type: &str,
    value: f64,
    power_avg: f64,
    gas_avg: f64,
    carbon_price: f64,
    power_symbols: &[String],
    gas_symbols: &[String],
    carbon_symbol: &str,
) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO calculated_spreads
            (date, spread_type, value,
             power_avg, gas_avg, carbon_price,
             power_symbols, gas_symbols, carbon_symbol)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
        ON CONFLICT (date, spread_type) DO UPDATE SET
            value         = EXCLUDED.value,
            power_avg     = EXCLUDED.power_avg,
            gas_avg       = EXCLUDED.gas_avg,
            carbon_price  = EXCLUDED.carbon_price,
            power_symbols = EXCLUDED.power_symbols,
            gas_symbols   = EXCLUDED.gas_symbols,
            carbon_symbol = EXCLUDED.carbon_symbol,
            calculated_at = NOW()
        "#,
    )
    .bind(date)
    .bind(spread_type)
    .bind(value)
    .bind(power_avg)
    .bind(gas_avg)
    .bind(carbon_price)
    .bind(power_symbols)
    .bind(gas_symbols)
    .bind(carbon_symbol)
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::Timelike;

    #[test]
    fn test_pse_timestamp_parsing() {
        let raw = "2024-09-24 00:15:00";
        let parsed = chrono::NaiveDateTime::parse_from_str(raw, "%Y-%m-%d %H:%M:%S");
        assert!(parsed.is_ok());
        let ts = parsed.unwrap().and_utc();
        assert_eq!(ts.hour(), 0);
        assert_eq!(ts.minute(), 15);
    }

    #[test]
    fn test_pse_timestamp_with_timezone() {
        use chrono_tz::Europe::Warsaw;

        let raw = "2024-06-15 14:30:00";
        let ndt = chrono::NaiveDateTime::parse_from_str(raw, "%Y-%m-%d %H:%M:%S").unwrap();
        let warsaw_dt = ndt.and_local_timezone(Warsaw).single().unwrap();
        let utc_dt = warsaw_dt.with_timezone(&chrono::Utc);
        // June = CEST (UTC+2), so 14:30 Warsaw = 12:30 UTC
        assert_eq!(utc_dt.hour(), 12);
        assert_eq!(utc_dt.minute(), 30);
    }

    #[test]
    fn test_curtailment_mw_to_mwh_conversion() {
        let mw = 100.0f64;
        let mwh = mw * 0.25;
        assert!((mwh - 25.0).abs() < 0.001);
    }

    #[test]
    fn test_backfill_days_cap() {
        let requested = 9999i64;
        let capped = requested.min(730);
        assert_eq!(capped, 730);
    }
}
