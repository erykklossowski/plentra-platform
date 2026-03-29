use augurs::mstl::MSTLModel;
use augurs::{Fit, Predict};

pub struct DecompositionResult {
    pub trend: Vec<f64>,
    pub seasonal_24h: Option<Vec<f64>>,
    pub seasonal_7d: Vec<f64>,
    pub residual: Vec<f64>,
    pub series_len: usize,
}

/// Decompose daily time series into trend + weekly seasonal + residual.
/// Requires at least 14 observations.
pub fn decompose_daily(series: &[f64]) -> anyhow::Result<DecompositionResult> {
    if series.len() < 14 {
        return Err(anyhow::anyhow!(
            "Need at least 14 daily observations for MSTL, got {}",
            series.len()
        ));
    }

    let periods = vec![7]; // weekly seasonality
    let model = MSTLModel::naive(periods);
    let fit = model.fit(series)?;

    // Access decomposition via in-sample prediction
    // MSTL decomposes: series = trend + seasonal_7d + residual
    // We reconstruct from the fitted model's predictions
    let in_sample = fit.predict_in_sample(0.95)?;
    let predicted = &in_sample.point;

    // The residuals are series - predicted
    let residual: Vec<f64> = series
        .iter()
        .zip(predicted.iter())
        .map(|(y, p)| y - p)
        .collect();

    // For MSTL with naive trend, we approximate trend as a centered moving average
    let trend = moving_average(series, 7);

    // Seasonal = series - trend - residual (approximately)
    let seasonal_7d: Vec<f64> = series
        .iter()
        .zip(trend.iter())
        .zip(residual.iter())
        .map(|((y, t), r)| y - t - r)
        .collect();

    Ok(DecompositionResult {
        trend,
        seasonal_24h: None,
        seasonal_7d,
        residual,
        series_len: series.len(),
    })
}

/// Decompose hourly time series into trend + 24h seasonal + 7d seasonal + residual.
/// Requires at least 48 observations.
pub fn decompose_hourly(series: &[f64]) -> anyhow::Result<DecompositionResult> {
    if series.len() < 48 {
        return Err(anyhow::anyhow!(
            "Need at least 48 hourly observations for MSTL, got {}",
            series.len()
        ));
    }

    let periods = vec![24, 168]; // daily + weekly
    let model = MSTLModel::naive(periods);
    let fit = model.fit(series)?;

    let in_sample = fit.predict_in_sample(0.95)?;
    let predicted = &in_sample.point;

    let residual: Vec<f64> = series
        .iter()
        .zip(predicted.iter())
        .map(|(y, p)| y - p)
        .collect();

    let trend = moving_average(series, 24);

    // 24h seasonal component: subtract trend from series, then take mod-24 average
    let detrended: Vec<f64> = series
        .iter()
        .zip(trend.iter())
        .map(|(y, t)| y - t)
        .collect();

    let seasonal_24h = extract_seasonal(&detrended, 24);
    let seasonal_7d: Vec<f64> = series
        .iter()
        .zip(trend.iter())
        .zip(seasonal_24h.iter())
        .zip(residual.iter())
        .map(|(((y, t), s24), r)| y - t - s24 - r)
        .collect();

    Ok(DecompositionResult {
        trend,
        seasonal_24h: Some(seasonal_24h),
        seasonal_7d,
        residual,
        series_len: series.len(),
    })
}

/// Simple centered moving average for trend extraction.
fn moving_average(series: &[f64], window: usize) -> Vec<f64> {
    let n = series.len();
    let half = window / 2;
    (0..n)
        .map(|i| {
            let start = i.saturating_sub(half);
            let end = (i + half + 1).min(n);
            let sum: f64 = series[start..end].iter().sum();
            sum / (end - start) as f64
        })
        .collect()
}

/// Extract seasonal component by averaging values at each position mod period.
fn extract_seasonal(detrended: &[f64], period: usize) -> Vec<f64> {
    let mut sums = vec![0.0; period];
    let mut counts = vec![0usize; period];

    for (i, v) in detrended.iter().enumerate() {
        let pos = i % period;
        sums[pos] += v;
        counts[pos] += 1;
    }

    let means: Vec<f64> = sums
        .iter()
        .zip(counts.iter())
        .map(|(s, c)| if *c > 0 { s / *c as f64 } else { 0.0 })
        .collect();

    detrended
        .iter()
        .enumerate()
        .map(|(i, _)| means[i % period])
        .collect()
}

/// Compute DTW distance between two series using augurs-dtw.
pub fn dtw_distance(a: &[f64], b: &[f64]) -> f64 {
    use augurs::dtw::Dtw;
    Dtw::euclidean().distance(a, b)
}

/// Find top-N most similar historical weeks using DTW distance.
pub fn find_dtw_analogs(
    current_week: &[f64],
    historical_weeks: &[(String, Vec<f64>, f64)], // (week_start, features, next_week_return)
    top_n: usize,
) -> Option<super::signal_aggregator::DtwAnalogs> {
    if historical_weeks.len() < 4 || current_week.len() < 5 {
        return None;
    }

    let mut distances: Vec<(usize, f64)> = historical_weeks
        .iter()
        .enumerate()
        .filter_map(|(i, (_, features, _))| {
            if features.len() != current_week.len() {
                return None;
            }
            Some((i, dtw_distance(current_week, features)))
        })
        .collect();

    distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    distances.truncate(top_n);

    let analogs: Vec<super::signal_aggregator::AnalogWeek> = distances
        .iter()
        .map(|(i, dist)| {
            let (week_start, _, outcome) = &historical_weeks[*i];
            super::signal_aggregator::AnalogWeek {
                week_start: week_start.clone(),
                dtw_distance: round2(*dist),
                outcome_return: round2(*outcome),
            }
        })
        .collect();

    let bullish = analogs.iter().filter(|a| a.outcome_return > 1.0).count();
    let bearish = analogs.iter().filter(|a| a.outcome_return < -1.0).count();
    let total = analogs.len();

    let (direction, confidence) = if total == 0 {
        ("neutral", 0.5)
    } else if bullish > bearish && bullish as f64 / total as f64 >= 0.6 {
        ("bullish", bullish as f64 / total as f64)
    } else if bearish > bullish && bearish as f64 / total as f64 >= 0.6 {
        ("bearish", bearish as f64 / total as f64)
    } else {
        ("neutral", 0.5)
    };

    Some(super::signal_aggregator::DtwAnalogs {
        closest_weeks: analogs,
        consensus_direction: direction.to_string(),
        confidence: round2(confidence),
    })
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mstl_requires_minimum_data() {
        let short = vec![1.0, 2.0, 3.0];
        assert!(decompose_daily(&short).is_err());
    }

    #[test]
    fn test_mstl_decomposition_produces_components() {
        let series: Vec<f64> = (0..60)
            .map(|i| 100.0 + (i as f64 * 0.1).sin() * 10.0)
            .collect();
        let result = decompose_daily(&series).unwrap();
        assert_eq!(result.trend.len(), 60);
        assert_eq!(result.seasonal_7d.len(), 60);
        assert_eq!(result.residual.len(), 60);
        assert!(result.seasonal_24h.is_none());
        // Trend should be smooth (close to the input since input is smooth)
        assert!(result.trend[30] > 90.0 && result.trend[30] < 110.0);
    }

    #[test]
    fn test_hourly_requires_minimum_data() {
        let short = vec![1.0; 20];
        assert!(decompose_hourly(&short).is_err());
    }
}
