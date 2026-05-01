use async_trait::async_trait;
use std::time::Duration;

use crate::error::SourceError;
use crate::models::{Inspection, InspectionResult, Registration, RegistrationStatus, SourceData};
use crate::sources::VehicleSource;

/// Poland — pojazd.gov.pl (CEPIK public data)
/// https://pojazd.gov.pl/
/// Free VIN-based lookup backed by the central CEPIK vehicle register.
pub struct PlHistoriaPojazdu {
    client: reqwest::Client,
}

impl PlHistoriaPojazdu {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VehicleSource for PlHistoriaPojazdu {
    fn id(&self) -> &'static str {
        "pl_historiapojazdu"
    }
    fn country(&self) -> &'static str {
        "PL"
    }
    fn name(&self) -> &'static str {
        "pojazd.gov.pl (PL)"
    }
    fn cache_ttl(&self) -> Duration {
        Duration::from_secs(60 * 60 * 24)
    }

    async fn fetch_by_vin(&self, vin: &str) -> Result<SourceData, SourceError> {
        let url = format!("https://historiapojazdu.gov.pl/api/vehicle/{}", vin);
        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .header("Referer", "https://historiapojazdu.gov.pl/")
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

        let status = match body["statusRejestracji"].as_str() {
            Some("ZAREJESTROWANY") => RegistrationStatus::Active,
            Some("WYREJESTROWANY") => RegistrationStatus::Deregistered,
            Some("SKRADZIONY") => RegistrationStatus::Stolen,
            _ => RegistrationStatus::Unknown,
        };

        let reg = Registration {
            country: "PL".into(),
            plate: body["numerRejestracyjny"].as_str().map(|s| s.to_string()),
            first_registered: body["datapierwszejRejestracji"]
                .as_str()
                .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
            deregistered: body["dataWyrejestrowania"]
                .as_str()
                .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
            status,
            color: body["kolor"].as_str().map(|s| s.to_string()),
            fuel: body["rodzajPaliwa"].as_str().map(|s| s.to_string()),
            body: body["rodzajNadwozia"].as_str().map(|s| s.to_string()),
            engine_cc: body["pojemnoscSilnika"].as_u64().map(|v| v as u32),
            power_kw: body["mocSilnikaKw"].as_u64().map(|v| v as u32),
            seats: body["liczbaMiejscSiedzacych"].as_u64().map(|v| v as u8),
            weight_kg: body["masaWlasna"].as_u64().map(|v| v as u32),
            source: self.id().into(),
        };

        let empty = vec![];
        let przeglady = body["przeglady"].as_array().unwrap_or(&empty);
        let inspections: Vec<Inspection> = przeglady
            .iter()
            .filter_map(|p| {
                let date_str = p["data"].as_str()?;
                let date = chrono::NaiveDate::parse_from_str(&date_str[..10], "%Y-%m-%d").ok()?;
                let result = match p["wynik"].as_str() {
                    Some("POZYTYWNY") | Some("PASSED") => InspectionResult::Pass,
                    Some("NEGATYWNY") | Some("FAILED") => InspectionResult::Fail,
                    Some("WARUNKOWO") => InspectionResult::Advisory,
                    _ => InspectionResult::Unknown,
                };
                Some(Inspection {
                    country: "PL".into(),
                    date,
                    result,
                    mileage_km: p["przebiegKm"].as_u64(),
                    defects: vec![],
                    advisories: vec![],
                    expiry: p["waznoscDo"]
                        .as_str()
                        .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
                    test_number: p["numerBadania"].as_str().map(|s| s.to_string()),
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
