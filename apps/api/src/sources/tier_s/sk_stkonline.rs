use async_trait::async_trait;
use std::time::Duration;

use crate::error::SourceError;
use crate::models::{Inspection, InspectionResult, Registration, RegistrationStatus, SourceData};
use crate::sources::VehicleSource;

/// Slovakia — STKonline.sk (JISCD data)
/// https://www.stkonline.sk/
/// Free public lookup: full TK (technical inspection) + EK (emission) history by VIN or plate.
pub struct SkStkOnline {
    client: reqwest::Client,
}

impl SkStkOnline {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VehicleSource for SkStkOnline {
    fn id(&self) -> &'static str {
        "sk_stkonline"
    }
    fn country(&self) -> &'static str {
        "SK"
    }
    fn name(&self) -> &'static str {
        "STKonline (SK)"
    }
    fn cache_ttl(&self) -> Duration {
        Duration::from_secs(60 * 60 * 24)
    }

    async fn fetch_by_vin(&self, vin: &str) -> Result<SourceData, SourceError> {
        let url = format!("https://www.stkonline.sk/api/vehicle/{}", vin);
        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .header("Referer", "https://www.stkonline.sk/")
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

        // Registration
        let mut registrations = vec![];
        if body.get("make").is_some() {
            let reg = Registration {
                country: "SK".into(),
                plate: body["plate"].as_str().map(|s| s.to_string()),
                first_registered: body["firstRegistration"]
                    .as_str()
                    .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
                deregistered: None,
                status: RegistrationStatus::Unknown,
                color: body["color"].as_str().map(|s| s.to_string()),
                fuel: body["fuelType"].as_str().map(|s| s.to_string()),
                body: body["bodyType"].as_str().map(|s| s.to_string()),
                engine_cc: body["engineVolume"].as_u64().map(|v| v as u32),
                power_kw: body["powerKw"].as_u64().map(|v| v as u32),
                seats: body["seats"].as_u64().map(|v| v as u8),
                weight_kg: body["weightKg"].as_u64().map(|v| v as u32),
                source: self.id().into(),
            };
            registrations.push(reg);
        }

        // Inspections
        let empty = vec![];
        let records = body["inspections"].as_array().unwrap_or(&empty);
        let inspections: Vec<Inspection> = records
            .iter()
            .filter_map(|r| {
                let date_str = r["date"].as_str()?;
                let date = chrono::NaiveDate::parse_from_str(&date_str[..10], "%Y-%m-%d").ok()?;

                let mileage_km = r["mileage"].as_u64();
                let result = match r["result"].as_str() {
                    Some("PASSED") | Some("P") => InspectionResult::Pass,
                    Some("FAILED") | Some("F") => InspectionResult::Fail,
                    _ => InspectionResult::Unknown,
                };

                Some(Inspection {
                    country: "SK".into(),
                    date,
                    result,
                    mileage_km,
                    defects: r["defects"]
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .filter_map(|d| d.as_str().map(|s| s.to_string()))
                        .collect(),
                    advisories: vec![],
                    expiry: r["validUntil"]
                        .as_str()
                        .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
                    test_number: r["testNumber"].as_str().map(|s| s.to_string()),
                    source: self.id().into(),
                })
            })
            .collect();

        if registrations.is_empty() && inspections.is_empty() {
            return Err(SourceError::NotFound);
        }

        Ok(SourceData {
            registrations,
            inspections,
            ..Default::default()
        })
    }
}
