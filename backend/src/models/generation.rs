use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JKZEntry {
    pub technology: String,
    pub efficiency: f64,
    pub emission_factor: f64,
    pub fuel_cost_eur_mwh: f64,
    pub co2_cost_eur_mwh: f64,
    pub jkz_eur_mwh: f64,
    pub clean_spread_eur_mwh: f64,
    pub dispatch_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationData {
    pub jkz_table: Vec<JKZEntry>,
    pub dispatch_signal: String,
    pub css_spot: f64,
    pub cds_spot_eta42: f64,
    pub css_history_30d: Vec<f64>,
    pub cds_history_30d: Vec<f64>,
    pub eur_usd_rate: f64,
    pub rdn_eur_mwh: f64,
    pub fetched_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale: Option<bool>,
}
