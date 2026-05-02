use async_trait::async_trait;
use std::time::Duration;
use crate::error::SourceError;
use crate::models::SourceData;
use crate::sources::VehicleSource;

/// Poland — historiapojazdu.gov.pl (CEPIK vehicle register)
/// The portal is web-form based. The CEPIK API (api.cepik.gov.pl) requires
/// an API key issued to registered parties. No public VIN endpoint available.
pub struct PlHistoriaPojazdu {
    client: reqwest::Client,
}

impl PlHistoriaPojazdu {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VehicleSource for PlHistoriaPojazdu {
    fn id(&self) -> &'static str { "pl_historiapojazdu" }
    fn country(&self) -> &'static str { "PL" }
    fn name(&self) -> &'static str { "HistoriaPojazdu (PL)" }
    fn cache_ttl(&self) -> Duration { Duration::from_secs(60 * 60 * 24) }

    async fn fetch_by_vin(&self, _vin: &str) -> Result<SourceData, SourceError> {
        Err(SourceError::Unavailable(
            "CEPIK API requires registration; no public VIN endpoint".into(),
        ))
    }
}
