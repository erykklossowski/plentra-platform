use chrono::NaiveDate;

/// One daily OHLCV bar for a single futures instrument.
#[derive(Debug, Clone)]
pub struct FuelOhlcv {
    pub date: NaiveDate,
    pub instrument_id: i64,
    pub dataset: String,
    pub ticker: String,
    pub raw_symbol: String,
    pub unit: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64, // price used for CSS
    pub volume: i64,
}
