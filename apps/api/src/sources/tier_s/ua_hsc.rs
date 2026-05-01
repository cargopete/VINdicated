use async_trait::async_trait;
use std::time::Duration;

use crate::error::SourceError;
use crate::models::{Registration, RegistrationStatus, SourceData};
use crate::sources::VehicleSource;

/// Ukraine — HSC/MVS Open Data portal
/// https://opendata.hsc.gov.ua/
/// Bulk CSV updated daily; we query the search API endpoint.
pub struct UaHsc {
    client: reqwest::Client,
}

impl UaHsc {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VehicleSource for UaHsc {
    fn id(&self) -> &'static str {
        "ua_hsc"
    }
    fn country(&self) -> &'static str {
        "UA"
    }
    fn name(&self) -> &'static str {
        "MVS Open Data (UA)"
    }
    fn cache_ttl(&self) -> Duration {
        Duration::from_secs(60 * 60 * 24)
    }

    async fn fetch_by_vin(&self, vin: &str) -> Result<SourceData, SourceError> {
        // HSC open data search API — VIN search available since 2021+
        let url = format!(
            "https://opendata.hsc.gov.ua/api/v1/vehicle/search?vin={}",
            vin
        );
        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
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

        let results = match body.get("data").and_then(|d| d.as_array()) {
            Some(r) if !r.is_empty() => r.clone(),
            _ => return Err(SourceError::NotFound),
        };

        let registrations: Vec<Registration> = results
            .iter()
            .filter_map(|r| {
                let first_registered = r["d_reg"]
                    .as_str()
                    .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok());

                Some(Registration {
                    country: "UA".into(),
                    plate: r["n_reg_new"].as_str().map(|s| s.to_string()),
                    first_registered,
                    deregistered: None,
                    status: RegistrationStatus::Unknown,
                    color: r["color"].as_str().map(|s| s.to_string()),
                    fuel: r["fuel"].as_str().map(|s| s.to_string()),
                    body: r["body"].as_str().map(|s| s.to_string()),
                    engine_cc: r["capacity"].as_u64().map(|v| v as u32),
                    power_kw: None,
                    seats: None,
                    weight_kg: None,
                    source: self.id().into(),
                })
            })
            .collect();

        if registrations.is_empty() {
            return Err(SourceError::NotFound);
        }

        Ok(SourceData {
            registrations,
            ..Default::default()
        })
    }
}
