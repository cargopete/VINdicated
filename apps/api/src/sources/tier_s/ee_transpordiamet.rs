use async_trait::async_trait;
use std::time::Duration;

use crate::error::SourceError;
use crate::models::{Inspection, InspectionResult, Registration, RegistrationStatus, SourceData};
use crate::sources::VehicleSource;

/// Estonia — Transpordiamet public vehicle lookup
/// https://eteenindus.mnt.ee/public/soidukTaring.jsf
/// Returns registration data + technical inspection (ülevaatus) history with mileage.
pub struct EeTranspordiamet {
    client: reqwest::Client,
}

impl EeTranspordiamet {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VehicleSource for EeTranspordiamet {
    fn id(&self) -> &'static str {
        "ee_transpordiamet"
    }
    fn country(&self) -> &'static str {
        "EE"
    }
    fn name(&self) -> &'static str {
        "Transpordiamet (EE)"
    }
    fn cache_ttl(&self) -> Duration {
        Duration::from_secs(60 * 60 * 24)
    }

    async fn fetch_by_vin(&self, vin: &str) -> Result<SourceData, SourceError> {
        // Public JSON endpoint used by the Transpordiamet portal
        let url = format!(
            "https://eteenindus.mnt.ee/public/soidukTaringJson.jsf?vin={}",
            vin
        );
        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .header("Referer", "https://eteenindus.mnt.ee/")
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

        if body.get("regNr").is_none() && body.get("vin").is_none() {
            return Err(SourceError::NotFound);
        }

        let reg = Registration {
            country: "EE".into(),
            plate: body["regNr"].as_str().map(|s| s.to_string()),
            first_registered: body["firstRegDate"]
                .as_str()
                .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
            deregistered: None,
            status: match body["status"].as_str() {
                Some("REGISTERED") => RegistrationStatus::Active,
                Some("DEREGISTERED") => RegistrationStatus::Deregistered,
                Some("STOLEN") => RegistrationStatus::Stolen,
                _ => RegistrationStatus::Unknown,
            },
            color: body["color"].as_str().map(|s| s.to_string()),
            fuel: body["fuelType"].as_str().map(|s| s.to_string()),
            body: body["bodyType"].as_str().map(|s| s.to_string()),
            engine_cc: body["engineVolume"].as_u64().map(|v| v as u32),
            power_kw: body["powerKw"].as_u64().map(|v| v as u32),
            seats: body["seats"].as_u64().map(|v| v as u8),
            weight_kg: body["massInService"].as_u64().map(|v| v as u32),
            source: self.id().into(),
        };

        let empty = vec![];
        let tests = body["inspections"].as_array().unwrap_or(&empty);
        let inspections: Vec<Inspection> = tests
            .iter()
            .filter_map(|t| {
                let date_str = t["inspectionDate"].as_str()?;
                let date = chrono::NaiveDate::parse_from_str(&date_str[..10], "%Y-%m-%d").ok()?;

                let result = match t["result"].as_str() {
                    Some("PASSED") => InspectionResult::Pass,
                    Some("FAILED") => InspectionResult::Fail,
                    _ => InspectionResult::Unknown,
                };

                Some(Inspection {
                    country: "EE".into(),
                    date,
                    result,
                    mileage_km: t["odometer"].as_u64(),
                    defects: t["defects"]
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .filter_map(|d| d.as_str().map(|s| s.to_string()))
                        .collect(),
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
