use async_trait::async_trait;
use std::time::Duration;
use crate::error::SourceError;
use crate::models::SourceData;
use crate::sources::VehicleSource;

pub struct PlHistoriaPojazdu { client: reqwest::Client }
impl PlHistoriaPojazdu { pub fn new(client: reqwest::Client) -> Self { Self { client } } }

#[async_trait]
impl VehicleSource for PlHistoriaPojazdu {
    fn id(&self) -> &'static str { "pl_historiapojazdu" }
    fn country(&self) -> &'static str { "PL" }
    fn name(&self) -> &'static str { "HistoriaPojazdu" }
    fn cache_ttl(&self) -> Duration { Duration::from_secs(60 * 60 * 24) }
    async fn fetch_by_vin(&self, _vin: &str) -> Result<SourceData, SourceError> {
        // TODO: implement HistoriaPojazdu integration
        Err(SourceError::Unavailable("Not yet implemented".into()))
    }
}
