use async_trait::async_trait;
use std::time::Duration;
use crate::error::SourceError;
use crate::models::SourceData;
use crate::sources::VehicleSource;

pub struct CzKontrolaTachometru { client: reqwest::Client }
impl CzKontrolaTachometru { pub fn new(client: reqwest::Client) -> Self { Self { client } } }

#[async_trait]
impl VehicleSource for CzKontrolaTachometru {
    fn id(&self) -> &'static str { "cz_kontrolatachometru" }
    fn country(&self) -> &'static str { "CZ" }
    fn name(&self) -> &'static str { "Kontrola Tachometru" }
    fn cache_ttl(&self) -> Duration { Duration::from_secs(60 * 60 * 24) }
    async fn fetch_by_vin(&self, _vin: &str) -> Result<SourceData, SourceError> {
        // TODO: implement Kontrola Tachometru integration
        Err(SourceError::Unavailable("Not yet implemented".into()))
    }
}
