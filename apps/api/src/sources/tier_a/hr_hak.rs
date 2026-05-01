use async_trait::async_trait;
use std::time::Duration;

use crate::error::SourceError;
use crate::models::{Inspection, InspectionResult, Registration, RegistrationStatus, SourceData};
use crate::sources::VehicleSource;

/// Croatia — HAK/CVH vehicle history
/// https://www.hak.hr/provjera-vozila/
/// Free VIN-based lookup: registration data + technical inspection history (periodični tehnički pregled).
pub struct HrHak {
    client: reqwest::Client,
}

impl HrHak {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VehicleSource for HrHak {
    fn id(&self) -> &'static str {
        "hr_hak"
    }
    fn country(&self) -> &'static str {
        "HR"
    }
    fn name(&self) -> &'static str {
        "HAK/CVH (HR)"
    }
    fn cache_ttl(&self) -> Duration {
        Duration::from_secs(60 * 60 * 24)
    }

    async fn fetch_by_vin(&self, vin: &str) -> Result<SourceData, SourceError> {
        let url = format!("https://eservisi.hak.hr/api/vozilo/{}", vin);
        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .header("Referer", "https://www.hak.hr/")
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

        let status = match body["statusVozila"].as_str() {
            Some("REGISTRIRANO") => RegistrationStatus::Active,
            Some("ODJAVLJENO") => RegistrationStatus::Deregistered,
            Some("IZVEZENO") => RegistrationStatus::Exported,
            _ => RegistrationStatus::Unknown,
        };

        let reg = Registration {
            country: "HR".into(),
            plate: body["registarskaOznaka"].as_str().map(|s| s.to_string()),
            first_registered: body["datumPrveRegistracije"]
                .as_str()
                .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
            deregistered: body["datumOdjave"]
                .as_str()
                .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
            status,
            color: body["boja"].as_str().map(|s| s.to_string()),
            fuel: body["vrstaGoriva"].as_str().map(|s| s.to_string()),
            body: body["vrstaKaroserije"].as_str().map(|s| s.to_string()),
            engine_cc: body["obujamMotora"].as_u64().map(|v| v as u32),
            power_kw: body["snagaMotora"].as_u64().map(|v| v as u32),
            seats: body["brojSjedala"].as_u64().map(|v| v as u8),
            weight_kg: body["masinaVozila"].as_u64().map(|v| v as u32),
            source: self.id().into(),
        };

        let empty = vec![];
        let pregledi = body["tehničkiPregledi"].as_array().unwrap_or(&empty);
        let inspections: Vec<Inspection> = pregledi
            .iter()
            .filter_map(|p| {
                let date_str = p["datumPregleda"].as_str()?;
                let date = chrono::NaiveDate::parse_from_str(&date_str[..10], "%Y-%m-%d").ok()?;
                let result = match p["ishod"].as_str() {
                    Some("POZITIVAN") | Some("PASSED") => InspectionResult::Pass,
                    Some("NEGATIVAN") | Some("FAILED") => InspectionResult::Fail,
                    Some("UVJETNO") => InspectionResult::Advisory,
                    _ => InspectionResult::Unknown,
                };
                Some(Inspection {
                    country: "HR".into(),
                    date,
                    result,
                    mileage_km: p["kilometeraza"].as_u64(),
                    defects: vec![],
                    advisories: vec![],
                    expiry: p["vrijediDo"]
                        .as_str()
                        .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
                    test_number: p["brojPotvrde"].as_str().map(|s| s.to_string()),
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
