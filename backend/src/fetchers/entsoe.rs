use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;

const ENTSOE_BASE_URL: &str = "https://web-api.tp.entsoe.eu/api";
const POLAND_AREA: &str = "10YPL-AREA-----S";

/// Per-type generation data: PSR type code -> total MW
#[derive(Debug, Clone, Default)]
pub struct GenerationByType {
    pub data: HashMap<String, f64>,
}

impl GenerationByType {
    pub fn get(&self, psr_type: &str) -> f64 {
        self.data.get(psr_type).copied().unwrap_or(0.0)
    }

    pub fn wind_mw(&self) -> f64 {
        self.get("B19") + self.get("B18") // onshore + offshore
    }

    pub fn solar_mw(&self) -> f64 {
        self.get("B16")
    }

    pub fn lignite_mw(&self) -> f64 {
        self.get("B02")
    }

    pub fn nuclear_mw(&self) -> f64 {
        self.get("B14")
    }

    pub fn total_renewable_mw(&self) -> f64 {
        self.wind_mw() + self.solar_mw()
    }
}

/// Fetch actual generation per type (document A75, process A16)
pub async fn fetch_actual_generation(
    client: &reqwest::Client,
    token: &str,
) -> Result<GenerationByType> {
    let now = Utc::now();
    let start = (now - Duration::hours(2)).format("%Y%m%d%H00").to_string();
    let end = now.format("%Y%m%d%H00").to_string();

    let url = format!(
        "{ENTSOE_BASE_URL}?securityToken={token}&documentType=A75&processType=A16\
         &in_Domain={POLAND_AREA}&outBiddingZone_Domain={POLAND_AREA}\
         &periodStart={start}&periodEnd={end}"
    );

    let text = fetch_xml(client, &url, "A75").await?;
    parse_generation_xml(&text)
}

/// Fetch actual total load (document A65, process A16)
pub async fn fetch_actual_load(
    client: &reqwest::Client,
    token: &str,
) -> Result<f64> {
    let now = Utc::now();
    let start = (now - Duration::hours(2)).format("%Y%m%d%H00").to_string();
    let end = now.format("%Y%m%d%H00").to_string();

    let url = format!(
        "{ENTSOE_BASE_URL}?securityToken={token}&documentType=A65&processType=A16\
         &outBiddingZone_Domain={POLAND_AREA}\
         &periodStart={start}&periodEnd={end}"
    );

    let text = fetch_xml(client, &url, "A65").await?;
    parse_load_xml(&text)
}

/// Fetch day-ahead generation forecast for wind/solar (document A69)
pub async fn fetch_generation_forecast(
    client: &reqwest::Client,
    token: &str,
) -> Result<(f64, f64)> {
    let now = Utc::now();
    let start = now.format("%Y%m%d0000").to_string();
    let end = (now + Duration::hours(24)).format("%Y%m%d0000").to_string();

    let url = format!(
        "{ENTSOE_BASE_URL}?securityToken={token}&documentType=A69&processType=A01\
         &in_Domain={POLAND_AREA}\
         &periodStart={start}&periodEnd={end}"
    );

    let text = fetch_xml(client, &url, "A69").await?;
    parse_forecast_xml(&text)
}

/// Fetch 24-hour generation profile for hourly breakdown
pub async fn fetch_hourly_generation(
    client: &reqwest::Client,
    token: &str,
) -> Result<Vec<(u32, GenerationByType)>> {
    let now = Utc::now();
    let start = now.format("%Y%m%d0000").to_string();
    let end = (now + Duration::hours(24)).format("%Y%m%d0000").to_string();

    let url = format!(
        "{ENTSOE_BASE_URL}?securityToken={token}&documentType=A75&processType=A16\
         &in_Domain={POLAND_AREA}&outBiddingZone_Domain={POLAND_AREA}\
         &periodStart={start}&periodEnd={end}"
    );

    let text = fetch_xml(client, &url, "A75-hourly").await?;
    parse_hourly_generation_xml(&text)
}

/// Fetch 24-hour load profile
pub async fn fetch_hourly_load(
    client: &reqwest::Client,
    token: &str,
) -> Result<Vec<(u32, f64)>> {
    let now = Utc::now();
    let start = now.format("%Y%m%d0000").to_string();
    let end = (now + Duration::hours(24)).format("%Y%m%d0000").to_string();

    let url = format!(
        "{ENTSOE_BASE_URL}?securityToken={token}&documentType=A65&processType=A16\
         &outBiddingZone_Domain={POLAND_AREA}\
         &periodStart={start}&periodEnd={end}"
    );

    let text = fetch_xml(client, &url, "A65-hourly").await?;
    parse_hourly_load_xml(&text)
}

async fn fetch_xml(client: &reqwest::Client, url: &str, label: &str) -> Result<String> {
    tracing::info!("Fetching ENTSO-E {label}");

    let response = client
        .get(url)
        .header("Accept", "application/xml")
        .send()
        .await
        .context(format!("Failed to fetch ENTSO-E {label}"))?;

    let status = response.status();
    let text = response
        .text()
        .await
        .context(format!("Failed to read ENTSO-E {label} response"))?;

    if !status.is_success() {
        tracing::warn!("ENTSO-E {label} returned {status}: {}", &text[..text.len().min(200)]);
        anyhow::bail!("ENTSO-E {label} returned HTTP {status}");
    }

    if text.contains("<Reason>") {
        tracing::warn!("ENTSO-E {label} error response: {}", &text[..text.len().min(300)]);
        anyhow::bail!("ENTSO-E {label} returned error in XML");
    }

    Ok(text)
}

fn parse_generation_xml(xml: &str) -> Result<GenerationByType> {
    let mut reader = Reader::from_str(xml);
    let mut result = GenerationByType::default();
    let mut current_psr: Option<String> = None;
    let mut last_quantity: Option<f64> = None;
    let mut in_psr_type = false;
    let mut in_quantity = false;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match name.as_str() {
                    "psrType" | "MktPSRType" => in_psr_type = true,
                    "quantity" => in_quantity = true,
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "TimeSeries" {
                    if let (Some(psr), Some(qty)) = (&current_psr, last_quantity) {
                        let entry = result.data.entry(psr.clone()).or_insert(0.0);
                        *entry = qty; // take latest value
                    }
                    current_psr = None;
                    last_quantity = None;
                }
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                if in_psr_type {
                    current_psr = Some(text);
                    in_psr_type = false;
                } else if in_quantity {
                    if let Ok(v) = text.parse::<f64>() {
                        last_quantity = Some(v);
                    }
                    in_quantity = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                tracing::warn!("XML parse error: {e}");
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    anyhow::ensure!(!result.data.is_empty(), "No generation data found in XML");
    tracing::info!("Parsed generation: {:?}", result.data.keys().collect::<Vec<_>>());
    Ok(result)
}

fn parse_load_xml(xml: &str) -> Result<f64> {
    let mut reader = Reader::from_str(xml);
    let mut last_quantity: f64 = 0.0;
    let mut in_quantity = false;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                if e.name().as_ref() == b"quantity" {
                    in_quantity = true;
                }
            }
            Ok(Event::Text(e)) => {
                if in_quantity {
                    if let Ok(v) = e.unescape().unwrap_or_default().parse::<f64>() {
                        last_quantity = v;
                    }
                    in_quantity = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    anyhow::ensure!(last_quantity > 0.0, "No load data found in XML");
    Ok(last_quantity)
}

fn parse_forecast_xml(xml: &str) -> Result<(f64, f64)> {
    let mut reader = Reader::from_str(xml);
    let mut current_psr: Option<String> = None;
    let mut wind_total: f64 = 0.0;
    let mut solar_total: f64 = 0.0;
    let mut wind_count: u32 = 0;
    let mut solar_count: u32 = 0;
    let mut in_psr_type = false;
    let mut in_quantity = false;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match name.as_str() {
                    "psrType" | "MktPSRType" => in_psr_type = true,
                    "quantity" => in_quantity = true,
                    _ => {}
                }
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                if in_psr_type {
                    current_psr = Some(text);
                    in_psr_type = false;
                } else if in_quantity {
                    if let Ok(v) = text.parse::<f64>() {
                        match current_psr.as_deref() {
                            Some("B19") | Some("B18") => {
                                wind_total += v;
                                wind_count += 1;
                            }
                            Some("B16") => {
                                solar_total += v;
                                solar_count += 1;
                            }
                            _ => {}
                        }
                    }
                    in_quantity = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    let wind_avg = if wind_count > 0 { wind_total / wind_count as f64 } else { 0.0 };
    let solar_avg = if solar_count > 0 { solar_total / solar_count as f64 } else { 0.0 };

    Ok((wind_avg, solar_avg))
}

fn parse_hourly_generation_xml(xml: &str) -> Result<Vec<(u32, GenerationByType)>> {
    // Simplified: parse all points and group by position (hour)
    let mut reader = Reader::from_str(xml);
    let mut hourly: HashMap<u32, GenerationByType> = HashMap::new();
    let mut current_psr: Option<String> = None;
    let mut current_position: Option<u32> = None;
    let mut in_psr_type = false;
    let mut in_position = false;
    let mut in_quantity = false;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match name.as_str() {
                    "psrType" | "MktPSRType" => in_psr_type = true,
                    "position" => in_position = true,
                    "quantity" => in_quantity = true,
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "TimeSeries" {
                    current_psr = None;
                }
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                if in_psr_type {
                    current_psr = Some(text);
                    in_psr_type = false;
                } else if in_position {
                    current_position = text.parse().ok();
                    in_position = false;
                } else if in_quantity {
                    if let (Some(psr), Some(pos), Ok(qty)) =
                        (&current_psr, current_position, text.parse::<f64>())
                    {
                        let hour = pos.saturating_sub(1); // position is 1-based
                        if hour < 24 {
                            let entry = hourly.entry(hour).or_default();
                            entry.data.insert(psr.clone(), qty);
                        }
                    }
                    in_quantity = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    let mut result: Vec<(u32, GenerationByType)> = hourly.into_iter().collect();
    result.sort_by_key(|(h, _)| *h);
    Ok(result)
}

fn parse_hourly_load_xml(xml: &str) -> Result<Vec<(u32, f64)>> {
    let mut reader = Reader::from_str(xml);
    let mut hourly: Vec<(u32, f64)> = Vec::new();
    let mut current_position: Option<u32> = None;
    let mut in_position = false;
    let mut in_quantity = false;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match name.as_str() {
                    "position" => in_position = true,
                    "quantity" => in_quantity = true,
                    _ => {}
                }
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                if in_position {
                    current_position = text.parse().ok();
                    in_position = false;
                } else if in_quantity {
                    if let (Some(pos), Ok(qty)) = (current_position, text.parse::<f64>()) {
                        let hour = pos.saturating_sub(1);
                        if hour < 24 {
                            hourly.push((hour, qty));
                        }
                    }
                    in_quantity = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    hourly.sort_by_key(|(h, _)| *h);
    // Deduplicate: keep last value per hour
    hourly.dedup_by_key(|(h, _)| *h);
    Ok(hourly)
}

// ─── Calculation functions ───

pub fn calculate_residual_demand_gw(load_mw: f64, wind_mw: f64, solar_mw: f64) -> f64 {
    round2((load_mw - wind_mw - solar_mw) / 1000.0)
}

pub fn calculate_must_run_floor_gw(gen: &GenerationByType) -> f64 {
    let nuclear = gen.nuclear_mw();
    let lignite_min = gen.lignite_mw() * 0.6; // technical minimum
    round2((nuclear + lignite_min) / 1000.0)
}

pub fn calculate_cri(load_mw: f64, residual_mw: f64, must_run_mw: f64, renewable_mw: f64) -> (f64, String) {
    if load_mw <= 0.0 {
        return (0.0, "LOW".to_string());
    }

    let grid_stress = 1.0 - ((residual_mw - must_run_mw) / load_mw);
    let renewable_penetration = renewable_mw / load_mw;
    let cri = (100.0 * grid_stress * renewable_penetration).clamp(0.0, 100.0);
    let cri = round2(cri);

    let level = match cri as u32 {
        0..=33 => "LOW",
        34..=59 => "MODERATE",
        60..=79 => "ELEVATED",
        _ => "CRITICAL",
    };

    (cri, level.to_string())
}

pub fn calculate_congestion_probability(cri: f64) -> f64 {
    round2((cri / 100.0_f64).powf(1.5) * 100.0)
}

pub fn calculate_correlation(xs: &[f64], ys: &[f64]) -> (f64, f64, f64) {
    let n = xs.len().min(ys.len());
    if n < 3 {
        return (0.0, 0.0, 1.0);
    }

    let n_f = n as f64;
    let mean_x: f64 = xs.iter().take(n).sum::<f64>() / n_f;
    let mean_y: f64 = ys.iter().take(n).sum::<f64>() / n_f;

    let mut sum_xy = 0.0;
    let mut sum_x2 = 0.0;
    let mut sum_y2 = 0.0;

    for i in 0..n {
        let dx = xs[i] - mean_x;
        let dy = ys[i] - mean_y;
        sum_xy += dx * dy;
        sum_x2 += dx * dx;
        sum_y2 += dy * dy;
    }

    if sum_x2 == 0.0 || sum_y2 == 0.0 {
        return (0.0, 0.0, 1.0);
    }

    let r = sum_xy / (sum_x2.sqrt() * sum_y2.sqrt());
    let r2 = r * r;

    // Approximate p-value using t-distribution
    let t = r * ((n_f - 2.0) / (1.0 - r2)).sqrt();
    let df = n_f - 2.0;
    // Simple approximation: p ≈ 2 * exp(-0.717 * t^2 / df) for large df
    let p = (2.0 * (-0.717 * t * t / df).exp()).clamp(0.0, 1.0);

    (round2(r), round2(r2), round3(p))
}

pub fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

fn round3(v: f64) -> f64 {
    (v * 1000.0).round() / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_residual_demand() {
        // load=20000MW, wind=5000MW, solar=2000MW -> residual=13.0 GW
        let r = calculate_residual_demand_gw(20000.0, 5000.0, 2000.0);
        assert!((r - 13.0).abs() < 0.01);
    }

    #[test]
    fn test_must_run_floor() {
        let mut gen = GenerationByType::default();
        gen.data.insert("B14".to_string(), 0.0); // no nuclear in Poland
        gen.data.insert("B02".to_string(), 8000.0); // lignite at 8GW
        let floor = calculate_must_run_floor_gw(&gen);
        // 0 + 8000 * 0.6 = 4800MW = 4.8GW
        assert!((floor - 4.8).abs() < 0.01);
    }

    #[test]
    fn test_cri_calculation() {
        // load=20000, residual=13000, must_run=4800, renewable=7000
        let (cri, level) = calculate_cri(20000.0, 13000.0, 4800.0, 7000.0);
        // grid_stress = 1 - (13000-4800)/20000 = 1 - 0.41 = 0.59
        // renewable_pen = 7000/20000 = 0.35
        // cri = 100 * 0.59 * 0.35 = 20.65
        assert!(cri > 15.0 && cri < 25.0);
        assert_eq!(level, "LOW");
    }

    #[test]
    fn test_correlation() {
        let xs = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let ys = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let (r, r2, p) = calculate_correlation(&xs, &ys);
        assert!((r - 1.0).abs() < 0.01); // perfect correlation
        assert!((r2 - 1.0).abs() < 0.01);
        assert!(p < 0.05);
    }

    #[test]
    fn test_congestion_probability() {
        let p = calculate_congestion_probability(74.2);
        assert!(p > 50.0 && p < 70.0);
    }

    #[test]
    fn test_parse_generation_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<GL_MarketDocument>
  <TimeSeries>
    <MktPSRType><psrType>B19</psrType></MktPSRType>
    <Period>
      <Point><position>1</position><quantity>3500</quantity></Point>
    </Period>
  </TimeSeries>
  <TimeSeries>
    <MktPSRType><psrType>B16</psrType></MktPSRType>
    <Period>
      <Point><position>1</position><quantity>1200</quantity></Point>
    </Period>
  </TimeSeries>
</GL_MarketDocument>"#;
        let gen = parse_generation_xml(xml).unwrap();
        assert!((gen.wind_mw() - 3500.0).abs() < 0.01);
        assert!((gen.solar_mw() - 1200.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_load_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<GL_MarketDocument>
  <TimeSeries>
    <Period>
      <Point><position>1</position><quantity>20500</quantity></Point>
      <Point><position>2</position><quantity>21000</quantity></Point>
    </Period>
  </TimeSeries>
</GL_MarketDocument>"#;
        let load = parse_load_xml(xml).unwrap();
        assert!((load - 21000.0).abs() < 0.01); // takes last value
    }
}
