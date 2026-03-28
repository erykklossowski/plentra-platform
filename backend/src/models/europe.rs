use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EURankingEntry {
    pub rank: u32,
    pub country_code: String,
    pub country_name: String,
    pub da_price_eur_mwh: f64,
    pub bar_pct: f64,
    pub is_focus: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtremePriceEntry {
    pub code: String,
    pub price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EuropeData {
    pub rankings: Vec<EURankingEntry>,
    pub poland_rank: u32,
    pub poland_price: f64,
    pub eu_average: f64,
    pub cheapest: ExtremePriceEntry,
    pub most_expensive: ExtremePriceEntry,
    pub fetched_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossBorderHourly {
    pub hour: u32,
    pub pl: f64,
    pub de: f64,
    pub spread: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossBorderData {
    pub pl_da_eur_mwh: f64,
    pub de_da_eur_mwh: f64,
    pub spread_eur_mwh: f64,
    pub spread_direction: String,
    pub hourly_profile: Vec<CrossBorderHourly>,
    pub avg_spread_30d: f64,
    pub flow_direction: String,
    pub interconnector_utilization_pct: f64,
    pub fetched_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale: Option<bool>,
}
