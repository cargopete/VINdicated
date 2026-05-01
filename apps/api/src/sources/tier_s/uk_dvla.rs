use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::error::SourceError;
use crate::models::{Registration, RegistrationStatus, SourceData};
use crate::sources::VehicleSource;

/// DVLA Vehicle Enquiry Service (UK)
/// https://developer-portal.driver-vehicle-licensing.api.gov.uk/
pub struct UkDvla {
    client: reqwest::Client,
    api_key: Option<String>,
}

impl UkDvla {
    pub fn new(client: reqwest::Client, api_key: Option<String>) -> Self {
        Self { client, api_key }
    }
}

#[derive(Debug, Serialize)]
struct VesRequest<'a> {
    #[serde(rename = "registrationNumber")]
    registration_number: &'a str,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VesResponse {
    registration_number: Option<String>,
    make: Option<String>,
    fuel_type: Option<String>,
    primary_colour: Option<String>,
    engine_capacity: Option<u32>,
    month_of_first_registration: Option<String>,
    year_of_manufacture: Option<u16>,
    tax_status: Option<String>,
    mot_status: Option<String>,
    marked_for_export: Option<bool>,
}

#[async_trait]
impl VehicleSource for UkDvla {
    fn id(&self) -> &'static str {
        "uk_dvla"
    }
    fn country(&self) -> &'static str {
        "GB"
    }
    fn name(&self) -> &'static str {
        "DVLA Vehicle Enquiry"
    }
    fn cache_ttl(&self) -> Duration {
        Duration::from_secs(60 * 60 * 24)
    }

    async fn fetch_by_vin(&self, vin: &str) -> Result<SourceData, SourceError> {
        let key = match &self.api_key {
            Some(k) => k.clone(),
            None => return Err(SourceError::Unavailable("No DVLA API key configured".into())),
        };

        // DVLA VES takes plate, not VIN — VIN lookup is indirect.
        // We return Skipped here; plate-based lookup is handled separately.
        // TODO: cross-reference DVSA MOT data to find plate from VIN, then query DVLA.
        let _ = vin;
        let _ = key;
        Err(SourceError::NotSupported)
    }
}

/// Fetch by UK registration plate (primary use case for DVLA).
impl UkDvla {
    pub async fn fetch_by_plate(&self, plate: &str) -> Result<SourceData, SourceError> {
        let key = match &self.api_key {
            Some(k) => k.clone(),
            None => return Err(SourceError::Unavailable("No DVLA API key configured".into())),
        };

        let body = VesRequest {
            registration_number: plate,
        };

        let resp = self
            .client
            .post("https://driver-vehicle-licensing.api.gov.uk/vehicle-enquiry/v1/vehicles")
            .header("x-api-key", &key)
            .json(&body)
            .send()
            .await?;

        match resp.status().as_u16() {
            200 => {}
            404 => return Err(SourceError::NotFound),
            429 => return Err(SourceError::RateLimited),
            s => return Err(SourceError::Unavailable(format!("HTTP {}", s))),
        }

        let v: VesResponse = resp
            .json()
            .await
            .map_err(|e| SourceError::Parse(e.to_string()))?;

        let first_registered = v
            .month_of_first_registration
            .as_deref()
            .and_then(|s| chrono::NaiveDate::parse_from_str(&format!("{}-01", s), "%Y-%m-%d").ok());

        let status = if v.marked_for_export.unwrap_or(false) {
            RegistrationStatus::Exported
        } else {
            RegistrationStatus::Active
        };

        let reg = Registration {
            country: "GB".into(),
            plate: v.registration_number,
            first_registered,
            deregistered: None,
            status,
            color: v.primary_colour,
            fuel: v.fuel_type,
            body: None,
            engine_cc: v.engine_capacity,
            power_kw: None,
            seats: None,
            weight_kg: None,
            source: self.id().into(),
        };

        Ok(SourceData {
            registrations: vec![reg],
            ..Default::default()
        })
    }
}
