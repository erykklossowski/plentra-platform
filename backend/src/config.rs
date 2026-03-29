use std::env;

#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub database_url: Option<String>,
    pub entsoe_token: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub cache_ttl_fuels: u64,
    pub cache_ttl_entsoe: u64,
    pub allowed_origins: Vec<String>,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            port: env::var("PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8080),
            database_url: env::var("DATABASE_URL").ok(),
            entsoe_token: env::var("ENTSOE_TOKEN").ok(),
            anthropic_api_key: env::var("ANTHROPIC_API_KEY").ok(),
            cache_ttl_fuels: env::var("CACHE_TTL_FUELS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(900),
            cache_ttl_entsoe: env::var("CACHE_TTL_ENTSOE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3600),
            allowed_origins: env::var("ALLOWED_ORIGINS")
                .map(|v| v.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_else(|_| vec![
                    "http://localhost:3000".to_string(),
                    "https://plentra.vercel.app".to_string(),
                    "https://frontend-gamma-pink-76.vercel.app".to_string(),
                ]),
        }
    }
}
