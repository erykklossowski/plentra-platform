use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadHistoryEntry {
    pub date: String,
    pub css: f64,
    pub cds_42: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadData {
    pub css_spot: f64,
    pub css_spot_pct_change: f64,
    pub cds_spot_eta34: f64,
    pub cds_spot_eta42: f64,
    pub cds_spot_pct_change: f64,
    pub css_term_y1: f64,
    pub cds_term_y1: Option<f64>,
    pub baseload_profitability_eur_mwh: f64,
    pub peak_load_advantage_eur_mwh: f64,
    pub carbon_impact_factor: f64,
    pub dispatch_signal: String,
    pub history_30d: Vec<SpreadHistoryEntry>,
    pub fetched_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale: Option<bool>,
}
