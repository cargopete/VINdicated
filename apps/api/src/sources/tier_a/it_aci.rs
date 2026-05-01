use async_trait::async_trait;
use std::time::Duration;
use crate::error::SourceError;
use crate::models::SourceData;
use crate::sources::VehicleSource;

pub struct ItAci { client: reqwest::Client }
impl ItAci { pub fn new(client: reqwest::Client) -> Self { Self { client } } }

#[async_trait]
impl VehicleSource for ItAci {
    fn id(&self) -> &'static str { "it_aci" }
    fn country(&self) -> &'static str { "IT" }
    fn name(&self) -> &'static str { "ACI Italy" }
    fn cache_ttl(&self) -> Duration { Duration::from_secs(60 * 60 * 24) }
    async fn fetch_by_vin(&self, _vin: &str) -> Result<SourceData, SourceError> {
        Err(SourceError::Unavailable("Not yet implemented".into()))
    }
}
