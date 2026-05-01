use async_trait::async_trait;
use std::time::Duration;

use crate::error::SourceError;
use crate::models::{Inspection, InspectionResult, SourceData};
use crate::sources::VehicleSource;

/// Czech Republic — Ministerstvo dopravy odometer / STK check
/// https://www.kontrolatachometru.cz/
pub struct CzKontrolaTachometru {
    client: reqwest::Client,
}

impl CzKontrolaTachometru {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VehicleSource for CzKontrolaTachometru {
    fn id(&self) -> &'static str {
        "cz_kontrolatachometru"
    }
    fn country(&self) -> &'static str {
        "CZ"
    }
    fn name(&self) -> &'static str {
        "Kontrola Tachometru (CZ)"
    }
    fn cache_ttl(&self) -> Duration {
        Duration::from_secs(60 * 60 * 24)
    }

    async fn fetch_by_vin(&self, vin: &str) -> Result<SourceData, SourceError> {
        let url = format!(
            "https://www.kontrolatachometru.cz/api/v2/vehicle/{}",
            vin
        );
        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .header("Referer", "https://www.kontrolatachometru.cz/")
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

        let records = match body.get("data").and_then(|d| d.as_array()) {
            Some(r) if !r.is_empty() => r.clone(),
            _ => return Err(SourceError::NotFound),
        };

        let inspections: Vec<Inspection> = records
            .iter()
            .filter_map(|r| {
                let date_str = r["date"].as_str()?;
                let date = chrono::NaiveDate::parse_from_str(&date_str[..10], "%Y-%m-%d").ok()?;

                let mileage_raw = r["mileage"]
                    .as_u64()
                    .or_else(|| r["mileage"].as_str().and_then(|s| s.parse().ok()));
                let mileage_km = mileage_raw.map(|m| {
                    if r["mileageUnit"].as_str() == Some("mi") {
                        (m as f64 * 1.60934) as u64
                    } else {
                        m
                    }
                });

                let result = match r["testResult"].as_str() {
                    Some("PASSED") | Some("OK") => InspectionResult::Pass,
                    Some("FAILED") | Some("NOK") => InspectionResult::Fail,
                    _ => InspectionResult::Unknown,
                };

                Some(Inspection {
                    country: "CZ".into(),
                    date,
                    result,
                    mileage_km,
                    defects: vec![],
                    advisories: vec![],
                    expiry: None,
                    test_number: None,
                    source: self.id().into(),
                })
            })
            .collect();

        if inspections.is_empty() {
            return Err(SourceError::NotFound);
        }

        Ok(SourceData {
            inspections,
            ..Default::default()
        })
    }
}
