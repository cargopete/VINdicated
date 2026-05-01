use async_trait::async_trait;
use std::time::Duration;
use crate::error::SourceError;
use crate::models::SourceData;
use crate::sources::VehicleSource;

pub struct FrHistovec { client: reqwest::Client }
impl FrHistovec { pub fn new(client: reqwest::Client) -> Self { Self { client } } }

#[async_trait]
impl VehicleSource for FrHistovec {
    fn id(&self) -> &'static str { "fr_histovec" }
    fn country(&self) -> &'static str { "FR" }
    fn name(&self) -> &'static str { "HistoVec France" }
    fn cache_ttl(&self) -> Duration { Duration::from_secs(60 * 60 * 24 * 7) }
    async fn fetch_by_vin(&self, _vin: &str) -> Result<SourceData, SourceError> {
        Err(SourceError::Unavailable("Not yet implemented".into()))
    }
}
