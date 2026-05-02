use async_trait::async_trait;
use std::time::Duration;
use crate::error::SourceError;
use crate::models::SourceData;
use crate::sources::VehicleSource;

/// Sweden — Transportstyrelsen (Swedish Transport Agency)
/// The Fordonsfrågans API requires Swedish e-identification (BankID) or an API key
/// issued to authorised parties. The public endpoint returns 401 without credentials.
pub struct SeTransportstyrelsen {
    client: reqwest::Client,
}

impl SeTransportstyrelsen {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VehicleSource for SeTransportstyrelsen {
    fn id(&self) -> &'static str { "se_transportstyrelsen" }
    fn country(&self) -> &'static str { "SE" }
    fn name(&self) -> &'static str { "Transportstyrelsen (SE)" }
    fn cache_ttl(&self) -> Duration { Duration::from_secs(60 * 60 * 24) }

    async fn fetch_by_vin(&self, _vin: &str) -> Result<SourceData, SourceError> {
        Err(SourceError::Unavailable(
            "Authentication required (Swedish e-ID or API key)".into(),
        ))
    }
}
