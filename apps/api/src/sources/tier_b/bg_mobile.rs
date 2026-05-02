use async_trait::async_trait;
use std::time::Duration;
use crate::error::SourceError;
use crate::models::SourceData;
use crate::sources::VehicleSource;

/// Bulgaria — IAMA (Infrastructure Agency) / KAT vehicle register
/// No public JSON API verified for VIN-based lookup. The portal is web-form based.
pub struct BgMobile {
    client: reqwest::Client,
}

impl BgMobile {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VehicleSource for BgMobile {
    fn id(&self) -> &'static str { "bg_mobile" }
    fn country(&self) -> &'static str { "BG" }
    fn name(&self) -> &'static str { "IAMA (BG)" }
    fn cache_ttl(&self) -> Duration { Duration::from_secs(60 * 60 * 24 * 7) }

    async fn fetch_by_vin(&self, _vin: &str) -> Result<SourceData, SourceError> {
        Err(SourceError::Unavailable(
            "No verified public VIN API for Bulgarian vehicle register".into(),
        ))
    }
}
