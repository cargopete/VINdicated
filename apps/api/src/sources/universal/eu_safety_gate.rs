use async_trait::async_trait;
use std::time::Duration;

use crate::error::SourceError;
use crate::models::SourceData;
use crate::sources::VehicleSource;

/// EU Safety Gate (ex-RAPEX) — model-level recall alerts.
/// VIN-keyed lookup is not possible; this source is used for
/// make/model enrichment after VIN decode.
pub struct EuSafetyGate {
    client: reqwest::Client,
}

impl EuSafetyGate {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VehicleSource for EuSafetyGate {
    fn id(&self) -> &'static str {
        "eu_safety_gate"
    }
    fn country(&self) -> &'static str {
        "EU"
    }
    fn name(&self) -> &'static str {
        "EU Safety Gate"
    }
    fn cache_ttl(&self) -> Duration {
        Duration::from_secs(60 * 60 * 24 * 7)
    }

    async fn fetch_by_vin(&self, _vin: &str) -> Result<SourceData, SourceError> {
        // EU Safety Gate does not support VIN-level lookup.
        // Model-level enrichment is done as a post-decode step.
        // Returning empty here; the aggregator handles model-level enrichment separately.
        Ok(SourceData::default())
    }
}
