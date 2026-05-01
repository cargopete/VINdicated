use async_trait::async_trait;
use std::time::Duration;
use crate::error::SourceError;
use crate::models::SourceData;
use crate::sources::VehicleSource;

pub struct UaHsc { client: reqwest::Client }
impl UaHsc { pub fn new(client: reqwest::Client) -> Self { Self { client } } }

#[async_trait]
impl VehicleSource for UaHsc {
    fn id(&self) -> &'static str { "ua_hsc" }
    fn country(&self) -> &'static str { "UA" }
    fn name(&self) -> &'static str { "HSC Open Data" }
    fn cache_ttl(&self) -> Duration { Duration::from_secs(60 * 60 * 24) }
    async fn fetch_by_vin(&self, _vin: &str) -> Result<SourceData, SourceError> {
        // TODO: implement HSC Open Data integration
        Err(SourceError::Unavailable("Not yet implemented".into()))
    }
}
