use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub redis_url: String,
    // UK
    pub dvla_api_key: Option<String>,
    pub dvsa_client_id: Option<String>,
    pub dvsa_client_secret: Option<String>,
    // NL
    pub rdw_app_token: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "8080".into())
                .parse()
                .context("Invalid PORT")?,
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".into()),
            dvla_api_key: std::env::var("DVLA_API_KEY").ok(),
            dvsa_client_id: std::env::var("DVSA_CLIENT_ID").ok(),
            dvsa_client_secret: std::env::var("DVSA_CLIENT_SECRET").ok(),
            rdw_app_token: std::env::var("RDW_APP_TOKEN").ok(),
        })
    }
}
