use async_trait::async_trait;
use std::time::Duration;
use crate::error::SourceError;
use crate::models::SourceData;
use crate::sources::VehicleSource;

/// Netherlands — RDW Open Data (Socrata)
/// The public dataset (m9d7-ebf2) is keyed by registration plate (kenteken), not VIN.
/// There is no chassisnummer column in the open dataset; VIN-based lookup is unavailable.
/// A plate-keyed lookup path (kenteken → APK history) could be added once we have a
/// cross-reference mechanism from VIN to plate.
pub struct NlRdw {
    client: reqwest::Client,
}

impl NlRdw {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VehicleSource for NlRdw {
    fn id(&self) -> &'static str { "nl_rdw" }
    fn country(&self) -> &'static str { "NL" }
    fn name(&self) -> &'static str { "RDW Open Data" }
    fn cache_ttl(&self) -> Duration { Duration::from_secs(60 * 60 * 24) }

    async fn fetch_by_vin(&self, _vin: &str) -> Result<SourceData, SourceError> {
        Err(SourceError::Unavailable(
            "RDW public dataset is plate-keyed; no VIN column available".into(),
        ))
    }
}
