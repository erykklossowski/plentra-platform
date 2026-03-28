use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyProfileEntry {
    pub hour: u32,
    pub residual_gw: f64,
    pub must_run_gw: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatmapEntry {
    pub month: String,
    pub hour: u32,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidualData {
    pub current_residual_gw: f64,
    pub must_run_floor_gw: f64,
    pub stability_margin_gw: f64,
    pub congestion_probability_pct: f64,
    pub cri_value: f64,
    pub cri_level: String,
    pub hourly_profile: Vec<HourlyProfileEntry>,
    pub heatmap_data: Vec<HeatmapEntry>,
    pub ytd_curtailment_gwh: f64,
    pub forecast_curtailment_gwh: f64,
    pub wind_reduction_gwh: f64,
    pub solar_reduction_gwh: f64,
    pub correlation_r: f64,
    pub correlation_r2: f64,
    pub correlation_p: f64,
    pub fetched_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale: Option<bool>,
}
