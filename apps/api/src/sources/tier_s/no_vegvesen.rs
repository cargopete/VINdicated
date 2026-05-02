use async_trait::async_trait;
use std::time::Duration;
use crate::error::SourceError;
use crate::models::SourceData;
use crate::sources::VehicleSource;

/// Norway — Statens vegvesen (vehicle register)
/// The public API at vegvesen.no returns HTTP 401 and requires a Bearer token
/// issued via the Maskinporten OAuth2 service (for registered organisations only).
pub struct NoVegvesen {
    client: reqwest::Client,
}

impl NoVegvesen {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VehicleSource for NoVegvesen {
    fn id(&self) -> &'static str { "no_vegvesen" }
    fn country(&self) -> &'static str { "NO" }
    fn name(&self) -> &'static str { "Vegvesen (NO)" }
    fn cache_ttl(&self) -> Duration { Duration::from_secs(60 * 60 * 24) }

    async fn fetch_by_vin(&self, _vin: &str) -> Result<SourceData, SourceError> {
        Err(SourceError::Unavailable(
            "Authentication required (Maskinporten Bearer token)".into(),
        ))
    }
}
