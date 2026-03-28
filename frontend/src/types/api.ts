export interface FuelsResponse {
  ttf_eur_mwh: number;
  ttf_change_pct: number;
  ttf_history_30d: number[];
  ara_usd_tonne: number;
  ara_change_pct: number;
  ara_history_30d: number[];
  eua_eur_tonne: number;
  eua_change_pct: number;
  eua_history_30d: number[];
  fetched_at: string;
  stale?: boolean;
}

export interface SpreadHistoryEntry {
  date: string;
  css: number;
  cds_42: number;
}

export interface SpreadsResponse {
  css_spot: number;
  css_spot_pct_change: number;
  cds_spot_eta34: number;
  cds_spot_eta42: number;
  css_term_y1: number;
  cds_term_y1: number | null;
  baseload_profitability_eur_mwh: number;
  peak_load_advantage_eur_mwh: number;
  carbon_impact_factor: number;
  dispatch_signal: string;
  history_30d: SpreadHistoryEntry[];
  fetched_at: string;
  stale?: boolean;
}

export interface ForwardSignal {
  commodity: string;
  direction: "UP" | "DOWN" | "FLAT";
  conviction: number;
  horizon: string;
}

export interface KeyIndicator {
  id: string;
  label: string;
  unit: string;
  spot: number;
  forward_m1: number;
  mom_delta_pct: number;
  spread_label: string;
  spread_value: number;
  spread_direction: "UP" | "DOWN";
}

export interface IndustrialSpread {
  baseload_eur_mwh: number;
  baseload_change_pct: number;
  peak_eur_mwh: number;
  peak_change_pct: number;
  carbon_impact_eur_mwh: number;
  carbon_change_pct: number;
}

export interface SummaryResponse {
  retrospective_text: string;
  average_system_margin_pct: number;
  system_margin_signal: string;
  forward_signals: ForwardSignal[];
  key_indicators: KeyIndicator[];
  industrial_spread: IndustrialSpread;
  fetched_at: string;
}

export interface HourlyProfile {
  hour: number;
  residual_gw: number;
  must_run_gw: number;
}

export interface HeatmapEntry {
  month: string;
  hour: number;
  value: number;
}

export interface ResidualResponse {
  current_residual_gw: number;
  must_run_floor_gw: number;
  stability_margin_gw: number;
  congestion_probability_pct: number;
  cri_value: number;
  cri_level: string;
  hourly_profile: HourlyProfile[];
  heatmap_data: HeatmapEntry[];
  ytd_curtailment_gwh: number;
  forecast_curtailment_gwh: number;
  wind_reduction_gwh: number;
  solar_reduction_gwh: number;
  correlation_r: number;
  correlation_r2: number;
  correlation_p: number;
  fetched_at: string;
}
