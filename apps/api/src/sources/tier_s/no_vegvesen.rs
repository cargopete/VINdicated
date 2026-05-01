use async_trait::async_trait;
use std::time::Duration;
use crate::error::SourceError;
use crate::models::SourceData;
use crate::sources::VehicleSource;

pub struct NoVegvesen { client: reqwest::Client }
impl NoVegvesen { pub fn new(client: reqwest::Client) -> Self { Self { client } } }

#[async_trait]
impl VehicleSource for NoVegvesen {
    fn id(&self) -> &'static str { "no_vegvesen" }
    fn country(&self) -> &'static str { "NO" }
    fn name(&self) -> &'static str { "Statens vegvesen" }
    fn cache_ttl(&self) -> Duration { Duration::from_secs(60 * 60 * 24) }
    async fn fetch_by_vin(&self, _vin: &str) -> Result<SourceData, SourceError> {
        // TODO: implement Statens vegvesen integration
        Err(SourceError::Unavailable("Not yet implemented".into()))
    }
}
