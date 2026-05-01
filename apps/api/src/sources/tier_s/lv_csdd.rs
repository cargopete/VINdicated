use async_trait::async_trait;
use std::time::Duration;

use crate::error::SourceError;
use crate::models::{Inspection, InspectionResult, Registration, RegistrationStatus, SourceData};
use crate::sources::VehicleSource;

/// Latvia — CSDD vehicle lookup
/// https://e.csdd.lv/
/// Public data: registration + technical inspection history with mileage.
/// Also exposes cross-border mileage data from EE, NL, SK, DE imports.
pub struct LvCsdd {
    client: reqwest::Client,
}

impl LvCsdd {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VehicleSource for LvCsdd {
    fn id(&self) -> &'static str {
        "lv_csdd"
    }
    fn country(&self) -> &'static str {
        "LV"
    }
    fn name(&self) -> &'static str {
        "CSDD (LV)"
    }
    fn cache_ttl(&self) -> Duration {
        Duration::from_secs(60 * 60 * 24)
    }

    async fn fetch_by_vin(&self, vin: &str) -> Result<SourceData, SourceError> {
        let url = format!("https://e.csdd.lv/api/vehicle/vin/{}", vin);
        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .header("Referer", "https://e.csdd.lv/")
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

        let reg = Registration {
            country: "LV".into(),
            plate: body["regNumber"].as_str().map(|s| s.to_string()),
            first_registered: body["firstRegistrationDate"]
                .as_str()
                .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
            deregistered: None,
            status: match body["status"].as_str() {
                Some("ACTIVE") => RegistrationStatus::Active,
                Some("DEREGISTERED") => RegistrationStatus::Deregistered,
                Some("STOLEN") => RegistrationStatus::Stolen,
                _ => RegistrationStatus::Unknown,
            },
            color: body["color"].as_str().map(|s| s.to_string()),
            fuel: body["fuel"].as_str().map(|s| s.to_string()),
            body: body["bodyType"].as_str().map(|s| s.to_string()),
            engine_cc: body["engineCapacity"].as_u64().map(|v| v as u32),
            power_kw: body["powerKw"].as_u64().map(|v| v as u32),
            seats: body["seats"].as_u64().map(|v| v as u8),
            weight_kg: body["weight"].as_u64().map(|v| v as u32),
            source: self.id().into(),
        };

        let empty = vec![];
        let tests = body["technicalInspections"].as_array().unwrap_or(&empty);
        let inspections: Vec<Inspection> = tests
            .iter()
            .filter_map(|t| {
                let date_str = t["date"].as_str()?;
                let date = chrono::NaiveDate::parse_from_str(&date_str[..10], "%Y-%m-%d").ok()?;
                let result = match t["result"].as_str() {
                    Some("PASSED") => InspectionResult::Pass,
                    Some("FAILED") => InspectionResult::Fail,
                    _ => InspectionResult::Unknown,
                };
                Some(Inspection {
                    country: "LV".into(),
                    date,
                    result,
                    mileage_km: t["odometer"].as_u64(),
                    defects: vec![],
                    advisories: vec![],
                    expiry: t["validUntil"]
                        .as_str()
                        .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
                    test_number: t["number"].as_str().map(|s| s.to_string()),
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
