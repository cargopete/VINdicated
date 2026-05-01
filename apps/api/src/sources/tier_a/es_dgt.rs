use async_trait::async_trait;
use std::time::Duration;

use crate::error::SourceError;
use crate::models::{Inspection, InspectionResult, Registration, RegistrationStatus, SourceData};
use crate::sources::VehicleSource;

/// Spain — DGT / ITV (Inspección Técnica de Vehículos)
/// https://itv.dgt.es/
/// Free lookup by VIN: registration status + ITV (roadworthiness) inspection history.
/// Note: DGT's Informe Reducido requires plate + owner ID; the ITV portal is VIN-accessible.
pub struct EsDgt {
    client: reqwest::Client,
}

impl EsDgt {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VehicleSource for EsDgt {
    fn id(&self) -> &'static str {
        "es_dgt"
    }
    fn country(&self) -> &'static str {
        "ES"
    }
    fn name(&self) -> &'static str {
        "DGT/ITV (ES)"
    }
    fn cache_ttl(&self) -> Duration {
        Duration::from_secs(60 * 60 * 24)
    }

    async fn fetch_by_vin(&self, vin: &str) -> Result<SourceData, SourceError> {
        // Spanish ITV public API — VIN-based lookup
        let url = format!(
            "https://itv.dgt.es/ItvSigeWeb/rest/vehiculo/bastidor/{}",
            vin
        );
        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .header("Referer", "https://itv.dgt.es/")
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

        if body.is_null() || (body.get("bastidor").is_none() && body.get("vin").is_none()) {
            return Err(SourceError::NotFound);
        }

        let status = match body["estadoVehiculo"].as_str() {
            Some("ALTA") => RegistrationStatus::Active,
            Some("BAJA") | Some("BAJA_TEMPORAL") => RegistrationStatus::Deregistered,
            _ => RegistrationStatus::Unknown,
        };

        let reg = Registration {
            country: "ES".into(),
            plate: body["matricula"].as_str().map(|s| s.to_string()),
            first_registered: body["fechaPrimeraMatriculacion"]
                .as_str()
                .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
            deregistered: body["fechaBaja"]
                .as_str()
                .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
            status,
            color: body["color"].as_str().map(|s| s.to_string()),
            fuel: body["combustible"].as_str().map(|s| s.to_string()),
            body: body["carroceria"].as_str().map(|s| s.to_string()),
            engine_cc: body["cilindrada"].as_u64().map(|v| v as u32),
            power_kw: body["potenciaKw"].as_u64().map(|v| v as u32),
            seats: body["plazas"].as_u64().map(|v| v as u8),
            weight_kg: body["taraKg"].as_u64().map(|v| v as u32),
            source: self.id().into(),
        };

        let empty = vec![];
        let inspecciones = body["inspeccionesTecnicas"].as_array().unwrap_or(&empty);
        let inspections: Vec<Inspection> = inspecciones
            .iter()
            .filter_map(|i| {
                let date_str = i["fechaInspeccion"].as_str()?;
                let date = chrono::NaiveDate::parse_from_str(&date_str[..10], "%Y-%m-%d").ok()?;
                let result = match i["resultado"].as_str() {
                    Some("FAVORABLE") | Some("F") => InspectionResult::Pass,
                    Some("DESFAVORABLE") | Some("D") => InspectionResult::Fail,
                    Some("NEGATIVA") | Some("N") => InspectionResult::Fail,
                    Some("CONDICIONADA") | Some("C") => InspectionResult::Advisory,
                    _ => InspectionResult::Unknown,
                };
                Some(Inspection {
                    country: "ES".into(),
                    date,
                    result,
                    mileage_km: i["kilometraje"].as_u64(),
                    defects: vec![],
                    advisories: vec![],
                    expiry: i["fechaProximaItv"]
                        .as_str()
                        .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
                    test_number: i["numeroDictamen"].as_str().map(|s| s.to_string()),
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
