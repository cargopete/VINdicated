use async_trait::async_trait;
use std::time::Duration;
use crate::error::SourceError;
use crate::models::SourceData;
use crate::sources::VehicleSource;

/// Iceland — Samgöngustofa (Transport Authority of Iceland)
/// samgongustofa.is is web-form based. No public JSON API endpoint verified for VIN lookup.
pub struct IsSamgongustofa {
    client: reqwest::Client,
}

impl IsSamgongustofa {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VehicleSource for IsSamgongustofa {
    fn id(&self) -> &'static str { "is_samgongustofa" }
    fn country(&self) -> &'static str { "IS" }
    fn name(&self) -> &'static str { "Samgöngustofa (IS)" }
    fn cache_ttl(&self) -> Duration { Duration::from_secs(60 * 60 * 24) }

    async fn fetch_by_vin(&self, _vin: &str) -> Result<SourceData, SourceError> {
        Err(SourceError::Unavailable(
            "No verified public VIN API for Icelandic Samgöngustofa".into(),
        ))
    }
}
