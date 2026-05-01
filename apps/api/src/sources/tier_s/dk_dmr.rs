use async_trait::async_trait;
use std::time::Duration;
use crate::error::SourceError;
use crate::models::SourceData;
use crate::sources::VehicleSource;

pub struct DkDmr { client: reqwest::Client }
impl DkDmr { pub fn new(client: reqwest::Client) -> Self { Self { client } } }

#[async_trait]
impl VehicleSource for DkDmr {
    fn id(&self) -> &'static str { "dk_dmr" }
    fn country(&self) -> &'static str { "DK" }
    fn name(&self) -> &'static str { "DMR Motorregister" }
    fn cache_ttl(&self) -> Duration { Duration::from_secs(60 * 60 * 24) }
    async fn fetch_by_vin(&self, _vin: &str) -> Result<SourceData, SourceError> {
        // TODO: implement DMR Motorregister integration
        Err(SourceError::Unavailable("Not yet implemented".into()))
    }
}
