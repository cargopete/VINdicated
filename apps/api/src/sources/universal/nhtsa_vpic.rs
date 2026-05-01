use async_trait::async_trait;
use serde::Deserialize;
use std::time::Duration;

use crate::error::SourceError;
use crate::models::{SourceData, VinDecode};
use crate::sources::VehicleSource;

pub struct NhtsaVpic {
    client: reqwest::Client,
}

impl NhtsaVpic {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[derive(Debug, Deserialize)]
struct VpicResponse {
    #[serde(rename = "Results")]
    results: Vec<VpicResult>,
}

#[derive(Debug, Deserialize)]
struct VpicResult {
    #[serde(rename = "Make")]
    make: Option<String>,
    #[serde(rename = "Model")]
    model: Option<String>,
    #[serde(rename = "ModelYear")]
    model_year: Option<String>,
    #[serde(rename = "BodyClass")]
    body_class: Option<String>,
    #[serde(rename = "EngineCylinders")]
    engine_cylinders: Option<String>,
    #[serde(rename = "FuelTypePrimary")]
    fuel_type: Option<String>,
    #[serde(rename = "TransmissionStyle")]
    transmission: Option<String>,
    #[serde(rename = "DriveType")]
    drive_type: Option<String>,
    #[serde(rename = "PlantCountry")]
    plant_country: Option<String>,
    #[serde(rename = "PlantCity")]
    plant_city: Option<String>,
    #[serde(rename = "Manufacturer")]
    manufacturer: Option<String>,
    #[serde(rename = "Series")]
    series: Option<String>,
    #[serde(rename = "Trim")]
    trim: Option<String>,
    #[serde(rename = "EngineModel")]
    engine_model: Option<String>,
}

fn nonempty(s: Option<String>) -> Option<String> {
    s.filter(|v| !v.trim().is_empty())
}

#[async_trait]
impl VehicleSource for NhtsaVpic {
    fn id(&self) -> &'static str {
        "nhtsa_vpic"
    }
    fn country(&self) -> &'static str {
        "XX"
    }
    fn name(&self) -> &'static str {
        "NHTSA vPIC"
    }
    fn cache_ttl(&self) -> Duration {
        Duration::from_secs(60 * 60 * 24 * 30) // 30 days — decode is static
    }

    async fn fetch_by_vin(&self, vin: &str) -> Result<SourceData, SourceError> {
        let url = format!(
            "https://vpic.nhtsa.dot.gov/api/vehicles/DecodeVinValues/{}?format=json",
            vin
        );
        let resp: VpicResponse = self
            .client
            .get(&url)
            .send()
            .await?
            .json()
            .await
            .map_err(|e| SourceError::Parse(e.to_string()))?;

        let r = resp
            .results
            .into_iter()
            .next()
            .ok_or(SourceError::NotFound)?;

        let year = nonempty(r.model_year.clone())
            .and_then(|y| y.parse::<u16>().ok());

        if r.make.as_deref().unwrap_or("").is_empty() && year.is_none() {
            return Err(SourceError::NotFound);
        }

        let wmi = vin.get(..3).unwrap_or("").to_string();

        let decode = VinDecode {
            make: nonempty(r.make),
            model: nonempty(r.model),
            year,
            body_style: nonempty(r.body_class),
            engine: nonempty(r.engine_model),
            fuel_type: nonempty(r.fuel_type),
            transmission: nonempty(r.transmission),
            drive_type: nonempty(r.drive_type),
            plant_country: nonempty(r.plant_country),
            plant_city: nonempty(r.plant_city),
            manufacturer: nonempty(r.manufacturer),
            series: nonempty(r.series),
            trim: nonempty(r.trim),
            wmi,
        };

        Ok(SourceData {
            decode: Some(decode),
            ..Default::default()
        })
    }
}
