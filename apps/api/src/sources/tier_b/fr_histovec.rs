use async_trait::async_trait;
use std::time::Duration;
use crate::error::SourceError;
use crate::models::SourceData;
use crate::sources::VehicleSource;

/// France — HistoVec (Ministère de l'Intérieur)
/// The full "Rapport Complet" requires the vehicle owner to generate a share token.
/// The public API does not support anonymous VIN-only lookups.
pub struct FrHistovec {
    client: reqwest::Client,
}

impl FrHistovec {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VehicleSource for FrHistovec {
    fn id(&self) -> &'static str { "fr_histovec" }
    fn country(&self) -> &'static str { "FR" }
    fn name(&self) -> &'static str { "HistoVec (FR)" }
    fn cache_ttl(&self) -> Duration { Duration::from_secs(60 * 60 * 24 * 7) }

    async fn fetch_by_vin(&self, _vin: &str) -> Result<SourceData, SourceError> {
        Err(SourceError::Unavailable(
            "Owner share token required; anonymous VIN lookup not supported".into(),
        ))
    }
}
