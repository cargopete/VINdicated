use async_trait::async_trait;
use serde::Deserialize;
use std::time::Duration;

use crate::error::SourceError;
use crate::models::{Recall, RecallStatus, SourceData};
use crate::sources::VehicleSource;

pub struct NhtsaRecalls {
    client: reqwest::Client,
}

impl NhtsaRecalls {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[derive(Debug, Deserialize)]
struct RecallsResponse {
    #[serde(rename = "Count")]
    count: usize,
    #[serde(rename = "results")]
    results: Option<Vec<RecallResult>>,
}

#[derive(Debug, Deserialize)]
struct RecallResult {
    #[serde(rename = "NHTSACampaignNumber")]
    campaign_number: Option<String>,
    #[serde(rename = "ReportReceivedDate")]
    report_date: Option<String>,
    #[serde(rename = "Subject")]
    subject: Option<String>,
    #[serde(rename = "Summary")]
    summary: Option<String>,
    #[serde(rename = "Remedy")]
    remedy: Option<String>,
    #[serde(rename = "Component")]
    component: Option<String>,
}

#[async_trait]
impl VehicleSource for NhtsaRecalls {
    fn id(&self) -> &'static str {
        "nhtsa_recalls"
    }
    fn country(&self) -> &'static str {
        "XX"
    }
    fn name(&self) -> &'static str {
        "NHTSA Recalls"
    }
    fn cache_ttl(&self) -> Duration {
        Duration::from_secs(60 * 60 * 24 * 7) // 7 days
    }

    async fn fetch_by_vin(&self, vin: &str) -> Result<SourceData, SourceError> {
        // NHTSA recalls are by make/model/year — we need decode first.
        // For direct VIN lookup we use the complaints/recall endpoint.
        let url = format!(
            "https://api.nhtsa.gov/recalls/recallsByVehicle?vin={}",
            vin
        );
        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(SourceError::Unavailable(resp.status().to_string()));
        }
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| SourceError::Parse(e.to_string()))?;

        let results = match body.get("results").and_then(|v| v.as_array()) {
            Some(arr) => arr.clone(),
            None => return Ok(SourceData::default()),
        };

        let recalls: Vec<Recall> = results
            .iter()
            .filter_map(|r| {
                let campaign = r["NHTSACampaignNumber"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                let description = r["Summary"]
                    .as_str()
                    .or(r["Subject"].as_str())
                    .unwrap_or("No description")
                    .to_string();
                Some(Recall {
                    id: campaign.clone(),
                    campaign_number: Some(campaign),
                    date: r["ReportReceivedDate"]
                        .as_str()
                        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()),
                    description,
                    component: r["Component"].as_str().map(|s| s.to_string()),
                    remedy: r["Remedy"].as_str().map(|s| s.to_string()),
                    status: RecallStatus::Unknown,
                    source: "nhtsa_recalls".into(),
                    url: None,
                })
            })
            .collect();

        Ok(SourceData {
            recalls,
            ..Default::default()
        })
    }
}
