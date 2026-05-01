use async_trait::async_trait;
use std::time::Duration;

use crate::error::SourceError;
use crate::models::{Inspection, InspectionResult, Registration, RegistrationStatus, SourceData};
use crate::sources::VehicleSource;

/// France — HistoVec (Ministère de l'Intérieur)
/// https://histovec.interieur.gouv.fr/
/// Free VIN-based lookup via the public HistoVec report API.
/// The full "Rapport Complet" requires the owner's share token; this queries the
/// public endpoint which returns basic registration and CT (contrôle technique) data.
pub struct FrHistovec {
    client: reqwest::Client,
}

impl FrHistovec {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VehicleSource for FrHistovec {
    fn id(&self) -> &'static str {
        "fr_histovec"
    }
    fn country(&self) -> &'static str {
        "FR"
    }
    fn name(&self) -> &'static str {
        "HistoVec (FR)"
    }
    fn cache_ttl(&self) -> Duration {
        Duration::from_secs(60 * 60 * 24 * 7)
    }

    async fn fetch_by_vin(&self, vin: &str) -> Result<SourceData, SourceError> {
        // HistoVec public API — VIN-only lookup (basic public data, no owner token required)
        let url = format!(
            "https://histovec.interieur.gouv.fr/histovec/api/v1/vehicle/vin/{}",
            vin
        );
        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .header("Referer", "https://histovec.interieur.gouv.fr/")
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

        let status = match body["etatVehicule"].as_str() {
            Some("EN_CIRCULATION") => RegistrationStatus::Active,
            Some("VOLE") => RegistrationStatus::Stolen,
            Some("SORTI_DU_PARC") => RegistrationStatus::Deregistered,
            _ => RegistrationStatus::Unknown,
        };

        let reg = Registration {
            country: "FR".into(),
            plate: body["immatriculation"].as_str().map(|s| s.to_string()),
            first_registered: body["datePremiereMiseEnCirculation"]
                .as_str()
                .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
            deregistered: None,
            status,
            color: body["couleur"].as_str().map(|s| s.to_string()),
            fuel: body["energie"].as_str().map(|s| s.to_string()),
            body: body["carrosserie"].as_str().map(|s| s.to_string()),
            engine_cc: body["cylindree"].as_u64().map(|v| v as u32),
            power_kw: body["puissanceKw"].as_u64().map(|v| v as u32),
            seats: body["nombrePlaces"].as_u64().map(|v| v as u8),
            weight_kg: body["masseEnOrdreDeMarche"].as_u64().map(|v| v as u32),
            source: self.id().into(),
        };

        let empty = vec![];
        let controles = body["controlesTechniques"].as_array().unwrap_or(&empty);
        let inspections: Vec<Inspection> = controles
            .iter()
            .filter_map(|c| {
                let date_str = c["dateControle"].as_str()?;
                let date = chrono::NaiveDate::parse_from_str(&date_str[..10], "%Y-%m-%d").ok()?;
                let result = match c["resultat"].as_str() {
                    Some("A") => InspectionResult::Pass,       // Favorable
                    Some("S") => InspectionResult::Advisory,  // Favorable avec défaillances mineures
                    Some("R") => InspectionResult::Fail,      // Défaillances majeures
                    Some("X") => InspectionResult::Fail,      // Défaillances critiques
                    _ => InspectionResult::Unknown,
                };
                Some(Inspection {
                    country: "FR".into(),
                    date,
                    result,
                    mileage_km: c["kilometrage"].as_u64(),
                    defects: vec![],
                    advisories: vec![],
                    expiry: c["dateExpiration"]
                        .as_str()
                        .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
                    test_number: c["numeroProcesVerbal"].as_str().map(|s| s.to_string()),
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
