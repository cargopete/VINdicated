use async_trait::async_trait;
use std::time::Duration;
use crate::error::SourceError;
use crate::models::SourceData;
use crate::sources::VehicleSource;

pub struct EeTranspordiamet { client: reqwest::Client }
impl EeTranspordiamet { pub fn new(client: reqwest::Client) -> Self { Self { client } } }

#[async_trait]
impl VehicleSource for EeTranspordiamet {
    fn id(&self) -> &'static str { "ee_transpordiamet" }
    fn country(&self) -> &'static str { "EE" }
    fn name(&self) -> &'static str { "Transpordiamet" }
    fn cache_ttl(&self) -> Duration { Duration::from_secs(60 * 60 * 24) }
    async fn fetch_by_vin(&self, _vin: &str) -> Result<SourceData, SourceError> {
        // TODO: implement Transpordiamet integration
        Err(SourceError::Unavailable("Not yet implemented".into()))
    }
}
