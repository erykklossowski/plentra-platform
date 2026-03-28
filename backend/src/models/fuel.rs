use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuelData {
    pub ttf_eur_mwh: f64,
    pub ttf_change_pct: f64,
    pub ttf_history_30d: Vec<f64>,
    pub ara_usd_tonne: f64,
    pub ara_change_pct: f64,
    pub ara_history_30d: Vec<f64>,
    pub eua_eur_tonne: f64,
    pub eua_change_pct: f64,
    pub eua_history_30d: Vec<f64>,
    pub fetched_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale: Option<bool>,
}
