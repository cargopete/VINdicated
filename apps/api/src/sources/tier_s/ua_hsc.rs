use async_trait::async_trait;
use std::time::Duration;
use crate::error::SourceError;
use crate::models::SourceData;
use crate::sources::VehicleSource;

/// Ukraine — HSC (Humanitarian Service Centre) open data
/// opendata.hsc.gov.ua returns "Access Denied" (Akamai WAF) from non-Ukrainian IPs.
/// Geo-restricted; not accessible from VPS or other external infrastructure.
pub struct UaHsc {
    client: reqwest::Client,
}

impl UaHsc {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VehicleSource for UaHsc {
    fn id(&self) -> &'static str { "ua_hsc" }
    fn country(&self) -> &'static str { "UA" }
    fn name(&self) -> &'static str { "HSC Open Data (UA)" }
    fn cache_ttl(&self) -> Duration { Duration::from_secs(60 * 60 * 24) }

    async fn fetch_by_vin(&self, _vin: &str) -> Result<SourceData, SourceError> {
        Err(SourceError::Unavailable(
            "Geo-restricted — API blocked outside Ukraine".into(),
        ))
    }
}
