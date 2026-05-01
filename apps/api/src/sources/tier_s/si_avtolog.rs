use async_trait::async_trait;
use std::time::Duration;

use crate::error::SourceError;
use crate::models::{Inspection, InspectionResult, Registration, RegistrationStatus, SourceData};
use crate::sources::VehicleSource;

/// Slovenia — Avtolog (AMZS / OPSI data)
/// https://avtolog.si/
/// Free VIN-based lookup: registration, mileage progression from technical inspections.
pub struct SiAvtolog {
    client: reqwest::Client,
}

impl SiAvtolog {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VehicleSource for SiAvtolog {
    fn id(&self) -> &'static str {
        "si_avtolog"
    }
    fn country(&self) -> &'static str {
        "SI"
    }
    fn name(&self) -> &'static str {
        "Avtolog (SI)"
    }
    fn cache_ttl(&self) -> Duration {
        Duration::from_secs(60 * 60 * 24)
    }

    async fn fetch_by_vin(&self, vin: &str) -> Result<SourceData, SourceError> {
        let url = format!("https://avtolog.si/api/v1/vehicle/{}", vin);
        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .header("Referer", "https://avtolog.si/")
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

        if body.get("vin").is_none() && body.get("registrskaOznaka").is_none() {
            return Err(SourceError::NotFound);
        }

        let reg = Registration {
            country: "SI".into(),
            plate: body["registrskaOznaka"].as_str().map(|s| s.to_string()),
            first_registered: body["datumPrveRegistracije"]
                .as_str()
                .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
            deregistered: None,
            status: RegistrationStatus::Unknown,
            color: body["barva"].as_str().map(|s| s.to_string()),
            fuel: body["gorivo"].as_str().map(|s| s.to_string()),
            body: body["vrstaKaroserije"].as_str().map(|s| s.to_string()),
            engine_cc: body["prostorninaMotorja"].as_u64().map(|v| v as u32),
            power_kw: body["mocMotoraKw"].as_u64().map(|v| v as u32),
            seats: body["steviloSedezev"].as_u64().map(|v| v as u8),
            weight_kg: body["masaVozila"].as_u64().map(|v| v as u32),
            source: self.id().into(),
        };

        let empty = vec![];
        let pregledi = body["tehnicniPregledi"].as_array().unwrap_or(&empty);
        let inspections: Vec<Inspection> = pregledi
            .iter()
            .filter_map(|p| {
                let date_str = p["datum"].as_str()?;
                let date = chrono::NaiveDate::parse_from_str(&date_str[..10], "%Y-%m-%d").ok()?;
                let result = match p["rezultat"].as_str() {
                    Some("PASSED") | Some("POZITIVEN") => InspectionResult::Pass,
                    Some("FAILED") | Some("NEGATIVEN") => InspectionResult::Fail,
                    _ => InspectionResult::Unknown,
                };
                Some(Inspection {
                    country: "SI".into(),
                    date,
                    result,
                    mileage_km: p["stanjeStevcaKm"].as_u64(),
                    defects: vec![],
                    advisories: vec![],
                    expiry: p["veljavnostDo"]
                        .as_str()
                        .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
                    test_number: p["stevilkaPotrdila"].as_str().map(|s| s.to_string()),
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
