use async_trait::async_trait;
use std::time::Duration;

use crate::error::SourceError;
use crate::models::{Encumbrance, EncumbranceKind, Registration, RegistrationStatus, SourceData};
use crate::sources::VehicleSource;

/// Italy — ACI (Automobile Club d'Italia) / Portale dell'Automobilista
/// https://www.ilportaledellautomobilista.it/
/// Free VIN-based lookup: registration data, vincoli (encumbrances — liens, seizures, etc.).
pub struct ItAci {
    client: reqwest::Client,
}

impl ItAci {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VehicleSource for ItAci {
    fn id(&self) -> &'static str {
        "it_aci"
    }
    fn country(&self) -> &'static str {
        "IT"
    }
    fn name(&self) -> &'static str {
        "ACI (IT)"
    }
    fn cache_ttl(&self) -> Duration {
        Duration::from_secs(60 * 60 * 24)
    }

    async fn fetch_by_vin(&self, vin: &str) -> Result<SourceData, SourceError> {
        // Portale dell'Automobilista — VIN (telaio) lookup
        let url = format!(
            "https://www.ilportaledellautomobilista.it/api/vehicle/telaio/{}",
            vin
        );
        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .header("Referer", "https://www.ilportaledellautomobilista.it/")
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

        if body.is_null() || body.get("telaio").is_none() {
            return Err(SourceError::NotFound);
        }

        let status = match body["statoVeicolo"].as_str() {
            Some("IMMATRICOLATO") => RegistrationStatus::Active,
            Some("RADIATO") | Some("DEMOLITO") => RegistrationStatus::Deregistered,
            Some("ESPORTATO") => RegistrationStatus::Exported,
            _ => RegistrationStatus::Unknown,
        };

        let reg = Registration {
            country: "IT".into(),
            plate: body["targa"].as_str().map(|s| s.to_string()),
            first_registered: body["dataImmatricolazione"]
                .as_str()
                .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
            deregistered: body["dataRadiazione"]
                .as_str()
                .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
            status,
            color: body["colore"].as_str().map(|s| s.to_string()),
            fuel: body["alimentazione"].as_str().map(|s| s.to_string()),
            body: body["carrozzeria"].as_str().map(|s| s.to_string()),
            engine_cc: body["cilindrata"].as_u64().map(|v| v as u32),
            power_kw: body["potenzaKw"].as_u64().map(|v| v as u32),
            seats: body["posti"].as_u64().map(|v| v as u8),
            weight_kg: body["taraKg"].as_u64().map(|v| v as u32),
            source: self.id().into(),
        };

        // Vincoli (encumbrances): fermi, pignoramenti, ipoteche
        let empty = vec![];
        let vincoli = body["vincoli"].as_array().unwrap_or(&empty);
        let encumbrances: Vec<Encumbrance> = vincoli
            .iter()
            .map(|v| {
                let kind = match v["tipoVincolo"].as_str() {
                    Some("FERMO_AMMINISTRATIVO") => EncumbranceKind::Seizure,
                    Some("IPOTECA") => EncumbranceKind::Lien,
                    Some("PEGNO") => EncumbranceKind::Lien,
                    _ => EncumbranceKind::Other,
                };
                Encumbrance {
                    kind,
                    description: v["descrizione"].as_str().map(|s| s.to_string()),
                    country: "IT".into(),
                    source: self.id().into(),
                }
            })
            .collect();

        Ok(SourceData {
            registrations: vec![reg],
            encumbrances,
            ..Default::default()
        })
    }
}
