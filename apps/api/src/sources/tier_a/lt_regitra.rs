use async_trait::async_trait;
use std::time::Duration;

use crate::error::SourceError;
use crate::models::{Inspection, InspectionResult, Registration, RegistrationStatus, SourceData};
use crate::sources::VehicleSource;

/// Lithuania — Regitra (Road Transport Administration)
/// https://www.regitra.lt/
/// Free VIN-based lookup: registration status, technical inspection history with mileage.
pub struct LtRegitra {
    client: reqwest::Client,
}

impl LtRegitra {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VehicleSource for LtRegitra {
    fn id(&self) -> &'static str {
        "lt_regitra"
    }
    fn country(&self) -> &'static str {
        "LT"
    }
    fn name(&self) -> &'static str {
        "Regitra (LT)"
    }
    fn cache_ttl(&self) -> Duration {
        Duration::from_secs(60 * 60 * 24)
    }

    async fn fetch_by_vin(&self, vin: &str) -> Result<SourceData, SourceError> {
        let url = format!("https://www.regitra.lt/api/v1/vehicle/vin/{}", vin);
        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .header("Referer", "https://www.regitra.lt/")
            .send()
            .await?;

        match resp.status().as_u16() {
            200 => {}
            404 => return Err(SourceError::NotFound),
            429 => return Err(SourceError::RateLimited),
            s => return Err(SourceError::Unavailable(format!("HTTP {}", s))),
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| SourceError::Parse(e.to_string()))?;

        if body.is_null() || body.get("vin").is_none() {
            return Err(SourceError::NotFound);
        }

        let status = match body["registrationStatus"].as_str() {
            Some("REGISTERED") => RegistrationStatus::Active,
            Some("DEREGISTERED") => RegistrationStatus::Deregistered,
            Some("STOLEN") => RegistrationStatus::Stolen,
            Some("EXPORTED") => RegistrationStatus::Exported,
            _ => RegistrationStatus::Unknown,
        };

        let reg = Registration {
            country: "LT".into(),
            plate: body["plateNumber"].as_str().map(|s| s.to_string()),
            first_registered: body["firstRegistrationDate"]
                .as_str()
                .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
            deregistered: body["deregistrationDate"]
                .as_str()
                .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
            status,
            color: body["color"].as_str().map(|s| s.to_string()),
            fuel: body["fuelType"].as_str().map(|s| s.to_string()),
            body: body["bodyType"].as_str().map(|s| s.to_string()),
            engine_cc: body["engineCapacity"].as_u64().map(|v| v as u32),
            power_kw: body["powerKw"].as_u64().map(|v| v as u32),
            seats: body["seats"].as_u64().map(|v| v as u8),
            weight_kg: body["weight"].as_u64().map(|v| v as u32),
            source: self.id().into(),
        };

        let empty = vec![];
        let inspections_raw = body["technicalInspections"].as_array().unwrap_or(&empty);
        let inspections: Vec<Inspection> = inspections_raw
            .iter()
            .filter_map(|t| {
                let date_str = t["inspectionDate"].as_str()?;
                let date = chrono::NaiveDate::parse_from_str(&date_str[..10], "%Y-%m-%d").ok()?;
                let result = match t["result"].as_str() {
                    Some("PASSED") | Some("TEIGIAMAS") => InspectionResult::Pass,
                    Some("FAILED") | Some("NEIGIAMAS") => InspectionResult::Fail,
                    Some("CONDITIONAL") => InspectionResult::Advisory,
                    _ => InspectionResult::Unknown,
                };
                Some(Inspection {
                    country: "LT".into(),
                    date,
                    result,
                    mileage_km: t["odometer"].as_u64(),
                    defects: vec![],
                    advisories: vec![],
                    expiry: t["validUntil"]
                        .as_str()
                        .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
                    test_number: t["certificateNumber"].as_str().map(|s| s.to_string()),
                    source: self.id().into(),
                })
            })
            .collect();

        Ok(SourceData {
            registrations: vec![reg],
            inspections,
            ..Default::default()
        })
    }
}
