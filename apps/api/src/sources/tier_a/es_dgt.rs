use async_trait::async_trait;
use std::time::Duration;
use crate::error::SourceError;
use crate::models::SourceData;
use crate::sources::VehicleSource;

pub struct EsDgt { client: reqwest::Client }
impl EsDgt { pub fn new(client: reqwest::Client) -> Self { Self { client } } }

#[async_trait]
impl VehicleSource for EsDgt {
    fn id(&self) -> &'static str { "es_dgt" }
    fn country(&self) -> &'static str { "ES" }
    fn name(&self) -> &'static str { "DGT Spain" }
    fn cache_ttl(&self) -> Duration { Duration::from_secs(60 * 60 * 24) }
    async fn fetch_by_vin(&self, _vin: &str) -> Result<SourceData, SourceError> {
        Err(SourceError::Unavailable("Not yet implemented".into()))
    }
}
