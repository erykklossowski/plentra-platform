use serde_json::json;

pub struct RetrospectiveInput {
    pub rdn_pln_mwh: f64,
    pub rdn_change_pct: f64,
    pub ttf_eur_mwh: f64,
    pub ttf_change_pct: f64,
    pub ara_usd_tonne: f64,
    pub ara_change_pct: f64,
    pub eua_eur_tonne: f64,
    pub eua_change_pct: f64,
    pub css_spot: f64,
    pub cds_spot_eta42: f64,
    pub dispatch_signal: String,
    pub current_residual_gw: f64,
    pub must_run_floor_gw: f64,
    pub cri_value: f64,
    pub cri_level: String,
    pub ytd_total_gwh: f64,
    pub ytd_wind_gwh: f64,
    pub ytd_solar_gwh: f64,
    pub ytd_network_gwh: f64,
    pub ytd_balance_gwh: f64,
    pub afrr_g_pln_mw: f64,
    pub mfrrd_g_pln_mw: f64,
}

pub fn build_retrospective_prompt(input: &RetrospectiveInput) -> String {
    let dispatch_text = match input.dispatch_signal.as_str() {
        "GAS_MARGINAL" => "gas-fired CCGT units are the marginal price-setter",
        "COAL_MARGINAL" => "coal units remain the marginal price-setter",
        _ => "renewables or imports are suppressing thermal dispatch",
    };

    let curtailment_cause = if input.ytd_network_gwh > input.ytd_balance_gwh {
        "primarily driven by grid congestion (network constraints)"
    } else {
        "primarily driven by system balancing needs"
    };

    format!(
        r#"You are a senior energy market analyst at Plentra Research,
a Polish boutique energy analytics firm. Write a concise market retrospective
(130–170 words) of the Polish wholesale electricity market for the current
period based on the live data below.

Requirements:
- Write in English
- Use specific numbers from the data (do not round excessively)
- Identify the dominant price driver in the first sentence
- Mention curtailment if YTD total > 50 GWh
- Reference reserve price level if aFRR_G > 150 PLN/MW (elevated)
- End with exactly one forward-looking sentence about the next 2–4 weeks
- No headers, no bullet points, no markdown — plain prose only

CURRENT MARKET DATA:
Spot electricity (RDN): {rdn:.1} PLN/MWh (MoM: {rdn_mom:+.1}%)
TTF gas spot: {ttf:.2} EUR/MWh (MoM: {ttf_mom:+.1}%)
ARA coal: {ara:.2} USD/t (MoM: {ara_mom:+.1}%)
EUA CO₂: {eua:.2} EUR/t (MoM: {eua_mom:+.1}%)
Clean Spark Spread: {css:+.2} EUR/MWh
Clean Dark Spread: {cds:+.2} EUR/MWh
Dispatch signal: {dispatch_text}
Residual demand: {residual:.1} GW (must-run floor: {must_run:.1} GW)
Curtailment Risk Index: {cri:.1} ({cri_level})
OZE curtailment YTD: {curtailment_ytd:.1} GWh ({curtailment_cause})
  — wind: {wind_gwh:.1} GWh, solar: {solar_gwh:.1} GWh
aFRR_G capacity price: {afrr_g:.1} PLN/MW
mFRRd_G capacity price: {mfrrd_g:.1} PLN/MW"#,
        rdn = input.rdn_pln_mwh,
        rdn_mom = input.rdn_change_pct,
        ttf = input.ttf_eur_mwh,
        ttf_mom = input.ttf_change_pct,
        ara = input.ara_usd_tonne,
        ara_mom = input.ara_change_pct,
        eua = input.eua_eur_tonne,
        eua_mom = input.eua_change_pct,
        css = input.css_spot,
        cds = input.cds_spot_eta42,
        dispatch_text = dispatch_text,
        residual = input.current_residual_gw,
        must_run = input.must_run_floor_gw,
        cri = input.cri_value,
        cri_level = input.cri_level,
        curtailment_ytd = input.ytd_total_gwh,
        curtailment_cause = curtailment_cause,
        wind_gwh = input.ytd_wind_gwh,
        solar_gwh = input.ytd_solar_gwh,
        afrr_g = input.afrr_g_pln_mw,
        mfrrd_g = input.mfrrd_g_pln_mw,
    )
}

pub async fn generate_retrospective(
    client: &reqwest::Client,
    prompt: String,
    api_key: &str,
) -> anyhow::Result<String> {
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&json!({
            "model": "claude-sonnet-4-6",
            "max_tokens": 1024,
            "messages": [{
                "role": "user",
                "content": prompt
            }]
        }))
        .send()
        .await?
        .error_for_status()?
        .json::<serde_json::Value>()
        .await?;

    let text = response["content"][0]["text"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No text in Claude response"))?
        .to_string();

    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retrospective_prompt_contains_key_data() {
        let input = RetrospectiveInput {
            rdn_pln_mwh: 499.2,
            rdn_change_pct: -20.5,
            ttf_eur_mwh: 34.5,
            ttf_change_pct: -5.2,
            ara_usd_tonne: 95.3,
            ara_change_pct: 2.1,
            eua_eur_tonne: 68.4,
            eua_change_pct: -1.8,
            css_spot: 12.45,
            cds_spot_eta42: -8.3,
            dispatch_signal: "GAS_MARGINAL".to_string(),
            current_residual_gw: 12.4,
            must_run_floor_gw: 5.4,
            cri_value: 74.2,
            cri_level: "ELEVATED".to_string(),
            ytd_total_gwh: 46.7,
            ytd_wind_gwh: 28.1,
            ytd_solar_gwh: 18.6,
            ytd_network_gwh: 20.0,
            ytd_balance_gwh: 26.7,
            afrr_g_pln_mw: 109.9,
            mfrrd_g_pln_mw: 78.2,
        };
        let prompt = build_retrospective_prompt(&input);
        assert!(prompt.contains("499.2"));
        assert!(prompt.contains("109.9"));
        assert!(prompt.contains("gas-fired CCGT"));
    }
}
