use augurs::ets::AutoETS;
use augurs::mstl::MSTLModel;
use augurs::{Fit, Predict};

pub struct FuelForecast {
    pub ticker: String,
    pub horizon_days: usize,
    pub point_forecast: Vec<f64>,
    pub lower_80: Vec<f64>,
    pub upper_80: Vec<f64>,
    pub lower_95: Vec<f64>,
    pub upper_95: Vec<f64>,
    pub last_historical: f64,
    pub training_points: usize,
}

/// Fit ETS model via MSTL and produce N-day ahead forecast with intervals.
/// Input: daily close prices for ticker, oldest first, min 30 values.
pub fn forecast_fuel_ets(
    ticker: &str,
    history: &[f64],
    horizon_days: usize,
) -> anyhow::Result<FuelForecast> {
    if history.len() < 30 {
        return Err(anyhow::anyhow!(
            "ETS needs >=30 observations, got {} for {}",
            history.len(),
            ticker
        ));
    }

    // Use MSTL with ETS trend model for forecasting
    let ets = AutoETS::non_seasonal().into_trend_model();
    let periods = vec![7]; // weekly seasonality for daily data
    let model = MSTLModel::new(periods, ets);
    let fit = model.fit(history)?;

    // Get 95% prediction intervals
    let forecast_95 = fit.predict(horizon_days, 0.95)?;
    // Get 80% prediction intervals
    let forecast_80 = fit.predict(horizon_days, 0.80)?;

    let point_forecast: Vec<f64> = forecast_95.point.iter().map(|v| round2(*v)).collect();

    let (lower_95, upper_95) = match &forecast_95.intervals {
        Some(intervals) => (
            intervals.lower.iter().map(|v| round2(*v)).collect(),
            intervals.upper.iter().map(|v| round2(*v)).collect(),
        ),
        None => (vec![], vec![]),
    };

    let (lower_80, upper_80) = match &forecast_80.intervals {
        Some(intervals) => (
            intervals.lower.iter().map(|v| round2(*v)).collect(),
            intervals.upper.iter().map(|v| round2(*v)).collect(),
        ),
        None => (vec![], vec![]),
    };

    Ok(FuelForecast {
        ticker: ticker.to_string(),
        horizon_days,
        point_forecast,
        lower_80,
        upper_80,
        lower_95,
        upper_95,
        last_historical: *history.last().unwrap_or(&0.0),
        training_points: history.len(),
    })
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ets_forecast_requires_minimum_data() {
        let short: Vec<f64> = (0..10).map(|i| i as f64).collect();
        assert!(forecast_fuel_ets("TTF", &short, 7).is_err());
    }

    #[test]
    fn test_ets_forecast_length() {
        let history: Vec<f64> = (0..60)
            .map(|i| 50.0 + (i as f64 * 0.05).sin() * 5.0)
            .collect();
        let result = forecast_fuel_ets("TTF", &history, 14).unwrap();
        assert_eq!(result.point_forecast.len(), 14);
        assert_eq!(result.horizon_days, 14);
    }
}
