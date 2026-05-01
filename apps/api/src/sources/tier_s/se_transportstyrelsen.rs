use async_trait::async_trait;
use std::time::Duration;

use crate::error::SourceError;
use crate::models::{Inspection, InspectionResult, Registration, RegistrationStatus, SourceData};
use crate::sources::VehicleSource;

/// Sweden — Transportstyrelsen public vehicle lookup
/// https://fu-regnr.transportstyrelsen.se/extweb
/// Free per-plate lookup: model, year, color, fuel, weight, inspection history, mileage.
pub struct SeTransportstyrelsen {
    client: reqwest::Client,
}

impl SeTransportstyrelsen {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VehicleSource for SeTransportstyrelsen {
    fn id(&self) -> &'static str {
        "se_transportstyrelsen"
    }
    fn country(&self) -> &'static str {
        "SE"
    }
    fn name(&self) -> &'static str {
        "Transportstyrelsen (SE)"
    }
    fn cache_ttl(&self) -> Duration {
        Duration::from_secs(60 * 60 * 24)
    }

    async fn fetch_by_vin(&self, vin: &str) -> Result<SourceData, SourceError> {
        // Transportstyrelsen public API — queried by regnr (plate) or chassinummer (VIN)
        let url = format!(
            "https://fu-regnr.transportstyrelsen.se/api/v1/vehicles/chassinummer/{}",
            vin
        );
        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .header("Origin", "https://fu-regnr.transportstyrelsen.se")
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

        if body.get("registreringsnummer").is_none() && body.get("chassinummer").is_none() {
            return Err(SourceError::NotFound);
        }

        let status = match body["avregistreringsdatum"].as_str() {
            Some(_) => RegistrationStatus::Deregistered,
            None => RegistrationStatus::Active,
        };

        let reg = Registration {
            country: "SE".into(),
            plate: body["registreringsnummer"].as_str().map(|s| s.to_string()),
            first_registered: body["forstaRegistreringsDatum"]
                .as_str()
                .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
            deregistered: body["avregistreringsdatum"]
                .as_str()
                .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
            status,
            color: body["farg"].as_str().map(|s| s.to_string()),
            fuel: body["drivmedel"].as_str().map(|s| s.to_string()),
            body: body["karosseri"].as_str().map(|s| s.to_string()),
            engine_cc: body["cylindervolym"].as_u64().map(|v| v as u32),
            power_kw: body["effekt"].as_u64().map(|v| v as u32),
            seats: body["totalSittplatser"].as_u64().map(|v| v as u8),
            weight_kg: body["tjänstevikt"].as_u64().map(|v| v as u32),
            source: self.id().into(),
        };

        let empty = vec![];
        let besiktningar = body["besiktningar"].as_array().unwrap_or(&empty);
        let inspections: Vec<Inspection> = besiktningar
            .iter()
            .filter_map(|b| {
                let date_str = b["besiktningsDatum"].as_str()?;
                let date = chrono::NaiveDate::parse_from_str(&date_str[..10], "%Y-%m-%d").ok()?;
                let result = match b["besiktningsresultat"].as_str() {
                    Some("Godkänd") | Some("1") => InspectionResult::Pass,
                    Some("Underkänd") | Some("2") => InspectionResult::Fail,
                    Some("Komplettering") | Some("3") => InspectionResult::Advisory,
                    _ => InspectionResult::Unknown,
                };
                Some(Inspection {
                    country: "SE".into(),
                    date,
                    result,
                    mileage_km: b["milstand"].as_u64().map(|m| m * 10), // Swedish milstand is in mil (10km)
                    defects: vec![],
                    advisories: vec![],
                    expiry: b["besiktningsFrist"]
                        .as_str()
                        .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
                    test_number: b["protokollNr"].as_str().map(|s| s.to_string()),
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
