//! Databento Historical API fetcher for ICE Futures Europe settlement prices.
//!
//! Instruments: TTF natural gas, EUA carbon, ARA coal (API2 CIF)
//! Dataset: IFEU.IMPACT (ICE Futures Europe, iMpact feed)
//! Schema: Statistics (official exchange settlement prices)

use chrono::{DateTime, NaiveDate, Utc};
use databento::{
    dbn::{Schema, SType, StatMsg},
    historical::timeseries::GetRangeParams,
    HistoricalClient,
};
use time::OffsetDateTime;

/// Instrument definition.
#[derive(Debug, Clone, Copy)]
pub struct Instrument {
    pub name: &'static str,
    pub dataset: &'static str,
    pub symbol: &'static str,
    pub unit: &'static str,
    pub settlement_stat_type: u16,
    pub price_min: f64,
    pub price_max: f64,
}

pub const INSTRUMENTS: &[Instrument] = &[
    Instrument {
        name: "TTF",
        dataset: "IFEU.IMPACT",
        symbol: "TFU.FUT",
        unit: "EUR/MWh",
        settlement_stat_type: 4, // confirmed: SettlementPrice_print_stats
        price_min: 5.0,
        price_max: 300.0,
    },
    Instrument {
        name: "EUA",
        dataset: "IFEU.IMPACT",
        symbol: "ECF.FUT",
        unit: "EUR/t",
        settlement_stat_type: 1,
        price_min: 5.0,
        price_max: 200.0,
    },
    Instrument {
        name: "ARA",
        dataset: "IFEU.IMPACT",
        symbol: "ATW.FUT",
        unit: "USD/t",
        settlement_stat_type: 1,
        price_min: 30.0,
        price_max: 500.0,
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

    tracing::info!("=== DEBUG {} / {} ===", dataset, symbol);
    while let Some(msg) = decoder.decode_record::<StatMsg>().await? {
        tracing::info!(
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
            if msg.stat_type != instrument.settlement_stat_type {
                continue;
            }

            let price = msg.price_f64();

            if price.is_nan() {
                continue;
            }

            if price < instrument.price_min || price > instrument.price_max {
                tracing::warn!(
                    "Databento {} price {:.4} outside bounds [{}, {}] — skipping",
                    instrument.name,
                    price,
                    instrument.price_min,
                    instrument.price_max
                );
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
            "Databento backfill: {} settlement records for {}",
            count,
            instrument.name
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
    let today = Utc::now().date_naive();
    let tomorrow = today + chrono::Duration::days(1);

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
                    .date_time_range(to_time_odt(today)..to_time_odt(tomorrow))
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
            if msg.stat_type != instrument.settlement_stat_type {
                continue;
            }
            let price = msg.price_f64();
            if price.is_nan()
                || price < instrument.price_min
                || price > instrument.price_max
            {
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
    fn test_price_bounds() {
        let ttf = &INSTRUMENTS[0];
        assert!(54.53 >= ttf.price_min && 54.53 <= ttf.price_max);
        let ara = &INSTRUMENTS[2];
        assert!(130.90 >= ara.price_min && 130.90 <= ara.price_max);
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
