use async_trait::async_trait;
use std::time::Duration;
use crate::error::SourceError;
use crate::models::SourceData;
use crate::sources::VehicleSource;

pub struct LvCsdd { client: reqwest::Client }
impl LvCsdd { pub fn new(client: reqwest::Client) -> Self { Self { client } } }

#[async_trait]
impl VehicleSource for LvCsdd {
    fn id(&self) -> &'static str { "lv_csdd" }
    fn country(&self) -> &'static str { "LV" }
    fn name(&self) -> &'static str { "CSDD" }
    fn cache_ttl(&self) -> Duration { Duration::from_secs(60 * 60 * 24) }
    async fn fetch_by_vin(&self, _vin: &str) -> Result<SourceData, SourceError> {
        // TODO: implement CSDD integration
        Err(SourceError::Unavailable("Not yet implemented".into()))
    }
}
