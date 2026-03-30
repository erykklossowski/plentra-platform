use chrono::{Datelike, NaiveDate};
use sqlx::PgPool;

// --- Symbol generation --------------------------------------------------------

/// Generate an ICE raw symbol for a given parent, month offset, and base date.
///
/// Format: "{PARENT} FM{MONTH_CODE}0{YY}!"
///
/// Examples (base = 2026-03-29):
///   get_ice_symbol("TFM", 1) -> "TFM FMJ0026!"  (April 2026)
///   get_ice_symbol("GAB", 2) -> "GAB FMK0026!"  (May 2026)
///   get_ice_symbol("TFM", 3) -> "TFM FMM0026!"  (June 2026)
pub fn get_ice_symbol(parent: &str, months_forward: u32, base: NaiveDate) -> String {
    let target = add_months(base, months_forward);
    let month_code =
        month_to_ice_code(target.month()).expect("month 1-12 always maps to an ICE code");
    let year_2d = target.year() % 100;
    format!("{} FM{}{:04}!", parent, month_code, year_2d)
}

/// ECF always uses the December contract for the current calendar year.
/// get_ecf_symbol(2026-03-29) -> "ECF FMZ0026!"
pub fn get_ecf_symbol(base: NaiveDate) -> String {
    let year_2d = base.year() % 100;
    format!("ECF FMZ{:04}!", year_2d)
}

fn month_to_ice_code(month: u32) -> Option<char> {
    match month {
        1 => Some('F'),
        2 => Some('G'),
        3 => Some('H'),
        4 => Some('J'),
        5 => Some('K'),
        6 => Some('M'),
        7 => Some('N'),
        8 => Some('Q'),
        9 => Some('U'),
        10 => Some('V'),
        11 => Some('X'),
        12 => Some('Z'),
        _ => None,
    }
}

/// Add N calendar months to a date.
/// Day is clamped to the last valid day of the resulting month.
fn add_months(date: NaiveDate, months: u32) -> NaiveDate {
    let total = date.month0() + months;
    let year = date.year() + (total / 12) as i32;
    let month = (total % 12) + 1;
    let max_day = days_in_month(year, month);
    NaiveDate::from_ymd_opt(year, month, date.day().min(max_day)).unwrap()
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let (y, m) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    NaiveDate::from_ymd_opt(y, m, 1)
        .unwrap()
        .pred_opt()
        .unwrap()
        .day()
}

// --- CSS formula --------------------------------------------------------------

/// Efficiency and emission parameters for a CCGT plant.
const ETA: f64 = 0.50; // thermal efficiency
const EMISSION_FACTOR: f64 = 0.202; // tCO2/MWh of gas

/// Rolling 3-month Clean Spark Spread.
///
/// CSS = Power_avg - Gas_avg/eta - Carbon * emission_factor / eta
///
/// All inputs in native units:
///   power_prices: EUR/MWh (German baseload)
///   gas_prices:   EUR/MWh (TTF)
///   carbon_price: EUR/t   (EUA)
/// Output: EUR/MWh
pub fn calculate_css(power_prices: &[f64; 3], gas_prices: &[f64; 3], carbon_price: f64) -> f64 {
    let power_avg = power_prices.iter().sum::<f64>() / 3.0;
    let gas_avg = gas_prices.iter().sum::<f64>() / 3.0;
    let carbon_cost = carbon_price * EMISSION_FACTOR / ETA;
    power_avg - (gas_avg / ETA) - carbon_cost
}

// --- Orchestrator -------------------------------------------------------------

/// Fetch the 7 required close prices from fuel_ohlcv for a given date,
/// compute CSS, and persist the result to calculated_spreads.
///
/// Returns Err if any of the 7 prices is missing — no silent fallbacks.
pub async fn run_css(pool: &PgPool, calc_date: NaiveDate) -> anyhow::Result<f64> {
    let gas_syms: [String; 3] = [
        get_ice_symbol("TFM", 1, calc_date),
        get_ice_symbol("TFM", 2, calc_date),
        get_ice_symbol("TFM", 3, calc_date),
    ];
    let power_syms: [String; 3] = [
        get_ice_symbol("GAB", 1, calc_date),
        get_ice_symbol("GAB", 2, calc_date),
        get_ice_symbol("GAB", 3, calc_date),
    ];
    let carbon_sym = get_ecf_symbol(calc_date);

    let all_syms: Vec<String> = gas_syms
        .iter()
        .chain(power_syms.iter())
        .chain(std::iter::once(&carbon_sym))
        .cloned()
        .collect();

    tracing::debug!(
        "CSS {}: gas={:?} power={:?} carbon={}",
        calc_date,
        gas_syms,
        power_syms,
        carbon_sym
    );

    // Fetch close prices from fuel_ohlcv.
    let rows = sqlx::query_as::<_, (String, f64, NaiveDate)>(
        r#"
        SELECT DISTINCT ON (raw_symbol)
            raw_symbol,
            close,
            date
        FROM fuel_ohlcv
        WHERE raw_symbol = ANY($1)
          AND date <= $2
        ORDER BY raw_symbol, date DESC
        "#,
    )
    .bind(&all_syms)
    .bind(calc_date)
    .fetch_all(pool)
    .await?;

    // Build symbol -> (date, close) map
    let price_map: std::collections::HashMap<String, (NaiveDate, f64)> = rows
        .into_iter()
        .map(|(sym, close, date)| (sym, (date, close)))
        .collect();

    // Verify all 7 symbols have prices
    let missing: Vec<&String> = all_syms
        .iter()
        .filter(|s| !price_map.contains_key(s.as_str()))
        .collect();

    if !missing.is_empty() {
        anyhow::bail!(
            "CSS aborted for {}: no price data for {:?}",
            calc_date,
            missing
        );
    }

    // Warn if prices come from different dates (stale data for some symbols)
    let dates: std::collections::HashSet<NaiveDate> =
        price_map.values().map(|(d, _)| *d).collect();
    if dates.len() > 1 {
        tracing::warn!(
            "CSS {}: prices from different dates: {:?}. Using latest per symbol.",
            calc_date,
            dates
        );
    }

    let gas_prices = [
        price_map[gas_syms[0].as_str()].1,
        price_map[gas_syms[1].as_str()].1,
        price_map[gas_syms[2].as_str()].1,
    ];
    let power_prices = [
        price_map[power_syms[0].as_str()].1,
        price_map[power_syms[1].as_str()].1,
        price_map[power_syms[2].as_str()].1,
    ];
    let carbon_price = price_map[carbon_sym.as_str()].1;

    let css = calculate_css(&power_prices, &gas_prices, carbon_price);
    let power_avg = power_prices.iter().sum::<f64>() / 3.0;
    let gas_avg = gas_prices.iter().sum::<f64>() / 3.0;

    tracing::info!(
        "CSS {}: {:.4} EUR/MWh (power_avg={:.4}, gas_avg={:.4}, carbon={:.4})",
        calc_date,
        css,
        power_avg,
        gas_avg,
        carbon_price
    );

    crate::db::writer::upsert_spread(
        pool,
        calc_date,
        "rolling_3m_css",
        css,
        power_avg,
        gas_avg,
        carbon_price,
        &power_syms.to_vec(),
        &gas_syms.to_vec(),
        &carbon_sym,
    )
    .await?;

    Ok(css)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn d(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    // Symbol generation — verified against real instrument data
    #[test]
    fn test_tfm_april_from_march() {
        assert_eq!(get_ice_symbol("TFM", 1, d(2026, 3, 29)), "TFM FMJ0026!");
    }
    #[test]
    fn test_tfm_may_from_march() {
        assert_eq!(get_ice_symbol("TFM", 2, d(2026, 3, 29)), "TFM FMK0026!");
    }
    #[test]
    fn test_gab_june_from_march() {
        assert_eq!(get_ice_symbol("GAB", 3, d(2026, 3, 29)), "GAB FMM0026!");
    }
    #[test]
    fn test_ecf_always_december() {
        assert_eq!(get_ecf_symbol(d(2026, 3, 29)), "ECF FMZ0026!");
        assert_eq!(get_ecf_symbol(d(2026, 11, 15)), "ECF FMZ0026!");
    }
    #[test]
    fn test_year_rollover() {
        // November + 2 months = January 2027
        assert_eq!(
            get_ice_symbol("TFM", 2, d(2026, 11, 15)),
            "TFM FMF0027!"
        );
    }

    // CSS formula — verified against known values from the document
    #[test]
    fn test_css_formula() {
        // GAB: 91.26, 88.37, 95.18  -> avg = 91.603
        // TFM: 52.20, 52.60, 52.34  -> avg = 52.380
        // ECF: 71.46
        // CSS = 91.603 - 52.380/0.50 - 71.46*0.202/0.50
        //     = 91.603 - 104.760 - 28.870 = -42.027
        let css = calculate_css(&[91.26, 88.37, 95.18], &[52.20, 52.60, 52.34], 71.46);
        assert!(
            (css - (-42.027)).abs() < 0.01,
            "Expected ~-42.027, got {:.3}",
            css
        );
    }
}
