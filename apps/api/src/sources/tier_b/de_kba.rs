use async_trait::async_trait;
use std::time::Duration;
use crate::error::SourceError;
use crate::models::SourceData;
use crate::sources::VehicleSource;

pub struct DeKba { client: reqwest::Client }
impl DeKba { pub fn new(client: reqwest::Client) -> Self { Self { client } } }

#[async_trait]
impl VehicleSource for DeKba {
    fn id(&self) -> &'static str { "de_kba" }
    fn country(&self) -> &'static str { "DE" }
    fn name(&self) -> &'static str { "KBA Germany" }
    fn cache_ttl(&self) -> Duration { Duration::from_secs(60 * 60 * 24 * 7) }
    async fn fetch_by_vin(&self, _vin: &str) -> Result<SourceData, SourceError> {
        Err(SourceError::Unavailable("Not yet implemented".into()))
    }
}
