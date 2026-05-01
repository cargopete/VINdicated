use async_trait::async_trait;
use std::time::Duration;
use crate::error::SourceError;
use crate::models::SourceData;
use crate::sources::VehicleSource;

pub struct LtRegitra { client: reqwest::Client }
impl LtRegitra { pub fn new(client: reqwest::Client) -> Self { Self { client } } }

#[async_trait]
impl VehicleSource for LtRegitra {
    fn id(&self) -> &'static str { "lt_regitra" }
    fn country(&self) -> &'static str { "LT" }
    fn name(&self) -> &'static str { "Regitra Lithuania" }
    fn cache_ttl(&self) -> Duration { Duration::from_secs(60 * 60 * 24) }
    async fn fetch_by_vin(&self, _vin: &str) -> Result<SourceData, SourceError> {
        Err(SourceError::Unavailable("Not yet implemented".into()))
    }
}
