use async_trait::async_trait;
use std::time::Duration;
use crate::error::SourceError;
use crate::models::SourceData;
use crate::sources::VehicleSource;

pub struct SiAvtolog { client: reqwest::Client }
impl SiAvtolog { pub fn new(client: reqwest::Client) -> Self { Self { client } } }

#[async_trait]
impl VehicleSource for SiAvtolog {
    fn id(&self) -> &'static str { "si_avtolog" }
    fn country(&self) -> &'static str { "SI" }
    fn name(&self) -> &'static str { "Avtolog" }
    fn cache_ttl(&self) -> Duration { Duration::from_secs(60 * 60 * 24) }
    async fn fetch_by_vin(&self, _vin: &str) -> Result<SourceData, SourceError> {
        // TODO: implement Avtolog integration
        Err(SourceError::Unavailable("Not yet implemented".into()))
    }
}
