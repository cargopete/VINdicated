use async_trait::async_trait;
use std::time::Duration;

use crate::error::SourceError;
use crate::models::{Inspection, InspectionResult, Registration, RegistrationStatus, SourceData};
use crate::sources::VehicleSource;

/// Norway — Statens vegvesen public vehicle lookup
/// https://www.vegvesen.no/kjoretoy/kjop-og-salg/kjoretoyopplysninger/
/// Returns make, model, fuel, EU class, last inspection mileage (since 30 Nov 2009).
pub struct NoVegvesen {
    client: reqwest::Client,
}

impl NoVegvesen {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VehicleSource for NoVegvesen {
    fn id(&self) -> &'static str {
        "no_vegvesen"
    }
    fn country(&self) -> &'static str {
        "NO"
    }
    fn name(&self) -> &'static str {
        "Statens vegvesen (NO)"
    }
    fn cache_ttl(&self) -> Duration {
        Duration::from_secs(60 * 60 * 24)
    }

    async fn fetch_by_vin(&self, vin: &str) -> Result<SourceData, SourceError> {
        // Vegvesen's public API — undocumented but stable
        let url = format!(
            "https://www.vegvesen.no/kjoretoy/kjop-og-salg/kjoretoyopplysninger/kjoretoyopplysninger-enkeltoppslag/?kjennemerke={}",
            vin
        );
        // Use the JSON API endpoint
        let api_url = format!(
            "https://www.vegvesen.no/ws/no/vegvesen/kjoretoy/felles/datautlevering/enkeltoppslag/kjoretoydata?kjennemerke={}",
            vin
        );
        let resp = self
            .client
            .get(&api_url)
            .header("Accept", "application/json")
            .header("SVV-Authorization", "Svv-Token 0")
            .send()
            .await?;

        match resp.status().as_u16() {
            200 => {}
            400 | 404 => return Err(SourceError::NotFound),
            429 => return Err(SourceError::RateLimited),
            s => return Err(SourceError::Unavailable(format!("HTTP {}", s))),
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| SourceError::Parse(e.to_string()))?;

        let kjoretoy = match body.get("kjoretoydataListe").and_then(|l| l.as_array()).and_then(|a| a.first()) {
            Some(k) => k.clone(),
            None => return Err(SourceError::NotFound),
        };

        let godkjenning = &kjoretoy["godkjenning"]["forstegangsGodkjenning"];
        let first_reg = godkjenning["forstegangRegistrertDato"]
            .as_str()
            .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

        let teknisk = &kjoretoy["godkjenning"]["tekniskGodkjenning"]["tekniskeData"];
        let fuel = teknisk["motorOgDrivverk"]["motor"]
            .as_array()
            .and_then(|a| a.first())
            .and_then(|m| m["drivstoffKode"]["kodeBeskrivelse"].as_str())
            .map(|s| s.to_string());

        let reg = Registration {
            country: "NO".into(),
            plate: kjoretoy["kjennemerke"].as_str().map(|s| s.to_string()),
            first_registered: first_reg,
            deregistered: None,
            status: RegistrationStatus::Unknown,
            color: teknisk["karosseriOgLasteplan"]["rFarge"]
                .as_array()
                .and_then(|a| a.first())
                .and_then(|c| c["kodeBeskrivelse"].as_str())
                .map(|s| s.to_string()),
            fuel,
            body: teknisk["karosseriOgLasteplan"]["karosseritype"]["kodeBeskrivelse"]
                .as_str()
                .map(|s| s.to_string()),
            engine_cc: teknisk["motorOgDrivverk"]["motor"]
                .as_array()
                .and_then(|a| a.first())
                .and_then(|m| m["slagvolum"].as_u64())
                .map(|v| v as u32),
            power_kw: teknisk["motorOgDrivverk"]["motor"]
                .as_array()
                .and_then(|a| a.first())
                .and_then(|m| m["maksNettoEffekt"].as_u64())
                .map(|v| v as u32),
            seats: teknisk["persontall"]["sitteplasserTotalt"].as_u64().map(|v| v as u8),
            weight_kg: teknisk["vekter"]["egenvekt"].as_u64().map(|v| v as u32),
            source: self.id().into(),
        };

        // PKK (periodic inspection) — only the most recent is in the base response
        let mut inspections = vec![];
        if let Some(pkk) = kjoretoy.get("periodiskKjoretoyKontroll") {
            if let Some(date_str) = pkk["sistPKKDato"].as_str() {
                if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                    inspections.push(Inspection {
                        country: "NO".into(),
                        date,
                        result: InspectionResult::Pass,
                        mileage_km: pkk["kilometerstand"].as_u64(),
                        defects: vec![],
                        advisories: vec![],
                        expiry: pkk["nestePKKFrist"]
                            .as_str()
                            .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()),
                        test_number: None,
                        source: self.id().into(),
                    });
                }
            }
        }

        Ok(SourceData {
            registrations: vec![reg],
            inspections,
            ..Default::default()
        })
    }
}
