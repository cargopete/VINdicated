use async_trait::async_trait;
use std::time::Duration;
use crate::error::SourceError;
use crate::models::SourceData;
use crate::sources::VehicleSource;

/// Spain — DGT / ITV (Inspección Técnica de Vehículos)
/// The DGT Informe Reducido requires plate + owner NIF. The ITV portal is web-form based.
/// Plate-based only; no public VIN API endpoint verified.
pub struct EsDgt {
    client: reqwest::Client,
}

impl EsDgt {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VehicleSource for EsDgt {
    fn id(&self) -> &'static str { "es_dgt" }
    fn country(&self) -> &'static str { "ES" }
    fn name(&self) -> &'static str { "DGT/ITV (ES)" }
    fn cache_ttl(&self) -> Duration { Duration::from_secs(60 * 60 * 24) }

    async fn fetch_by_vin(&self, _vin: &str) -> Result<SourceData, SourceError> {
        Err(SourceError::Unavailable(
            "Plate-based only; DGT requires plate + owner NIF".into(),
        ))
    }
}
