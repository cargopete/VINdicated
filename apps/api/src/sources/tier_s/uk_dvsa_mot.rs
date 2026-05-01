use async_trait::async_trait;
use serde::Deserialize;
use std::time::Duration;

use crate::error::SourceError;
use crate::models::{Inspection, InspectionResult, SourceData};
use crate::sources::VehicleSource;

/// DVSA MOT History API v2 (UK + NI)
/// https://documentation.history.mot.api.gov.uk/
pub struct UkDvsaMot {
    client: reqwest::Client,
    client_id: Option<String>,
    client_secret: Option<String>,
    token: tokio::sync::Mutex<Option<TokenCache>>,
}

struct TokenCache {
    access_token: String,
    expires_at: std::time::Instant,
}

impl UkDvsaMot {
    pub fn new(
        client: reqwest::Client,
        client_id: Option<String>,
        client_secret: Option<String>,
    ) -> Self {
        Self {
            client,
            client_id,
            client_secret,
            token: tokio::sync::Mutex::new(None),
        }
    }

    async fn get_token(&self) -> Result<String, SourceError> {
        let mut cache = self.token.lock().await;
        if let Some(ref t) = *cache {
            if t.expires_at > std::time::Instant::now() {
                return Ok(t.access_token.clone());
            }
        }
        let (cid, csecret) = match (&self.client_id, &self.client_secret) {
            (Some(a), Some(b)) => (a.clone(), b.clone()),
            _ => {
                return Err(SourceError::Unavailable(
                    "No DVSA credentials configured".into(),
                ))
            }
        };
        let resp = self
            .client
            .post("https://login.microsoftonline.com/a455b827-244d-4b72-b19c-0996a712dc8f/oauth2/v2.0/token")
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", &cid),
                ("client_secret", &csecret),
                ("scope", "https://tapi.dvsa.gov.uk/.default"),
            ])
            .send()
            .await?
            .json::<serde_json::Value>()
            .await
            .map_err(|e| SourceError::Parse(e.to_string()))?;

        let token = resp["access_token"]
            .as_str()
            .ok_or_else(|| SourceError::Parse("No access_token in DVSA response".into()))?
            .to_string();
        let expires_in = resp["expires_in"].as_u64().unwrap_or(3600);

        *cache = Some(TokenCache {
            access_token: token.clone(),
            expires_at: std::time::Instant::now()
                + std::time::Duration::from_secs(expires_in.saturating_sub(60)),
        });
        Ok(token)
    }
}

#[derive(Debug, Deserialize)]
struct MotResponse {
    registration: Option<String>,
    make: Option<String>,
    model: Option<String>,
    #[serde(rename = "motTests")]
    mot_tests: Option<Vec<MotTest>>,
}

#[derive(Debug, Deserialize)]
struct MotTest {
    #[serde(rename = "completedDate")]
    completed_date: Option<String>,
    #[serde(rename = "testResult")]
    test_result: Option<String>,
    #[serde(rename = "odometerValue")]
    odometer_value: Option<String>,
    #[serde(rename = "odometerUnit")]
    odometer_unit: Option<String>,
    #[serde(rename = "motTestNumber")]
    mot_test_number: Option<String>,
    #[serde(rename = "expiryDate")]
    expiry_date: Option<String>,
    #[serde(rename = "rfrAndComments")]
    rfr_and_comments: Option<Vec<RfrComment>>,
}

#[derive(Debug, Deserialize)]
struct RfrComment {
    text: Option<String>,
    #[serde(rename = "type")]
    kind: Option<String>,
}

#[async_trait]
impl VehicleSource for UkDvsaMot {
    fn id(&self) -> &'static str {
        "uk_dvsa_mot"
    }
    fn country(&self) -> &'static str {
        "GB"
    }
    fn name(&self) -> &'static str {
        "DVSA MOT History"
    }
    fn cache_ttl(&self) -> Duration {
        Duration::from_secs(60 * 60 * 24)
    }

    async fn fetch_by_vin(&self, vin: &str) -> Result<SourceData, SourceError> {
        let token = self.get_token().await?;
        let url = format!(
            "https://history.mot.api.gov.uk/v1/trade/vehicles/registration/{}",
            vin
        );
        let resp = self
            .client
            .get(&url)
            .bearer_auth(&token)
            .header("X-API-Key", "vindicateapp")
            .send()
            .await?;

        match resp.status().as_u16() {
            200 => {}
            404 => return Err(SourceError::NotFound),
            429 => return Err(SourceError::RateLimited),
            s => return Err(SourceError::Unavailable(format!("HTTP {}", s))),
        }

        let data: MotResponse = resp
            .json()
            .await
            .map_err(|e| SourceError::Parse(e.to_string()))?;

        let tests = data.mot_tests.unwrap_or_default();
        let inspections: Vec<Inspection> = tests
            .into_iter()
            .filter_map(|t| {
                let date = t.completed_date.as_deref().and_then(|d| {
                    // Format: "2023-11-01T09:00:00.000Z"
                    chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()
                })?;

                let result = match t.test_result.as_deref() {
                    Some("PASSED") => InspectionResult::Pass,
                    Some("FAILED") => InspectionResult::Fail,
                    _ => InspectionResult::Unknown,
                };

                let mut mileage_km = t
                    .odometer_value
                    .as_deref()
                    .and_then(|s| s.parse::<u64>().ok());
                // Convert miles to km if needed
                if t.odometer_unit.as_deref() == Some("mi") {
                    mileage_km = mileage_km.map(|m| (m as f64 * 1.60934) as u64);
                }

                let rfr = t.rfr_and_comments.unwrap_or_default();
                let defects: Vec<String> = rfr
                    .iter()
                    .filter(|r| r.kind.as_deref() == Some("FAIL"))
                    .filter_map(|r| r.text.clone())
                    .collect();
                let advisories: Vec<String> = rfr
                    .iter()
                    .filter(|r| r.kind.as_deref() == Some("ADVISORY"))
                    .filter_map(|r| r.text.clone())
                    .collect();

                let expiry = t.expiry_date.as_deref().and_then(|d| {
                    chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()
                });

                Some(Inspection {
                    country: "GB".into(),
                    date,
                    result,
                    mileage_km,
                    defects,
                    advisories,
                    expiry,
                    test_number: t.mot_test_number,
                    source: "uk_dvsa_mot".into(),
                })
            })
            .collect();

        Ok(SourceData {
            inspections,
            ..Default::default()
        })
    }
}
