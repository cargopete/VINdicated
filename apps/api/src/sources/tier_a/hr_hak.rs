use async_trait::async_trait;
use std::time::Duration;
use crate::error::SourceError;
use crate::models::SourceData;
use crate::sources::VehicleSource;

pub struct HrHak { client: reqwest::Client }
impl HrHak { pub fn new(client: reqwest::Client) -> Self { Self { client } } }

#[async_trait]
impl VehicleSource for HrHak {
    fn id(&self) -> &'static str { "hr_hak" }
    fn country(&self) -> &'static str { "HR" }
    fn name(&self) -> &'static str { "HAK/CVH Croatia" }
    fn cache_ttl(&self) -> Duration { Duration::from_secs(60 * 60 * 24) }
    async fn fetch_by_vin(&self, _vin: &str) -> Result<SourceData, SourceError> {
        Err(SourceError::Unavailable("Not yet implemented".into()))
    }
}
