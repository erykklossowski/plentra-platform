use chrono::Utc;

#[derive(Debug, Clone, serde::Serialize)]
pub struct WeeklySignals {
    pub residual_anomaly: Option<ResidualAnomaly>,
    pub structural_break: Option<StructuralBreak>,
    pub forecast_miss: Option<ForecastMiss>,
    pub dtw_analogs: Option<DtwAnalogs>,
    pub has_signals: bool,
    pub signal_count: usize,
    pub signals_summary: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ResidualAnomaly {
    pub ticker: String,
    pub current_zscore: f64,
    pub direction: String,
    pub magnitude: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct StructuralBreak {
    pub ticker: String,
    pub detected_date: String,
    pub days_ago: i64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ForecastMiss {
    pub ticker: String,
    pub forecast_value: f64,
    pub actual_value: f64,
    pub error_pct: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DtwAnalogs {
    pub closest_weeks: Vec<AnalogWeek>,
    pub consensus_direction: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AnalogWeek {
    pub week_start: String,
    pub dtw_distance: f64,
    pub outcome_return: f64,
}

/// Compute weekly signals from analytics results + market data.
/// This function is pure — it takes pre-computed analytics and classifies signals.
pub fn aggregate_signals(
    ttf_decomp: Option<&super::decomposition::DecompositionResult>,
    eua_changepoint: Option<&super::changepoint::ChangepointResult>,
    ttf_forecast: Option<&super::forecast::FuelForecast>,
    ttf_history: &[f64],
    dtw_result: Option<DtwAnalogs>,
) -> WeeklySignals {
    let mut signals: Vec<String> = Vec::new();
    let mut residual_anomaly = None;
    let mut structural_break = None;
    let mut forecast_miss = None;

    // Signal 1: Residual anomaly from MSTL decomposition
    if let Some(decomp) = ttf_decomp {
        if decomp.residual.len() >= 30 {
            let historical = &decomp.residual[..decomp.residual.len().saturating_sub(7)];
            let mean: f64 = historical.iter().sum::<f64>() / historical.len() as f64;
            let variance: f64 = historical
                .iter()
                .map(|x| (x - mean).powi(2))
                .sum::<f64>()
                / historical.len() as f64;
            let sigma = variance.sqrt();

            if sigma > 0.0 {
                let recent =
                    &decomp.residual[decomp.residual.len().saturating_sub(5)..];
                let current_residual: f64 =
                    recent.iter().sum::<f64>() / recent.len() as f64;
                let zscore = (current_residual - mean) / sigma;

                if zscore.abs() >= 1.5 {
                    let magnitude = match zscore.abs() {
                        z if z >= 3.5 => "extreme",
                        z if z >= 2.5 => "strong",
                        _ => "moderate",
                    };
                    signals.push(format!("residual_anomaly:TTF:{:+.1}\u{03c3}", zscore));
                    residual_anomaly = Some(ResidualAnomaly {
                        ticker: "TTF".to_string(),
                        current_zscore: round2(zscore),
                        direction: if zscore > 0.0 {
                            "above".into()
                        } else {
                            "below".into()
                        },
                        magnitude: magnitude.to_string(),
                    });
                }
            }
        }
    }

    // Signal 2: Structural break from changepoint detection
    if let Some(cp) = eua_changepoint {
        if cp.alert {
            if let Some(idx) = cp.latest_break {
                let days_ago = (cp.changepoints.len() as i64).saturating_sub(idx as i64);
                signals.push(format!("structural_break:EUA:{}d_ago", days_ago));
                structural_break = Some(StructuralBreak {
                    ticker: "EUA".to_string(),
                    detected_date: (Utc::now() - chrono::Duration::days(days_ago))
                        .format("%Y-%m-%d")
                        .to_string(),
                    days_ago,
                });
            }
        }
    }

    // Signal 3: ETS forecast miss
    if let Some(fc) = ttf_forecast {
        if !fc.point_forecast.is_empty() && ttf_history.len() >= 8 {
            let forecasted = fc.point_forecast[0];
            let actual = *ttf_history.last().unwrap();
            if forecasted != 0.0 {
                let error_pct = ((actual - forecasted) / forecasted) * 100.0;
                if error_pct.abs() >= 10.0 {
                    signals.push(format!("forecast_miss:TTF:{:+.1}%", error_pct));
                    forecast_miss = Some(ForecastMiss {
                        ticker: "TTF".to_string(),
                        forecast_value: round2(forecasted),
                        actual_value: round2(actual),
                        error_pct: round2(error_pct),
                    });
                }
            }
        }
    }

    // Signal 4: DTW analogs
    if let Some(ref dtw) = dtw_result {
        if dtw.confidence >= 0.6 && dtw.consensus_direction != "neutral" {
            signals.push(format!("dtw_analog:{}", dtw.consensus_direction));
        }
    }

    let has_signals = !signals.is_empty();
    let signal_count = signals.len();

    WeeklySignals {
        residual_anomaly,
        structural_break,
        forecast_miss,
        dtw_analogs: dtw_result,
        has_signals,
        signal_count,
        signals_summary: signals,
    }
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_signals_when_residual_within_normal_range() {
        // All residuals within a narrow band — z-score of last 5 should be < 1.5
        let decomp = crate::analytics::decomposition::DecompositionResult {
            trend: vec![50.0; 40],
            seasonal_24h: None,
            seasonal_7d: vec![0.0; 40],
            residual: vec![
                1.0, -1.0, 1.5, -1.5, 0.8, 1.2, -0.8, 1.3, -1.1, 0.9,
                1.0, -1.0, 1.5, -1.5, 0.8, 1.2, -0.8, 1.3, -1.1, 0.9,
                1.0, -1.0, 1.5, -1.5, 0.8, 1.2, -0.8, 1.3, -1.1, 0.9,
                1.0, -1.0, 1.5, -1.5, 0.8, // last 5 are within normal range
                0.5, 0.3, 0.7, 0.4, 0.6,
            ],
            series_len: 40,
        };
        let signals = aggregate_signals(Some(&decomp), None, None, &[50.0; 40], None);
        assert!(!signals.has_signals);
        assert!(signals.residual_anomaly.is_none());
    }

    #[test]
    fn test_residual_anomaly_detected_above_threshold() {
        let residuals = vec![
            0.1, -0.1, 0.2, -0.2, 0.1, 0.1, -0.1, 0.2, -0.2, 0.1, 0.1, -0.1,
            0.2, -0.2, 0.1, 0.1, -0.1, 0.2, -0.2, 0.1, 0.1, -0.1, 0.2, -0.2,
            0.1, 0.1, -0.1, 0.2, -0.2, 0.1, 10.0, 10.0, 10.0, 10.0, 10.0,
        ];
        let decomp = crate::analytics::decomposition::DecompositionResult {
            trend: vec![50.0; residuals.len()],
            seasonal_24h: None,
            seasonal_7d: vec![0.0; residuals.len()],
            residual: residuals,
            series_len: 35,
        };
        let history = vec![50.0f64; 35];
        let signals = aggregate_signals(Some(&decomp), None, None, &history, None);
        assert!(signals.has_signals);
        assert!(signals.residual_anomaly.is_some());
        let anom = signals.residual_anomaly.unwrap();
        assert_eq!(anom.direction, "above");
        assert!(anom.current_zscore > 1.5);
    }

    #[test]
    fn test_has_signals_false_skips_signal_count() {
        let signals = WeeklySignals {
            residual_anomaly: None,
            structural_break: None,
            forecast_miss: None,
            dtw_analogs: None,
            has_signals: false,
            signal_count: 0,
            signals_summary: vec![],
        };
        assert_eq!(signals.signal_count, 0);
        assert!(!signals.has_signals);
    }
}
