use async_trait::async_trait;
use std::time::Duration;

use crate::error::SourceError;
use crate::models::{Inspection, InspectionResult, Registration, RegistrationStatus, SourceData};
use crate::sources::VehicleSource;

/// Iceland — Samgöngustofa (Transport Authority of Iceland)
/// https://www.samgongustofa.is/
/// Free VIN-based lookup: registration data, periodic roadworthiness inspection history.
pub struct IsSamgongustofa {
    client: reqwest::Client,
}

impl IsSamgongustofa {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VehicleSource for IsSamgongustofa {
    fn id(&self) -> &'static str {
        "is_samgongustofa"
    }
    fn country(&self) -> &'static str {
        "IS"
    }
    fn name(&self) -> &'static str {
        "Samgöngustofa (IS)"
    }
    fn cache_ttl(&self) -> Duration {
        Duration::from_secs(60 * 60 * 24)
    }

    async fn fetch_by_vin(&self, vin: &str) -> Result<SourceData, SourceError> {
        // Samgöngustofa open vehicle register — VIN (undirstell) search
        let url = format!(
            "https://api.samgongustofa.is/vehicle/vin/{}",
            vin
        );
        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .header("Referer", "https://www.samgongustofa.is/")
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

        if body.is_null() || body.get("undirstell").is_none() {
            return Err(SourceError::NotFound);
        }

        let status = match body["skraningStada"].as_str() {
            Some("SKRADT") => RegistrationStatus::Active,
            Some("AFSKRADT") => RegistrationStatus::Deregistered,
            Some("FLUTT_UT") => RegistrationStatus::Exported,
            _ => RegistrationStatus::Unknown,
        };

        let reg = Registration {
            country: "IS".into(),
            plate: body["skraningNumer"].as_str().map(|s| s.to_string()),
            first_registered: body["fyrstaSkraningDagsetning"]
                .as_str()
                .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
            deregistered: body["afskraningDagsetning"]
                .as_str()
                .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
            status,
            color: body["litur"].as_str().map(|s| s.to_string()),
            fuel: body["eldsneyti"].as_str().map(|s| s.to_string()),
            body: body["yfirbygging"].as_str().map(|s| s.to_string()),
            engine_cc: body["velarrummal"].as_u64().map(|v| v as u32),
            power_kw: body["velarkrafturKw"].as_u64().map(|v| v as u32),
            seats: body["saetafjoldi"].as_u64().map(|v| v as u8),
            weight_kg: body["eiginThyngd"].as_u64().map(|v| v as u32),
            source: self.id().into(),
        };

        let empty = vec![];
        let skodunarferd = body["skodunarferd"].as_array().unwrap_or(&empty);
        let inspections: Vec<Inspection> = skodunarferd
            .iter()
            .filter_map(|s| {
                let date_str = s["skodunarDagur"].as_str()?;
                let date = chrono::NaiveDate::parse_from_str(&date_str[..10], "%Y-%m-%d").ok()?;
                let result = match s["nidurstada"].as_str() {
                    Some("SAMTHYKKT") | Some("PASSED") => InspectionResult::Pass,
                    Some("HAFNATH") | Some("FAILED") => InspectionResult::Fail,
                    Some("SKILYRTHIS") => InspectionResult::Advisory,
                    _ => InspectionResult::Unknown,
                };
                Some(Inspection {
                    country: "IS".into(),
                    date,
                    result,
                    mileage_km: s["kilometrar"].as_u64(),
                    defects: vec![],
                    advisories: vec![],
                    expiry: s["gildirTil"]
                        .as_str()
                        .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
                    test_number: s["skodunarnumer"].as_str().map(|s| s.to_string()),
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
