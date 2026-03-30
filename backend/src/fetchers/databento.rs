//! Databento Historical API fetcher for ICE Futures Europe settlement prices.
//!
//! Instruments: TTF natural gas, EUA carbon, ARA coal (API2 CIF)
//! Dataset: IFEU.IMPACT (ICE Futures Europe, iMpact feed)
//! Schema: Statistics (official exchange settlement prices)

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use databento::{
    dbn::{decode::DbnMetadata, OhlcvMsg, Schema, SType, StatMsg},
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

            if price.is_nan() || price <= 0.0 {
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
            if price.is_nan() || price <= 0.0 {
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

/// Fetch daily OHLCV bars for all instruments over a date range.
/// One FuelOhlcv per (date, instrument_id) — no aggregation.
/// Missing dates (weekends, holidays) simply produce no record.
pub async fn fetch_ohlcv(
    api_key: &str,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> anyhow::Result<Vec<FuelOhlcv>> {
    let mut all: Vec<FuelOhlcv> = Vec::new();

    for instrument_def in INSTRUMENTS {
        tracing::info!(
            "fetch_ohlcv: {} ({}) {} -> {} [{}]",
            instrument_def.name,
            instrument_def.symbol,
            start_date,
            end_date,
            instrument_def.dataset
        );

        let mut client = HistoricalClient::builder().key(api_key)?.build()?;

        let mut decoder = client
            .timeseries()
            .get_range(
                &GetRangeParams::builder()
                    .dataset(instrument_def.dataset)
                    .symbols(vec![instrument_def.symbol])
                    .schema(Schema::Ohlcv1D)
                    .date_time_range(to_time_odt(start_date)..to_time_odt(end_date))
                    .stype_in(SType::Parent)
                    .build(),
            )
            .await?;

        let metadata = decoder.metadata().clone();
        let mut count = 0usize;

        while let Some(msg) = decoder.decode_record::<OhlcvMsg>().await? {
            // ts_event is the start of the 1-day bar in nanoseconds
            let ts = chrono::DateTime::from_timestamp(
                (msg.hd.ts_event / 1_000_000_000) as i64,
                0,
            )
            .unwrap_or_else(chrono::Utc::now);

            let date = ts.date_naive();
            let instrument_id = msg.hd.instrument_id as i64;

            // Resolve raw_symbol from SymbolMap
            // TsSymbolMap::get takes (time::Date, u32)
            let time_date = time::Date::from_calendar_date(
                date.year(),
                time::Month::try_from(date.month() as u8).unwrap(),
                date.day() as u8,
            )
            .ok();
            let raw_symbol: String = metadata
                .symbol_map()
                .ok()
                .and_then(|sm| {
                    time_date.and_then(|td| {
                        sm.get(td, msg.hd.instrument_id).map(|s| s.to_string())
                    })
                })
                .unwrap_or_else(|| format!("UNKNOWN_{}", instrument_id));

            // Decode prices: int64 fixed-point, 1 unit = 1e-9
            let open = msg.open as f64 / 1_000_000_000.0;
            let high = msg.high as f64 / 1_000_000_000.0;
            let low = msg.low as f64 / 1_000_000_000.0;
            let close = msg.close as f64 / 1_000_000_000.0;

            // Sanity check — reject obviously wrong prices
            if close <= 0.0 {
                tracing::debug!(
                    "Skipping zero/negative close for {} {} on {}",
                    instrument_def.name,
                    raw_symbol,
                    date
                );
                continue;
            }

            all.push(FuelOhlcv {
                date,
                instrument_id,
                dataset: instrument_def.dataset.to_string(),
                ticker: instrument_def.name.to_string(),
                raw_symbol,
                unit: instrument_def.unit.to_string(),
                open,
                high,
                low,
                close,
                volume: msg.volume as i64,
            });

            count += 1;
        }

        tracing::info!("fetch_ohlcv {}: {} bars", instrument_def.name, count);
    }

    tracing::info!(
        "fetch_ohlcv total: {} bars across all instruments",
        all.len()
    );
    Ok(all)
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
