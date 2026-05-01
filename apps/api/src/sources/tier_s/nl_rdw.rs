use async_trait::async_trait;
use serde::Deserialize;
use std::time::Duration;

use crate::error::SourceError;
use crate::models::{Inspection, InspectionResult, Registration, RegistrationStatus, SourceData};
use crate::sources::VehicleSource;

/// RDW Open Data (Netherlands) — Socrata API
/// Dataset m9d7-ebf2 (basic) + sgfe-77wx (APK inspections)
pub struct NlRdw {
    client: reqwest::Client,
    app_token: Option<String>,
}

impl NlRdw {
    pub fn new(client: reqwest::Client, app_token: Option<String>) -> Self {
        Self { client, app_token }
    }

    fn base_url(&self) -> &'static str {
        "https://opendata.rdw.nl/resource"
    }

    async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        dataset: &str,
        query: &str,
    ) -> Result<Vec<T>, SourceError> {
        let url = format!("{}/{}.json?{}", self.base_url(), dataset, query);
        let mut req = self.client.get(&url);
        if let Some(token) = &self.app_token {
            req = req.header("X-App-Token", token);
        }
        let resp = req.send().await?;
        if resp.status() == 429 {
            return Err(SourceError::RateLimited);
        }
        let data = resp
            .json::<Vec<T>>()
            .await
            .map_err(|e| SourceError::Parse(e.to_string()))?;
        Ok(data)
    }
}

#[derive(Debug, Deserialize)]
struct RdwVehicle {
    kenteken: Option<String>,
    voertuigsoort: Option<String>,
    merk: Option<String>,
    handelsbenaming: Option<String>,
    datum_eerste_toelating: Option<String>,
    datum_tenaamstelling: Option<String>,
    vervaldatum_apk: Option<String>,
    voertuigstatus: Option<String>,
    kleur: Option<String>,
    brandstof_omschrijving: Option<String>,
    inrichting: Option<String>,
    cilinderinhoud: Option<String>,
    vermogen_massarijklaar: Option<String>,
    aantal_zitplaatsen: Option<String>,
    massa_rijklaar: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RdwApk {
    kenteken: Option<String>,
    soort_erkenning_omschrijving: Option<String>,
    vervaldatum_keuring: Option<String>,
    datum_eerste_toelating: Option<String>,
    toegestane_maximum_massa_voertuig: Option<String>,
    // mileage is not available in the APK dataset directly
}

fn parse_rdw_date(s: &str) -> Option<chrono::NaiveDate> {
    // RDW dates are YYYYMMDD
    chrono::NaiveDate::parse_from_str(s, "%Y%m%d").ok()
}

fn nonempty(s: Option<String>) -> Option<String> {
    s.filter(|v| !v.trim().is_empty())
}

#[async_trait]
impl VehicleSource for NlRdw {
    fn id(&self) -> &'static str {
        "nl_rdw"
    }
    fn country(&self) -> &'static str {
        "NL"
    }
    fn name(&self) -> &'static str {
        "RDW Open Data"
    }
    fn cache_ttl(&self) -> Duration {
        Duration::from_secs(60 * 60 * 24) // 24 hours
    }

    async fn fetch_by_vin(&self, vin: &str) -> Result<SourceData, SourceError> {
        // RDW is keyed by kenteken (plate), not VIN directly in the public dataset.
        // We search by VIN via the basic dataset where chassis_nummer == VIN.
        let query = format!("$where=voertuigstatus IS NOT NULL&$limit=5&chassisnummer={}", vin);
        let vehicles: Vec<RdwVehicle> = self.get("m9d7-ebf2", &query).await?;

        if vehicles.is_empty() {
            return Err(SourceError::NotFound);
        }

        let v = &vehicles[0];

        let first_registered = v
            .datum_eerste_toelating
            .as_deref()
            .and_then(parse_rdw_date);

        let status = match v.voertuigstatus.as_deref().unwrap_or("") {
            s if s.contains("Gestolen") => RegistrationStatus::Stolen,
            s if s.contains("Geëxporteerd") || s.contains("geexporteerd") => {
                RegistrationStatus::Exported
            }
            s if s.contains("Geregistreerd") => RegistrationStatus::Active,
            _ => RegistrationStatus::Unknown,
        };

        let reg = Registration {
            country: "NL".into(),
            plate: nonempty(v.kenteken.clone()),
            first_registered,
            deregistered: None,
            status,
            color: nonempty(v.kleur.clone()),
            fuel: nonempty(v.brandstof_omschrijving.clone()),
            body: nonempty(v.inrichting.clone()),
            engine_cc: v
                .cilinderinhoud
                .as_deref()
                .and_then(|s| s.parse::<u32>().ok()),
            power_kw: v
                .vermogen_massarijklaar
                .as_deref()
                .and_then(|s| s.parse::<u32>().ok()),
            seats: v
                .aantal_zitplaatsen
                .as_deref()
                .and_then(|s| s.parse::<u8>().ok()),
            weight_kg: v
                .massa_rijklaar
                .as_deref()
                .and_then(|s| s.parse::<u32>().ok()),
            source: self.id().into(),
        };

        // Fetch APK (inspection) history
        let plate = v.kenteken.as_deref().unwrap_or("");
        let apk_query = format!("kenteken={}&$order=vervaldatum_keuring DESC&$limit=50", plate);
        let apk_records: Vec<serde_json::Value> = self.get("sgfe-77wx", &apk_query).await.unwrap_or_default();

        let inspections: Vec<Inspection> = apk_records
            .iter()
            .filter_map(|r| {
                let date_str = r["vervaldatum_keuring"].as_str()?;
                let date = parse_rdw_date(date_str)?;
                Some(Inspection {
                    country: "NL".into(),
                    date,
                    result: InspectionResult::Pass, // APK pass = issued
                    mileage_km: r["kilometerstand"]
                        .as_str()
                        .and_then(|s| s.parse::<u64>().ok()),
                    defects: vec![],
                    advisories: vec![],
                    expiry: None,
                    test_number: r["keuringsinstantie_code"].as_str().map(|s| s.to_string()),
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
