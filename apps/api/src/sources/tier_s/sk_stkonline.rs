use async_trait::async_trait;
use std::time::Duration;
use crate::error::SourceError;
use crate::models::SourceData;
use crate::sources::VehicleSource;

pub struct SkStkOnline { client: reqwest::Client }
impl SkStkOnline { pub fn new(client: reqwest::Client) -> Self { Self { client } } }

#[async_trait]
impl VehicleSource for SkStkOnline {
    fn id(&self) -> &'static str { "sk_stkonline" }
    fn country(&self) -> &'static str { "SK" }
    fn name(&self) -> &'static str { "STKonline" }
    fn cache_ttl(&self) -> Duration { Duration::from_secs(60 * 60 * 24) }
    async fn fetch_by_vin(&self, _vin: &str) -> Result<SourceData, SourceError> {
        // TODO: implement STKonline integration
        Err(SourceError::Unavailable("Not yet implemented".into()))
    }
}
