use async_trait::async_trait;
use std::time::Duration;
use crate::error::SourceError;
use crate::models::SourceData;
use crate::sources::VehicleSource;

/// Estonia — Transpordiamet (Transport Administration)
/// The eteenindus.mnt.ee portal is web-form based. Tested VIN paths return 404.
/// No public JSON API endpoint verified.
pub struct EeTranspordiamet {
    client: reqwest::Client,
}

impl EeTranspordiamet {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VehicleSource for EeTranspordiamet {
    fn id(&self) -> &'static str { "ee_transpordiamet" }
    fn country(&self) -> &'static str { "EE" }
    fn name(&self) -> &'static str { "Transpordiamet (EE)" }
    fn cache_ttl(&self) -> Duration { Duration::from_secs(60 * 60 * 24) }

    async fn fetch_by_vin(&self, _vin: &str) -> Result<SourceData, SourceError> {
        Err(SourceError::Unavailable(
            "No verified public VIN API for Estonian Transport Administration".into(),
        ))
    }
}
