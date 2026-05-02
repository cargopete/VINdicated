use async_trait::async_trait;
use std::time::Duration;
use crate::error::SourceError;
use crate::models::SourceData;
use crate::sources::VehicleSource;

/// Slovenia — Avtolog / AJPES vehicle register
/// Web-form based. No public JSON API path verified for VIN-based lookup.
pub struct SiAvtolog {
    client: reqwest::Client,
}

impl SiAvtolog {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VehicleSource for SiAvtolog {
    fn id(&self) -> &'static str { "si_avtolog" }
    fn country(&self) -> &'static str { "SI" }
    fn name(&self) -> &'static str { "Avtolog (SI)" }
    fn cache_ttl(&self) -> Duration { Duration::from_secs(60 * 60 * 24) }

    async fn fetch_by_vin(&self, _vin: &str) -> Result<SourceData, SourceError> {
        Err(SourceError::Unavailable(
            "No verified public VIN API for Slovenian vehicle register".into(),
        ))
    }
}
