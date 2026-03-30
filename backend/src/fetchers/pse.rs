use chrono::{Datelike, NaiveDate, TimeZone, Utc};
use chrono_tz::Europe::Warsaw;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;

const PSE_UA: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 \
    (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36";

pub fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

// ─── Record schemas ───

#[derive(Debug, Deserialize, Clone)]
pub struct PozRedozeRecord {
    pub dtime: String,
    #[allow(dead_code)]
    pub period: String,
    pub business_date: String,
    pub pv_red_balance: Option<f64>,
    pub pv_red_network: Option<f64>,
    pub wi_red_balance: Option<f64>,
    pub wi_red_network: Option<f64>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ReservePriceRecord {
    #[allow(dead_code)]
    pub dtime: String,
    pub business_date: String,
    pub fcr_d: Option<f64>,
    pub fcr_g: Option<f64>,
    pub afrr_d: Option<f64>,
    pub afrr_g: Option<f64>,
    pub mfrrd_d: Option<f64>,
    pub mfrrd_g: Option<f64>,
    pub rr_g: Option<f64>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct ReserveVolumeRecord {
    pub dtime: String,
    pub business_date: String,
    pub afrr_g: Option<f64>,
    pub afrr_d: Option<f64>,
    pub mfrrd_g: Option<f64>,
    pub mfrrd_d: Option<f64>,
    pub fcr_g: Option<f64>,
    pub fcr_d: Option<f64>,
    pub rr_g: Option<f64>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct KseDailyRecord {
    pub dtime: String,
    pub business_date: String,
    pub zapotrzebowanie: Option<f64>,
    pub wi: Option<f64>,
    pub pv: Option<f64>,
}

// ─── Aggregation output ───

#[derive(Debug, Clone, Default, Serialize)]
pub struct DailyCurtailment {
    pub date: String,
    pub pv_balance_mwh: f64,
    pub pv_network_mwh: f64,
    pub wi_balance_mwh: f64,
    pub wi_network_mwh: f64,
    pub total_mwh: f64,
}

// ─── Generic PSE fetch ───

/// Fetch PSE data for a single filter expression (max ~100 records returned).
pub async fn fetch_pse<T: DeserializeOwned>(
    client: &reqwest::Client,
    endpoint: &str,
    filter: &str,
) -> anyhow::Result<Vec<T>> {
    let url = format!(
        "https://api.raporty.pse.pl/api/{}?$filter={}",
        endpoint,
        urlencoding::encode(filter),
    );
    let resp: serde_json::Value = client
        .get(&url)
        .header("User-Agent", PSE_UA)
        .header("Accept", "application/json")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let records = serde_json::from_value(resp["value"].clone())?;
    Ok(records)
}

/// Fetch PSE data for a date range, one day at a time (PSE returns max 100 records per request).
pub async fn fetch_pse_date_range<T: DeserializeOwned + Send + 'static>(
    client: &reqwest::Client,
    endpoint: &str,
    start_date: &str,
    end_date: &str,
) -> Vec<T> {
    let start = chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d");
    let end = chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d");

    let (start, end) = match (start, end) {
        (Ok(s), Ok(e)) => (s, e),
        _ => return vec![],
    };

    let mut all_records: Vec<T> = Vec::new();
    let mut date = start;

    while date <= end {
        let date_str = date.to_string();
        let filter = format!(
            "business_date ge '{}' and business_date le '{}'",
            date_str, date_str
        );
        match fetch_pse::<T>(client, endpoint, &filter).await {
            Ok(records) => all_records.extend(records),
            Err(e) => {
                tracing::warn!("PSE fetch failed for {}/{}: {}", endpoint, date_str, e);
            }
        }
        date += chrono::Duration::days(1);
    }

    all_records
}

// ─── Aggregation functions ───

pub fn aggregate_curtailment_daily(
    records: &[PozRedozeRecord],
    date: &str,
) -> DailyCurtailment {
    let day: Vec<&PozRedozeRecord> = records
        .iter()
        .filter(|r| r.business_date == date)
        .collect();

    let sum = |f: fn(&PozRedozeRecord) -> Option<f64>| -> f64 {
        day.iter().map(|r| f(r).unwrap_or(0.0) * 0.25).sum()
    };

    DailyCurtailment {
        date: date.to_string(),
        pv_balance_mwh: round2(sum(|r| r.pv_red_balance)),
        pv_network_mwh: round2(sum(|r| r.pv_red_network)),
        wi_balance_mwh: round2(sum(|r| r.wi_red_balance)),
        wi_network_mwh: round2(sum(|r| r.wi_red_network)),
        total_mwh: round2(
            sum(|r| r.pv_red_balance)
                + sum(|r| r.pv_red_network)
                + sum(|r| r.wi_red_balance)
                + sum(|r| r.wi_red_network),
        ),
    }
}

pub fn estimate_ytd_gwh(daily_30d: &[DailyCurtailment]) -> f64 {
    let sum_30d_mwh: f64 = daily_30d.iter().map(|d| d.total_mwh).sum();
    let day_of_year = chrono::Utc::now().ordinal() as f64;
    round2(sum_30d_mwh * (day_of_year / 30.0) / 1000.0)
}

pub fn estimate_ytd_gwh_field(
    daily_30d: &[DailyCurtailment],
    field: fn(&DailyCurtailment) -> f64,
) -> f64 {
    let sum_30d_mwh: f64 = daily_30d.iter().map(field).sum();
    let day_of_year = chrono::Utc::now().ordinal() as f64;
    round2(sum_30d_mwh * (day_of_year / 30.0) / 1000.0)
}

pub fn daily_avg_reserve_price(
    records: &[ReservePriceRecord],
    date: &str,
    field: fn(&ReservePriceRecord) -> Option<f64>,
) -> f64 {
    let values: Vec<f64> = records
        .iter()
        .filter(|r| r.business_date == date)
        .filter_map(field)
        .collect();
    if values.is_empty() {
        return 0.0;
    }
    round2(values.iter().sum::<f64>() / values.len() as f64)
}

pub fn aggregate_to_hourly(records: &[PozRedozeRecord]) -> Vec<serde_json::Value> {
    let mut hours: std::collections::BTreeMap<u8, (f64, f64, f64, f64)> =
        std::collections::BTreeMap::new();

    for r in records {
        let hour: u8 = r.dtime.get(11..13).and_then(|s| s.parse().ok()).unwrap_or(0);
        let entry = hours.entry(hour).or_insert((0.0, 0.0, 0.0, 0.0));
        entry.0 += r.wi_red_balance.unwrap_or(0.0) * 0.25;
        entry.1 += r.wi_red_network.unwrap_or(0.0) * 0.25;
        entry.2 += r.pv_red_balance.unwrap_or(0.0) * 0.25;
        entry.3 += r.pv_red_network.unwrap_or(0.0) * 0.25;
    }

    hours
        .iter()
        .map(|(hour, (wib, win, pvb, pvn))| {
            json!({
                "hour": hour,
                "wind_balance_mwh": round2(*wib),
                "wind_network_mwh": round2(*win),
                "pv_balance_mwh": round2(*pvb),
                "pv_network_mwh": round2(*pvn),
                "total_mwh": round2(wib + win + pvb + pvn),
            })
        })
        .collect()
}

pub fn build_monthly_avg_history(
    records: &[ReservePriceRecord],
) -> Vec<serde_json::Value> {
    let mut months: std::collections::BTreeMap<String, Vec<&ReservePriceRecord>> =
        std::collections::BTreeMap::new();

    for r in records {
        if r.business_date.len() >= 7 {
            let month = r.business_date[..7].to_string();
            months.entry(month).or_default().push(r);
        }
    }

    months
        .iter()
        .map(|(month, recs)| {
            let avg_field = |f: fn(&ReservePriceRecord) -> Option<f64>| -> f64 {
                let vals: Vec<f64> = recs.iter().filter_map(|r| f(r)).collect();
                if vals.is_empty() {
                    return 0.0;
                }
                round2(vals.iter().sum::<f64>() / vals.len() as f64)
            };
            json!({
                "month": month,
                "afrr_d": avg_field(|r| r.afrr_d),
                "afrr_g": avg_field(|r| r.afrr_g),
                "mfrrd_d": avg_field(|r| r.mfrrd_d),
                "mfrrd_g": avg_field(|r| r.mfrrd_g),
                "fcr_d": avg_field(|r| r.fcr_d),
                "fcr_g": avg_field(|r| r.fcr_g),
                "rr_g": avg_field(|r| r.rr_g),
            })
        })
        .collect()
}

// ─── Energy prices (CEN, CKOEB, SDAC) ───

#[derive(Debug, Deserialize)]
struct PseEnergyPricesResponse {
    value: Vec<PseEnergyPriceRecord>,
}

#[derive(Debug, Deserialize)]
struct PseEnergyPriceRecord {
    doba:       String,       // "YYYY-MM-DD" — trading day
    godzina:    u32,          // 1..24 — WARSAW LOCAL TIME, not UTC
    cen_rozl:   Option<f64>,  // CEN settlement price PLN/MWh
    ckoeb_rozl: Option<f64>,  // CKOEB balancing market PLN/MWh
    csdac_pln:  Option<f64>,  // SDAC DA coupling PLN/MWh
}

/// One hour of Polish electricity prices.
#[derive(Debug, Clone)]
pub struct PseHourlyPrice {
    pub ts:        chrono::DateTime<Utc>,  // UTC — converted from Warsaw local
    pub cen_pln:   Option<f64>,
    pub ckoeb_pln: Option<f64>,
    pub csdac_pln: Option<f64>,
}

/// Fetch CEN, CKOEB and SDAC hourly prices from PSE energy-prices.
/// Returns one record per hour for the given date range.
/// Single HTTP request regardless of date range size.
pub async fn fetch_energy_prices(
    client:     &reqwest::Client,
    start_date: NaiveDate,
    end_date:   NaiveDate,
) -> anyhow::Result<Vec<PseHourlyPrice>> {
    let url = format!(
        "https://api.raporty.pse.pl/api/energy-prices\
         ?$filter=doba ge '{}' and doba le '{}'\
         &$orderby=doba asc,godzina asc\
         &$select=doba,godzina,cen_rozl,ckoeb_rozl,csdac_pln\
         &$top=9000",
        start_date.format("%Y-%m-%d"),
        end_date.format("%Y-%m-%d"),
    );

    tracing::info!(
        "PSE energy-prices: fetching {} → {}",
        start_date, end_date
    );

    let resp = client
        .get(&url)
        .header("User-Agent", PSE_UA)
        .header("Accept", "application/json")
        .send()
        .await?
        .error_for_status()?
        .json::<PseEnergyPricesResponse>()
        .await?;

    tracing::info!(
        "PSE energy-prices: {} records received",
        resp.value.len()
    );

    let mut records: Vec<PseHourlyPrice> = Vec::with_capacity(resp.value.len());

    for r in resp.value {
        let date = NaiveDate::parse_from_str(&r.doba, "%Y-%m-%d")
            .map_err(|e| anyhow::anyhow!("PSE: invalid date '{}': {}", r.doba, e))?;

        // PSE godzina is 1-indexed Warsaw local time.
        // godzina=1 → 00:00 Warsaw, godzina=24 → 23:00 Warsaw.
        let local_hour = match r.godzina.checked_sub(1) {
            Some(h) => h,
            None => {
                tracing::warn!("PSE: invalid godzina=0 for {}, skipping", r.doba);
                continue;
            }
        };

        let naive_local = match date.and_hms_opt(local_hour, 0, 0) {
            Some(dt) => dt,
            None => {
                tracing::debug!(
                    "PSE: skipping non-existent local time {}T{:02}:00 (DST gap)",
                    r.doba, local_hour
                );
                continue;
            }
        };

        // Convert Warsaw local → UTC using chrono-tz
        let ts_utc = match Warsaw.from_local_datetime(&naive_local) {
            chrono::LocalResult::Single(dt) => dt.with_timezone(&Utc),
            chrono::LocalResult::Ambiguous(dt, _) => dt.with_timezone(&Utc),
            chrono::LocalResult::None => {
                tracing::debug!(
                    "PSE: skipping invalid Warsaw time {}T{:02}:00",
                    r.doba, local_hour
                );
                continue;
            }
        };

        records.push(PseHourlyPrice {
            ts:        ts_utc,
            cen_pln:   r.cen_rozl,
            ckoeb_pln: r.ckoeb_rozl,
            csdac_pln: r.csdac_pln,
        });
    }

    tracing::info!(
        "PSE energy-prices: {} records after timezone conversion",
        records.len()
    );

    Ok(records)
}

// ─── PSE base URL constant ───

// ─── Date helpers ───

pub fn today_warsaw() -> String {
    chrono::Utc::now()
        .with_timezone(&chrono_tz::Europe::Warsaw)
        .date_naive()
        .to_string()
}

pub fn date_days_ago(days: i64) -> String {
    (chrono::Utc::now() - chrono::Duration::days(days))
        .date_naive()
        .to_string()
}

pub fn thirteen_months_ago() -> String {
    let now = chrono::Utc::now().date_naive();
    let target = now - chrono::Duration::days(395);
    target.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_curtailment_aggregation_null_handling() {
        let records = vec![PozRedozeRecord {
            dtime: "2026-03-01 00:15:00".to_string(),
            period: "00:00 - 00:15".to_string(),
            business_date: "2026-03-01".to_string(),
            pv_red_balance: None,
            pv_red_network: Some(50.0),
            wi_red_balance: Some(100.0),
            wi_red_network: None,
        }];
        let agg = aggregate_curtailment_daily(&records, "2026-03-01");
        assert_eq!(agg.pv_balance_mwh, 0.0);
        assert!((agg.pv_network_mwh - 12.5).abs() < 0.01);
        assert!((agg.wi_balance_mwh - 25.0).abs() < 0.01);
        assert_eq!(agg.wi_network_mwh, 0.0);
        assert!((agg.total_mwh - 37.5).abs() < 0.01);
    }

    #[test]
    fn test_ytd_scaling() {
        let daily: Vec<DailyCurtailment> = (0..30)
            .map(|_| DailyCurtailment {
                date: "2026-03-01".to_string(),
                total_mwh: 10_000.0,
                ..Default::default()
            })
            .collect();
        let ytd = estimate_ytd_gwh(&daily);
        assert!(ytd > 0.0);
    }

    #[test]
    fn test_reserve_monthly_avg() {
        let records = vec![
            ReservePriceRecord {
                dtime: "2026-03-01 00:00:00".to_string(),
                business_date: "2026-03-01".to_string(),
                afrr_g: Some(120.0),
                afrr_d: Some(80.0),
                mfrrd_g: Some(90.0),
                mfrrd_d: Some(60.0),
                fcr_g: Some(100.0),
                fcr_d: Some(70.0),
                rr_g: Some(95.0),
            },
            ReservePriceRecord {
                dtime: "2026-03-01 01:00:00".to_string(),
                business_date: "2026-03-01".to_string(),
                afrr_g: Some(140.0),
                ..Default::default()
            },
        ];
        let avg = daily_avg_reserve_price(&records, "2026-03-01", |r| r.afrr_g);
        assert!((avg - 130.0).abs() < 0.01);
    }
}
