use async_trait::async_trait;
use std::time::Duration;
use crate::error::SourceError;
use crate::models::SourceData;
use crate::sources::VehicleSource;

/// Slovakia — STKonline (technical inspection database)
/// Web-form based. No public JSON API path verified for VIN-based lookup.
pub struct SkStkonline {
    client: reqwest::Client,
}

impl SkStkonline {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VehicleSource for SkStkonline {
    fn id(&self) -> &'static str { "sk_stkonline" }
    fn country(&self) -> &'static str { "SK" }
    fn name(&self) -> &'static str { "STKonline (SK)" }
    fn cache_ttl(&self) -> Duration { Duration::from_secs(60 * 60 * 24) }

    async fn fetch_by_vin(&self, _vin: &str) -> Result<SourceData, SourceError> {
        Err(SourceError::Unavailable(
            "No verified public VIN API for Slovak STK database".into(),
        ))
    }
}
