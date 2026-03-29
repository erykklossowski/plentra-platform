use augurs::changepoint::DefaultArgpcpDetector;

pub struct ChangepointResult {
    pub changepoints: Vec<usize>,
    pub latest_break: Option<usize>,
    pub alert: bool,
    pub alert_message: Option<String>,
}

/// Detect structural breaks in a time series using ARGPCP (Autoregressive
/// Gaussian Process Changepoint Detection).
/// Used to alert traders that historical correlations may no longer hold.
pub fn detect_changepoints(
    series: &[f64],
    series_dates: &[chrono::NaiveDate],
) -> anyhow::Result<ChangepointResult> {
    use augurs::changepoint::Detector;

    if series.len() < 20 {
        return Ok(ChangepointResult {
            changepoints: vec![],
            latest_break: None,
            alert: false,
            alert_message: None,
        });
    }

    let mut detector = DefaultArgpcpDetector::builder()
        .build();
    let changepoints = detector.detect_changepoints(series);

    // Filter out index 0 which is always included by the detector
    let changepoints: Vec<usize> = changepoints
        .into_iter()
        .filter(|&idx| idx > 0 && idx < series.len())
        .collect();

    let latest_break = changepoints.last().copied();
    let alert = latest_break
        .map(|idx| {
            series_dates.len() > idx
                && (chrono::Utc::now().date_naive() - series_dates[idx]).num_days() <= 14
        })
        .unwrap_or(false);

    let alert_message = if alert {
        latest_break
            .and_then(|idx| series_dates.get(idx))
            .map(|date| {
                format!(
                    "Structural break detected on {}. \
                     Historical correlations may not hold — model recalibration recommended.",
                    date.format("%d %b %Y")
                )
            })
    } else {
        None
    };

    Ok(ChangepointResult {
        changepoints,
        latest_break,
        alert,
        alert_message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_changepoint_empty_on_short_series() {
        let short = vec![1.0; 5];
        let dates: Vec<chrono::NaiveDate> = (0..5)
            .map(|i| {
                chrono::Utc::now().date_naive() - chrono::Duration::days(4 - i)
            })
            .collect();
        let result = detect_changepoints(&short, &dates).unwrap();
        assert!(!result.alert);
        assert!(result.changepoints.is_empty());
    }

    #[test]
    fn test_changepoint_stable_series_no_alert() {
        let stable: Vec<f64> = vec![50.0; 90];
        let dates: Vec<chrono::NaiveDate> = (0..90)
            .map(|i| {
                chrono::Utc::now().date_naive() - chrono::Duration::days(89 - i)
            })
            .collect();
        let result = detect_changepoints(&stable, &dates).unwrap();
        assert!(!result.alert);
    }
}
