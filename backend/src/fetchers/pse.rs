use chrono::Datelike;
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

pub async fn fetch_pse<T: DeserializeOwned>(
    client: &reqwest::Client,
    endpoint: &str,
    filter: &str,
    top: usize,
) -> anyhow::Result<Vec<T>> {
    let url = format!(
        "https://api.raporty.pse.pl/api/{}?$filter={}&$top={}",
        endpoint,
        urlencoding::encode(filter),
        top
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
