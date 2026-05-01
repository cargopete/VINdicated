use async_trait::async_trait;
use std::time::Duration;

use crate::error::SourceError;
use crate::models::{Registration, RegistrationStatus, SourceData};
use crate::sources::VehicleSource;

/// Bulgaria — KAT (Katastrofen Avtomobilen Tsentar) / IAMA vehicle register
/// https://www.iama.bg/
/// Free VIN-based lookup via the Bulgarian Road Infrastructure Agency public API.
pub struct BgMobile {
    client: reqwest::Client,
}

impl BgMobile {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VehicleSource for BgMobile {
    fn id(&self) -> &'static str {
        "bg_mobile"
    }
    fn country(&self) -> &'static str {
        "BG"
    }
    fn name(&self) -> &'static str {
        "IAMA (BG)"
    }
    fn cache_ttl(&self) -> Duration {
        Duration::from_secs(60 * 60 * 24 * 7)
    }

    async fn fetch_by_vin(&self, vin: &str) -> Result<SourceData, SourceError> {
        // Bulgarian Road Infrastructure Agency (ИАА) public vehicle register
        let url = format!("https://www.iama.bg/api/vehicle/vin/{}", vin);
        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .header("Referer", "https://www.iama.bg/")
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

        let status = match body["status"].as_str() {
            Some("REGISTERED") | Some("РЕГИСТРИРАН") => RegistrationStatus::Active,
            Some("DEREGISTERED") | Some("ДЕРЕГИСТРИРАН") => RegistrationStatus::Deregistered,
            Some("STOLEN") | Some("ОТКРАДНАТ") => RegistrationStatus::Stolen,
            _ => RegistrationStatus::Unknown,
        };

        let reg = Registration {
            country: "BG".into(),
            plate: body["registrationNumber"].as_str().map(|s| s.to_string()),
            first_registered: body["firstRegistrationDate"]
                .as_str()
                .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
            deregistered: body["deregistrationDate"]
                .as_str()
                .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
            status,
            color: body["color"].as_str().map(|s| s.to_string()),
            fuel: body["fuelType"].as_str().map(|s| s.to_string()),
            body: body["bodyType"].as_str().map(|s| s.to_string()),
            engine_cc: body["engineCapacity"].as_u64().map(|v| v as u32),
            power_kw: body["powerKw"].as_u64().map(|v| v as u32),
            seats: body["seats"].as_u64().map(|v| v as u8),
            weight_kg: body["weight"].as_u64().map(|v| v as u32),
            source: self.id().into(),
        };

        Ok(SourceData {
            registrations: vec![reg],
            ..Default::default()
        })
    }
}
