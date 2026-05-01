use async_trait::async_trait;
use std::time::Duration;

use crate::error::SourceError;
use crate::models::{Inspection, InspectionResult, Registration, RegistrationStatus, SourceData};
use crate::sources::VehicleSource;

/// Denmark — DMR Motorregister (via the public AltOmBilen.dk API)
/// https://motorregister.skat.dk/
/// Free public lookup by plate or VIN: make, model, year, fuel, weight, inspection history.
pub struct DkDmr {
    client: reqwest::Client,
}

impl DkDmr {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VehicleSource for DkDmr {
    fn id(&self) -> &'static str {
        "dk_dmr"
    }
    fn country(&self) -> &'static str {
        "DK"
    }
    fn name(&self) -> &'static str {
        "DMR Motorregister (DK)"
    }
    fn cache_ttl(&self) -> Duration {
        Duration::from_secs(60 * 60 * 24)
    }

    async fn fetch_by_vin(&self, vin: &str) -> Result<SourceData, SourceError> {
        // AltOmBilen.dk exposes DMR data via a public JSON API (no key required)
        let url = format!(
            "https://api.altombilen.dk/v1/vehicle/vin/{}",
            vin
        );
        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .header("Origin", "https://www.altombilen.dk")
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

        if body.get("stel").is_none() && body.get("vin").is_none() {
            return Err(SourceError::NotFound);
        }

        let status = match body["status"].as_str() {
            Some("REGISTRERET") => RegistrationStatus::Active,
            Some("AFMELDT") => RegistrationStatus::Deregistered,
            Some("EKSPORTERET") => RegistrationStatus::Exported,
            _ => RegistrationStatus::Unknown,
        };

        let reg = Registration {
            country: "DK".into(),
            plate: body["registreringsnummer"].as_str().map(|s| s.to_string()),
            first_registered: body["foersteregistreringsdato"]
                .as_str()
                .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
            deregistered: body["afmeldtDato"]
                .as_str()
                .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
            status,
            color: body["farve"].as_str().map(|s| s.to_string()),
            fuel: body["drivkraft"].as_str().map(|s| s.to_string()),
            body: body["karrosseri"].as_str().map(|s| s.to_string()),
            engine_cc: body["slagvolumen"].as_u64().map(|v| v as u32),
            power_kw: body["motorEffektKw"].as_u64().map(|v| v as u32),
            seats: body["siddepladserIAlt"].as_u64().map(|v| v as u8),
            weight_kg: body["egenvægt"].as_u64().map(|v| v as u32),
            source: self.id().into(),
        };

        let empty = vec![];
        let synshistorik = body["synshistorik"].as_array().unwrap_or(&empty);
        let inspections: Vec<Inspection> = synshistorik
            .iter()
            .filter_map(|s| {
                let date_str = s["synsdato"].as_str()?;
                let date = chrono::NaiveDate::parse_from_str(&date_str[..10], "%Y-%m-%d").ok()?;
                let result = match s["resultat"].as_str() {
                    Some("Godkendt") | Some("G") => InspectionResult::Pass,
                    Some("Ikke godkendt") | Some("U") => InspectionResult::Fail,
                    Some("Betinget godkendt") | Some("B") => InspectionResult::Advisory,
                    _ => InspectionResult::Unknown,
                };
                Some(Inspection {
                    country: "DK".into(),
                    date,
                    result,
                    mileage_km: s["kilometerstand"].as_u64(),
                    defects: vec![],
                    advisories: vec![],
                    expiry: s["udloebsdato"]
                        .as_str()
                        .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
                    test_number: s["synsnummer"].as_str().map(|s| s.to_string()),
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
