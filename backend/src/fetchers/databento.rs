//! Databento Historical API fetcher for ICE Futures Europe settlement prices.
//!
//! Instruments: TTF natural gas, EUA carbon, ARA coal (API2 CIF)
//! Dataset: IFEU.IMPACT (ICE Futures Europe, iMpact feed)
//! Schema: Statistics (official exchange settlement prices)

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use databento::{
    dbn::{OhlcvMsg, Schema, SType, StatMsg},
    historical::timeseries::GetRangeParams,
    HistoricalClient,
};
use time::OffsetDateTime;

use crate::db::models::FuelOhlcv;

/// Instrument definition.
#[derive(Debug, Clone, Copy)]
pub struct Instrument {
    pub name: &'static str,
    pub dataset: Dataset,
    pub symbol: &'static str,
    pub unit: &'static str,
}

use databento::dbn::Dataset;

pub const INSTRUMENTS: &[Instrument] = &[
    Instrument {
        name:    "TTF",
        dataset: Dataset::NdexImpact,  // FIXED: ICE Endex
        symbol:  "TFM.FUT",            // FIXED: TFM is the EUR/MWh contract (TFU is USD/MMBtu)
        unit:    "EUR/MWh",
    },
    Instrument {
        name:    "EUA",
        dataset: Dataset::NdexImpact,  // CORRECT: ICE Endex
        symbol:  "ECF.FUT",            // CORRECT: ECF is the EUA Future
        unit:    "EUR/t",
    },
    Instrument {
        name:    "ARA",
        dataset: Dataset::IfeuImpact,  // CORRECT: ICE Europe Commodities
        symbol:  "ATW.FUT",            // CORRECT: API2 Rotterdam Coal
        unit:    "USD/t",
    },
    Instrument {
        name:    "GAB",
        dataset: Dataset::NdexImpact,  // FIXED: ICE Endex
        symbol:  "GAB.FUT",            // CORRECT: German Power Base
        unit:    "EUR/MWh",
    },
];

/// Convert `NaiveDate` to `time::OffsetDateTime` at midnight UTC.
/// The databento crate uses `time` not `chrono`.
fn to_time_odt(date: NaiveDate) -> OffsetDateTime {
    let ts = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
    OffsetDateTime::from_unix_timestamp(ts.timestamp()).unwrap()
}

/// MoM delta percentage from a price history array.
pub fn mom_delta_pct(history: &[f64]) -> f64 {
    if history.len() < 2 {
        return 0.0;
    }
    let oldest = history[0];
    let latest = history[history.len() - 1];
    if oldest == 0.0 {
        return 0.0;
    }
    ((latest - oldest) / oldest * 100.0 * 100.0).round() / 100.0
}

/// Debug helper: print all stat records for a symbol on a recent trading day.
/// Call once at startup to verify symbols and stat_types, then remove.
pub async fn debug_print_stats(
    api_key: &str,
    dataset: &str,
    symbol: &str,
) -> anyhow::Result<()> {
    let mut client = HistoricalClient::builder().key(api_key)?.build()?;

    let mut decoder = client
        .timeseries()
        .get_range(
            &GetRangeParams::builder()
                .dataset(dataset)
                .date_time_range(
                    time::macros::datetime!(2026-03-27 00:00 UTC)
                        ..time::macros::datetime!(2026-03-28 00:00 UTC),
                )
                .symbols(vec![symbol])
                .stype_in(SType::Parent)
                .schema(Schema::Statistics)
                .build(),
        )
        .await?;

    tracing::debug!("=== DEBUG {} / {} ===", dataset, symbol);
    while let Some(msg) = decoder.decode_record::<StatMsg>().await? {
        tracing::debug!(
            "  stat_type={} price_f64={:.6} ts_ref={}",
            msg.stat_type,
            msg.price_f64(),
            msg.ts_ref,
        );
    }
    Ok(())
}

/// Fetch settlement prices for all three instruments over a date range.
/// Returns Vec of (ts, instrument_name, price, unit).
pub async fn fetch_history(
    api_key: &str,
    days: i64,
) -> anyhow::Result<Vec<(DateTime<Utc>, &'static str, f64, &'static str)>> {
    let today = Utc::now().date_naive();
    let start_date = today - chrono::Duration::days(days);

    let mut all: Vec<(DateTime<Utc>, &'static str, f64, &'static str)> = Vec::new();

    for instrument in INSTRUMENTS {
        tracing::info!(
            "Databento backfill: fetching {} ({}) from {} to {} [dataset={}]",
            instrument.name,
            instrument.symbol,
            start_date,
            today,
            instrument.dataset
        );

        let mut client = HistoricalClient::builder().key(api_key)?.build()?;

        let mut decoder = client
            .timeseries()
            .get_range(
                &GetRangeParams::builder()
                    .dataset(instrument.dataset)
                    .date_time_range(to_time_odt(start_date)..to_time_odt(today))
                    .symbols(vec![instrument.symbol])
                    .stype_in(SType::Parent)
                    .schema(Schema::Statistics)
                    .build(),
            )
            .await
            .map_err(|e| {
                anyhow::anyhow!("Databento fetch failed for {}: {}", instrument.name, e)
            })?;

        let mut count = 0usize;

        while let Some(msg) = decoder.decode_record::<StatMsg>().await.map_err(|e| {
            anyhow::anyhow!("Databento decode error for {}: {}", instrument.name, e)
        })? {
            // stat_type=1 is SettlementPrice for all ICE instruments
            if msg.stat_type != 1 {
                continue;
            }

            let price = msg.price_f64();

            // Filter NaN, non-positive, and Databento's UNDEF_PRICE sentinel
            // (i64::MAX / 1e9 ≈ 9.22e9 — means "no price defined")
            if price.is_nan() || price <= 0.0 || price > 1_000_000.0 {
                continue;
            }

            let ts = DateTime::from_timestamp(
                (msg.ts_ref / 1_000_000_000) as i64,
                (msg.ts_ref % 1_000_000_000) as u32,
            )
            .unwrap_or_else(Utc::now);

            all.push((ts, instrument.name, price, instrument.unit));
            count += 1;
        }

        tracing::info!(
            "Databento {}: {} valid settlement records (stat_type=1)",
            instrument.name,
            count
        );
    }

    // Sort oldest-first, then deduplicate by (date, instrument)
    all.sort_by_key(|(ts, name, _, _)| (*ts, *name));
    all.dedup_by(|a, b| a.0.date_naive() == b.0.date_naive() && a.1 == b.1);

    tracing::info!("Databento backfill total: {} records", all.len());
    Ok(all)
}

/// Fetch today's settlement for all instruments.
/// Called by the daily scheduler and by the live fuels route fallback.
pub async fn fetch_today(
    api_key: &str,
) -> Vec<(&'static str, f64, &'static str)> {
    let now = Utc::now();
    let today = now.date_naive();
    // Use now (not tomorrow midnight) as end to avoid 422 when dataset hasn't caught up yet
    let now_odt = OffsetDateTime::from_unix_timestamp(now.timestamp()).unwrap();

    let mut results = Vec::new();

    for instrument in INSTRUMENTS {
        let client_result = HistoricalClient::builder()
            .key(api_key)
            .and_then(|b| Ok(b.build()?));

        let mut client = match client_result {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(
                    "Databento client build failed for {}: {}",
                    instrument.name,
                    e
                );
                continue;
            }
        };

        let decoder_result = client
            .timeseries()
            .get_range(
                &GetRangeParams::builder()
                    .dataset(instrument.dataset)
                    .date_time_range(to_time_odt(today)..now_odt)
                    .symbols(vec![instrument.symbol])
                    .stype_in(SType::Parent)
                    .schema(Schema::Statistics)
                    .build(),
            )
            .await;

        let mut decoder = match decoder_result {
            Ok(d) => d,
            Err(e) => {
                tracing::warn!(
                    "Databento fetch_today failed for {}: {}",
                    instrument.name,
                    e
                );
                continue;
            }
        };

        while let Ok(Some(msg)) = decoder.decode_record::<StatMsg>().await {
            // stat_type=1 is SettlementPrice for all ICE instruments
            if msg.stat_type != 1 {
                continue;
            }
            let price = msg.price_f64();
            // Filter NaN, non-positive, and UNDEF_PRICE sentinel (~9.22e9)
            if price.is_nan() || price <= 0.0 || price > 1_000_000.0 {
                continue;
            }
            tracing::info!(
                "Databento today: {} = {:.4} {}",
                instrument.name,
                price,
                instrument.unit
            );
            results.push((instrument.name, price, instrument.unit));
            break;
        }
    }

    results
}

/// Fetch daily OHLCV bars for one exact raw symbol over a date range.
/// Uses `SType::RawSymbol` — no parent resolution, fast single-instrument query.
pub async fn fetch_ohlcv_symbol(
    api_key: &str,
    dataset: Dataset,
    raw_symbol: &str,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> anyhow::Result<Vec<FuelOhlcv>> {
    let mut client = HistoricalClient::builder().key(api_key)?.build()?;

    let mut decoder = client
        .timeseries()
        .get_range(
            &GetRangeParams::builder()
                .dataset(dataset)
                .symbols(vec![raw_symbol])
                .schema(Schema::Ohlcv1D)
                .date_time_range(to_time_odt(start_date)..to_time_odt(end_date))
                .stype_in(SType::RawSymbol)
                .build(),
        )
        .await?;

    let ticker = ticker_for_symbol(raw_symbol);
    let unit = unit_for_symbol(raw_symbol);
    let mut bars: Vec<FuelOhlcv> = Vec::new();

    while let Some(msg) = decoder.decode_record::<OhlcvMsg>().await? {
        let ts = DateTime::from_timestamp((msg.hd.ts_event / 1_000_000_000) as i64, 0)
            .unwrap_or_else(Utc::now);

        let close = msg.close_f64();
        // Filter UNDEF_PRICE sentinel (~9.22e9) and invalid prices
        if close <= 0.0 || close > 1_000_000.0 {
            continue;
        }

        // Clamp OHLC sentinels: replace UNDEF_PRICE (~9.22e9) with the valid close
        let clamp = |v: f64| if v > 1_000_000.0 || v <= 0.0 { close } else { v };

        bars.push(FuelOhlcv {
            date: ts.date_naive(),
            instrument_id: msg.hd.instrument_id as i64,
            dataset: dataset.as_str().to_string(),
            ticker: ticker.clone(),
            raw_symbol: raw_symbol.to_string(),
            unit: unit.clone(),
            open: clamp(msg.open_f64()),
            high: clamp(msg.high_f64()),
            low: clamp(msg.low_f64()),
            close,
            volume: msg.volume as i64,
        });
    }

    tracing::info!("fetch_ohlcv_symbol {}: {} bars", raw_symbol, bars.len());
    Ok(bars)
}

/// Generate all exact symbols needed for CSS/CDS over a date range,
/// then fetch OHLCV bars for each via `SType::RawSymbol`.
///
/// For rolling 3-month spreads we need M+1, M+2, M+3 for every day in the
/// range, so we generate symbols from (start_date + 1 month) through
/// (end_date + 3 months).
pub async fn fetch_ohlcv_for_css(
    api_key: &str,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> anyhow::Result<Vec<FuelOhlcv>> {
    use crate::analytics::css::{add_months, month_to_ice_code};

    let sym_start = add_months(start_date, 1);
    let sym_end = add_months(end_date, 3);

    let mut symbols: Vec<(Dataset, String, &str, &str)> = Vec::new();

    // Walk month by month from sym_start to sym_end
    let mut cursor =
        NaiveDate::from_ymd_opt(sym_start.year(), sym_start.month(), 1).unwrap();
    let sym_end_month =
        NaiveDate::from_ymd_opt(sym_end.year(), sym_end.month(), 1).unwrap();

    while cursor <= sym_end_month {
        let code = month_to_ice_code(cursor.month()).unwrap();
        let yy = cursor.year() % 100;

        // TFM monthly — NdexImpact (ICE Endex)
        symbols.push((
            Dataset::NdexImpact,
            format!("TFM FM{}00{:02}!", code, yy),
            "TTF",
            "EUR/MWh",
        ));

        // GAB monthly — NdexImpact (ICE Endex)
        symbols.push((
            Dataset::NdexImpact,
            format!("GAB FM{}00{:02}!", code, yy),
            "GAB",
            "EUR/MWh",
        ));

        // ATW monthly — IfeuImpact (ICE Futures Europe)
        symbols.push((
            Dataset::IfeuImpact,
            format!("ATW FM{}00{:02}!", code, yy),
            "ARA",
            "USD/t",
        ));

        cursor = add_months(cursor, 1);
    }

    // ECF: one December contract per year in the range
    let first_year = start_date.year();
    let last_year = add_months(end_date, 3).year();
    for year in first_year..=last_year {
        symbols.push((
            Dataset::NdexImpact,
            format!("ECF FMZ00{:02}!", year % 100),
            "EUA",
            "EUR/t",
        ));
    }

    // Deduplicate
    symbols.sort_by(|a, b| a.1.cmp(&b.1));
    symbols.dedup_by(|a, b| a.1 == b.1);

    tracing::info!(
        "fetch_ohlcv_for_css: fetching {} exact symbols for {} → {}",
        symbols.len(),
        start_date,
        end_date
    );

    let mut all: Vec<FuelOhlcv> = Vec::new();

    for (dataset, raw_symbol, _ticker, _unit) in &symbols {
        match fetch_ohlcv_symbol(api_key, *dataset, raw_symbol, start_date, end_date).await {
            Ok(mut bars) => all.append(&mut bars),
            Err(e) => {
                // A missing symbol is expected (contract not yet listed, etc.)
                tracing::debug!("Symbol {} not available: {}", raw_symbol, e);
            }
        }
    }

    tracing::info!("fetch_ohlcv_for_css total: {} bars", all.len());
    Ok(all)
}

/// Infer ticker from raw symbol prefix.
fn ticker_for_symbol(raw: &str) -> String {
    if raw.starts_with("TFM") {
        "TTF".to_string()
    } else if raw.starts_with("ECF") {
        "EUA".to_string()
    } else if raw.starts_with("ATW") {
        "ARA".to_string()
    } else if raw.starts_with("GAB") {
        "GAB".to_string()
    } else {
        "UNKNOWN".to_string()
    }
}

/// Infer unit from raw symbol prefix.
fn unit_for_symbol(raw: &str) -> String {
    if raw.starts_with("TFM") || raw.starts_with("GAB") {
        "EUR/MWh".to_string()
    } else if raw.starts_with("ECF") {
        "EUR/t".to_string()
    } else if raw.starts_with("ATW") {
        "USD/t".to_string()
    } else {
        String::new()
    }
}

/// Verify API key by listing datasets (lightweight metadata call).
pub async fn verify_api_key(api_key: &str) -> anyhow::Result<()> {
    let mut client = HistoricalClient::builder().key(api_key)?.build()?;
    client
        .metadata()
        .list_datasets(None)
        .await
        .map_err(|e| anyhow::anyhow!("Databento key verification failed: {}", e))?;
    tracing::info!("Databento API key: valid");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instrument_datasets() {
        assert_eq!(INSTRUMENTS[0].name, "TTF");
        assert!(matches!(INSTRUMENTS[0].dataset, Dataset::NdexImpact));
        assert_eq!(INSTRUMENTS[2].name, "ARA");
        assert!(matches!(INSTRUMENTS[2].dataset, Dataset::IfeuImpact));
    }

    #[test]
    fn test_to_time_odt_roundtrip() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 28).unwrap();
        let odt = to_time_odt(date);
        let back = odt.unix_timestamp();
        let back_date = chrono::DateTime::from_timestamp(back, 0)
            .unwrap()
            .date_naive();
        assert_eq!(back_date, date);
    }

    #[test]
    fn test_dedup_by_day_and_instrument() {
        use chrono::TimeZone;
        let ts1 = chrono::Utc.with_ymd_and_hms(2026, 3, 28, 10, 0, 0).unwrap();
        let ts2 = chrono::Utc.with_ymd_and_hms(2026, 3, 28, 17, 30, 0).unwrap();
        let ts3 = chrono::Utc.with_ymd_and_hms(2026, 3, 29, 17, 30, 0).unwrap();
        let mut v: Vec<(chrono::DateTime<Utc>, &str, f64, &str)> = vec![
            (ts1, "TTF", 54.0, "EUR/MWh"),
            (ts2, "TTF", 54.5, "EUR/MWh"),
            (ts3, "TTF", 55.0, "EUR/MWh"),
        ];
        v.sort_by_key(|(ts, name, _, _)| (*ts, *name));
        v.dedup_by(|a, b| a.0.date_naive() == b.0.date_naive() && a.1 == b.1);
        assert_eq!(v.len(), 2);
        assert!((v[0].2 - 54.0).abs() < 0.001);
        assert!((v[1].2 - 55.0).abs() < 0.001);
    }

    #[test]
    fn test_mom_delta_pct() {
        let history = vec![100.0, 105.0, 110.0];
        let delta = mom_delta_pct(&history);
        assert!((delta - 10.0).abs() < 0.01);
    }
}
