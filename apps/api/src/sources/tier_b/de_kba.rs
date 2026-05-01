use async_trait::async_trait;
use std::time::Duration;

use crate::error::SourceError;
use crate::models::{Recall, RecallStatus, SourceData};
use crate::sources::VehicleSource;

/// Germany — KBA Rückrufdatenbank (Federal Motor Transport Authority recall database)
/// https://www.kba.de/rueckrufe/
/// Free VIN-based recall lookup. German manufacturer-reported recalls not always in EU Safety Gate.
pub struct DeKba {
    client: reqwest::Client,
}

impl DeKba {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VehicleSource for DeKba {
    fn id(&self) -> &'static str {
        "de_kba"
    }
    fn country(&self) -> &'static str {
        "DE"
    }
    fn name(&self) -> &'static str {
        "KBA Rückrufe (DE)"
    }
    fn cache_ttl(&self) -> Duration {
        Duration::from_secs(60 * 60 * 24 * 7)
    }

    async fn fetch_by_vin(&self, vin: &str) -> Result<SourceData, SourceError> {
        // KBA public recall check API
        let url = format!(
            "https://www.kba.de/DE/ZentraleRegister/KraftfahrtBundesamt/RueckrufDatenbank/rueckruf_api/vin/{}",
            vin
        );
        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .header("Referer", "https://www.kba.de/")
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

        let empty = vec![];
        let rueckrufe = body["rueckrufe"].as_array().unwrap_or(&empty);

        if rueckrufe.is_empty() && body.get("rueckrufe").is_none() {
            return Err(SourceError::NotFound);
        }

        let recalls: Vec<Recall> = rueckrufe
            .iter()
            .filter_map(|r| {
                let id = r["referenzNummer"].as_str()?.to_string();
                let description = r["mangelbeschreibung"].as_str().unwrap_or("").to_string();
                Some(Recall {
                    id,
                    campaign_number: r["rueckrufNummer"].as_str().map(|s| s.to_string()),
                    date: r["bekanntmachungsDatum"]
                        .as_str()
                        .and_then(|d| chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d").ok()),
                    description,
                    component: r["betroffenesBauteil"].as_str().map(|s| s.to_string()),
                    remedy: r["abhilfemassnahme"].as_str().map(|s| s.to_string()),
                    status: match r["status"].as_str() {
                        Some("OFFEN") => RecallStatus::Open,
                        Some("ABGESCHLOSSEN") => RecallStatus::Remedied,
                        _ => RecallStatus::Unknown,
                    },
                    source: self.id().into(),
                    url: r["url"].as_str().map(|s| s.to_string()),
                })
            })
            .collect();

        Ok(SourceData {
            recalls,
            ..Default::default()
        })
    }
}
