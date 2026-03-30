// Common fields returned by all routes when data is unavailable
export interface DataStatusFields {
  data_status?: "unavailable";
  message?: string;
}

export interface FuelsResponse extends DataStatusFields {
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

export interface SpreadsResponse extends DataStatusFields {
  css_spot: number;
  css_spot_pct_change: number;
  cds_spot_eta34: number;
  cds_spot_eta42: number;
  cds_spot_pct_change: number;
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

export interface ForwardPrice {
  label: string;
  sublabel: string;
  value_eur_mwh: number | null;
  value_pln_mwh: number | null;
  change_pct: number | null;
  source: string;
  available: boolean;
}

export interface SummaryResponse extends DataStatusFields {
  retrospective_text: string;
  retrospective_generated_at?: string;
  retrospective_stale?: boolean;
  average_system_margin_pct: number;
  system_margin_signal: string;
  forward_signals: ForwardSignal[];
  key_indicators: KeyIndicator[];
  industrial_spread: IndustrialSpread;
  forward_prices: ForwardPrice[];
  model_insights?: string | null;
  model_insights_generated_at?: string | null;
  signal_count?: number;
  signals_summary?: string[];
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

// ─── Generation (Phase 3) ───

export interface JKZEntry {
  technology: string;
  efficiency: number;
  emission_factor: number;
  fuel_cost_eur_mwh: number;
  co2_cost_eur_mwh: number;
  jkz_eur_mwh: number;
  clean_spread_eur_mwh: number;
  dispatch_status: string;
}

export interface GenerationResponse extends DataStatusFields {
  jkz_table: JKZEntry[];
  dispatch_signal: string;
  css_spot: number;
  cds_spot_eta42: number;
  css_history_30d: number[];
  cds_history_30d: number[];
  eur_usd_rate: number;
  rdn_eur_mwh: number;
  fetched_at: string;
  stale?: boolean;
}

// ─── Cross-Border (Phase 4) ───

export interface CrossBorderHourly {
  hour: number;
  pl: number;
  de: number;
  spread: number;
}

export interface CrossBorderResponse extends DataStatusFields {
  pl_da_eur_mwh: number;
  de_da_eur_mwh: number;
  spread_eur_mwh: number;
  spread_direction: string;
  hourly_profile: CrossBorderHourly[];
  avg_spread_30d: number;
  flow_direction: string;
  interconnector_utilization_pct: number;
  fetched_at: string;
  stale?: boolean;
}

// ─── Europe (Phase 4) ───

export interface EURankingEntry {
  rank: number;
  country_code: string;
  country_name: string;
  da_price_eur_mwh: number;
  bar_pct: number;
  is_focus: boolean;
}

export interface ExtremePriceEntry {
  code: string;
  price: number;
}

export interface EuropeResponse extends DataStatusFields {
  rankings: EURankingEntry[];
  poland_rank: number;
  poland_price: number;
  eu_average: number;
  cheapest: ExtremePriceEntry;
  most_expensive: ExtremePriceEntry;
  fetched_at: string;
  stale?: boolean;
}

// ─── Curtailment (Phase 5b) ───

export interface CurtailmentHourly {
  hour: number;
  wind_balance_mwh: number;
  wind_network_mwh: number;
  pv_balance_mwh: number;
  pv_network_mwh: number;
  total_mwh: number;
}

export interface CurtailmentDaily {
  date: string;
  pv_balance_mwh: number;
  pv_network_mwh: number;
  wi_balance_mwh: number;
  wi_network_mwh: number;
  total_mwh: number;
}

export interface CurtailmentResponse extends DataStatusFields {
  today_total_mwh: number;
  today_wind_balance_mwh: number;
  today_wind_network_mwh: number;
  today_pv_balance_mwh: number;
  today_pv_network_mwh: number;
  ytd_total_gwh: number;
  ytd_wind_gwh: number;
  ytd_solar_gwh: number;
  ytd_network_gwh: number;
  ytd_balance_gwh: number;
  hourly_profile: CurtailmentHourly[];
  daily_30d: CurtailmentDaily[];
  is_estimate: boolean;
  source: string;
  fetched_at: string;
}

// ──��� Reserves (Phase 5b) ───

export interface ReservePrices {
  afrr_d_pln_mw: number;
  afrr_g_pln_mw: number;
  mfrrd_d_pln_mw: number;
  mfrrd_g_pln_mw: number;
  fcr_d_pln_mw: number;
  fcr_g_pln_mw: number;
  rr_g_pln_mw: number;
}

export interface ReserveMonthlyHistory {
  month: string;
  afrr_d: number;
  afrr_g: number;
  mfrrd_d: number;
  mfrrd_g: number;
  fcr_d: number;
  fcr_g: number;
  rr_g: number;
}

export interface ReserveDailyHistory {
  date: string;
  afrr_g: number;
  afrr_d: number;
  mfrrd_g: number;
  fcr_g: number;
}

export interface ReservesResponse extends DataStatusFields {
  date: string;
  prices: ReservePrices;
  daily_30d: ReserveDailyHistory[];
  history_13m: ReserveMonthlyHistory[];
  source: string;
  fetched_at: string;
}

// ─── Stability (Phase 2) ───

export interface ResidualResponse extends DataStatusFields {
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
  is_estimate: boolean;
  fetched_at: string;
  stale?: boolean;
}

// ─── Analytics (Prompt B) ───

export interface SpreadHistoryPoint {
  date: string;
  spread_type: string;
  value: number;
  rolling_7d_avg: number | null;
  rolling_30d_avg: number | null;
  rolling_30d_stddev: number | null;
}

export interface SeasonalityPoint {
  spread_type: string;
  month: string;
  min: number | null;
  q1: number | null;
  median: number | null;
  q3: number | null;
  max: number | null;
  mean: number | null;
  n_days: number;
}

export interface PositiveDaysPoint {
  month: string;
  spread_type: string;
  positive_days: number;
  total_days: number;
  positive_pct: number;
}

export interface SpreadsAnalyticsResponse {
  generated_at: string;
  history_90d: SpreadHistoryPoint[];
  seasonality: SeasonalityPoint[];
  positive_days: PositiveDaysPoint[];
}

export interface EveningDecompositionPoint {
  date: string;
  evening_avg_pln: number;
  baseline_pln: number;
  delta_fuel_pln: number;
  delta_oze_pln: number;
  residual_pln: number;
}

export interface EveningAnalyticsResponse {
  generated_at: string;
  days: number;
  constants: {
    eur_pln_rate: number;
    pass_through: number;
    oze_scale_pln_mwh: number;
  };
  summary: {
    avg_css_contribution_pct: number;
  };
  decomposition: EveningDecompositionPoint[];
}

export interface PsePricePoint {
  ts: string;
  value: number | null;
}

export interface PsePricesResponse {
  ticker: string;
  resolution: string;
  from: string;
  to: string;
  point_count: number;
  series: {
    cen: PsePricePoint[];
    ckoeb: PsePricePoint[];
    sdac: PsePricePoint[];
  };
  source: string;
}

// ─── Forecast (Phase 7) ───

export interface FuelForecastData {
  ticker: string;
  horizon_days: number;
  last_historical: number;
  training_points: number;
  point_forecast: number[];
  lower_80: number[];
  upper_80: number[];
  lower_95: number[];
  upper_95: number[];
}

export interface DecompositionData {
  ticker: string;
  series_len: number;
  trend: number[];
  seasonal_7d: number[];
  residual: number[];
}

export interface ChangepointAlert {
  ticker: string;
  alert: boolean;
  message: string;
  latest_break_index?: number;
}

export interface ForecastResponse extends DataStatusFields {
  generated_at?: string;
  fuel_forecasts?: {
    ttf?: FuelForecastData;
    ara?: FuelForecastData;
    eua?: FuelForecastData;
  };
  decomposition?: DecompositionData;
  changepoint_alerts?: ChangepointAlert;
  stale?: boolean;
}
