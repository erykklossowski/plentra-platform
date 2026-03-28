use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct StooqRow {
    #[serde(rename = "Date")]
    _date: String,
    #[serde(rename = "Open")]
    _open: f64,
    #[serde(rename = "High")]
    _high: f64,
    #[serde(rename = "Low")]
    _low: f64,
    #[serde(rename = "Close")]
    close: f64,
    #[serde(rename = "Volume")]
    _volume: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct StooqResult {
    pub current_price: f64,
    pub change_pct: f64,
    pub history_30d: Vec<f64>,
}

pub async fn fetch_commodity(client: &reqwest::Client, symbol: &str) -> Result<StooqResult> {
    let url = format!("https://stooq.com/q/d/l/?s={symbol}&i=d");

    let response = client
        .get(&url)
        .send()
        .await
        .context(format!("Failed to fetch {symbol} from Stooq"))?;

    let text = response
        .text()
        .await
        .context(format!("Failed to read response body for {symbol}"))?;

    parse_csv(&text, symbol)
}

fn parse_csv(csv_text: &str, symbol: &str) -> Result<StooqResult> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(csv_text.as_bytes());

    let mut rows: Vec<f64> = Vec::new();

    for result in reader.deserialize::<StooqRow>() {
        match result {
            Ok(row) => rows.push(row.close),
            Err(e) => {
                tracing::warn!("Skipping malformed CSV row for {symbol}: {e}");
            }
        }
    }

    anyhow::ensure!(!rows.is_empty(), "No valid data rows for {symbol}");

    // Take last 30 entries
    let history: Vec<f64> = if rows.len() > 30 {
        rows[rows.len() - 30..].to_vec()
    } else {
        rows.clone()
    };

    let current_price = *history.last().unwrap();
    let change_pct = if history.len() >= 2 {
        let prev = history[history.len() - 2];
        if prev != 0.0 {
            ((current_price - prev) / prev) * 100.0
        } else {
            0.0
        }
    } else {
        0.0
    };

    Ok(StooqResult {
        current_price,
        change_pct: (change_pct * 100.0).round() / 100.0, // round to 2 decimal places
        history_30d: history,
    })
}

pub async fn fetch_ttf(client: &reqwest::Client) -> Result<StooqResult> {
    fetch_commodity(client, "ttf.f").await
}

pub async fn fetch_eua(client: &reqwest::Client) -> Result<StooqResult> {
    fetch_commodity(client, "co2e.f").await
}

pub async fn fetch_ara(client: &reqwest::Client) -> Result<StooqResult> {
    fetch_commodity(client, "arac.f").await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_csv() {
        let csv = r#"Date,Open,High,Low,Close,Volume
2026-03-24,33.50,34.10,33.20,33.80,1000
2026-03-25,33.80,34.50,33.60,34.00,1200
2026-03-26,34.00,34.80,33.90,34.20,1100
"#;
        let result = parse_csv(csv, "test").unwrap();
        assert_eq!(result.current_price, 34.20);
        assert_eq!(result.history_30d.len(), 3);
        // change_pct = ((34.20 - 34.00) / 34.00) * 100 = 0.588... -> rounded to 0.59
        assert!((result.change_pct - 0.59).abs() < 0.01);
    }

    #[test]
    fn test_parse_csv_empty() {
        let csv = "Date,Open,High,Low,Close,Volume\n";
        let result = parse_csv(csv, "test");
        assert!(result.is_err());
    }
}
