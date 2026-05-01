use async_trait::async_trait;
use std::time::Duration;
use crate::error::SourceError;
use crate::models::SourceData;
use crate::sources::VehicleSource;

pub struct SeTransportstyrelsen { client: reqwest::Client }
impl SeTransportstyrelsen { pub fn new(client: reqwest::Client) -> Self { Self { client } } }

#[async_trait]
impl VehicleSource for SeTransportstyrelsen {
    fn id(&self) -> &'static str { "se_transportstyrelsen" }
    fn country(&self) -> &'static str { "SE" }
    fn name(&self) -> &'static str { "Transportstyrelsen" }
    fn cache_ttl(&self) -> Duration { Duration::from_secs(60 * 60 * 24) }
    async fn fetch_by_vin(&self, _vin: &str) -> Result<SourceData, SourceError> {
        // TODO: implement Transportstyrelsen integration
        Err(SourceError::Unavailable("Not yet implemented".into()))
    }
}
