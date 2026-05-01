use axum::{
    extract::{Path, State},
    Json,
};
use chrono::Utc;
use futures::future::join_all;
use std::sync::Arc;

use crate::{
    cache::Cache,
    error::ApiError,
    models::{SourceResult, SourceStatus, VehicleReport},
    AppState,
};

fn validate_vin(vin: &str) -> Result<(), ApiError> {
    if vin.len() != 17 {
        return Err(ApiError::InvalidVin(format!(
            "VIN must be 17 characters, got {}",
            vin.len()
        )));
    }
    if vin
        .chars()
        .any(|c| matches!(c, 'I' | 'O' | 'Q') || !c.is_ascii_alphanumeric())
    {
        return Err(ApiError::InvalidVin(
            "VIN contains invalid characters (I, O, Q not allowed)".into(),
        ));
    }
    Ok(())
}

pub async fn lookup_vin(
    Path(vin): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<VehicleReport>, ApiError> {
    let vin = vin.to_uppercase();
    validate_vin(&vin)?;

    // Check full-report cache first
    let report_key = format!("report:{}", &vin);
    if let Some(cached) = state.cache.get(&report_key).await {
        if let Ok(report) = serde_json::from_str::<VehicleReport>(&cached) {
            return Ok(Json(report));
        }
    }

    let mut report = VehicleReport::new(&vin);

    // Fan out to all sources concurrently
    let tasks: Vec<_> = state
        .sources
        .sources
        .iter()
        .map(|source| {
            let vin = vin.clone();
            let source = Arc::clone(source);
            let cache = Arc::clone(&state.cache);
            tokio::spawn(async move {
                let cache_key = Cache::vin_key(source.id(), &vin);
                let queried_at = Utc::now();

                // Check per-source cache
                if let Some(cached) = cache.get(&cache_key).await {
                    if let Ok(data) =
                        serde_json::from_str::<crate::models::SourceData>(&cached)
                    {
                        return (
                            source.id(),
                            source.country(),
                            source.name(),
                            Ok(data),
                            queried_at,
                            true,
                        );
                    }
                }

                let result = source.fetch_by_vin(&vin).await;
                if let Ok(ref data) = result {
                    if let Ok(json) = serde_json::to_string(data) {
                        let _ = cache.set(&cache_key, &json, source.cache_ttl()).await;
                    }
                }
                (
                    source.id(),
                    source.country(),
                    source.name(),
                    result,
                    queried_at,
                    false,
                )
            })
        })
        .collect();

    let results = join_all(tasks).await;

    for task_result in results {
        let Ok((id, country, name, source_result, queried_at, cached)) = task_result else {
            continue;
        };

        let (status, error) = match &source_result {
            Ok(_) => (SourceStatus::Ok, None),
            Err(e) => match e {
                crate::error::SourceError::NotFound => (SourceStatus::NotFound, None),
                crate::error::SourceError::RateLimited => {
                    (SourceStatus::RateLimited, Some("Rate limited".into()))
                }
                crate::error::SourceError::NotSupported => (SourceStatus::Skipped, None),
                crate::error::SourceError::Unavailable(msg) => {
                    (SourceStatus::Skipped, Some(msg.clone()))
                }
                e => (SourceStatus::Error, Some(e.to_string())),
            },
        };

        report.sources.push(SourceResult {
            id: id.into(),
            country: country.into(),
            name: name.into(),
            status,
            queried_at,
            cached,
            error,
        });

        if let Ok(data) = source_result {
            if report.decode.is_none() {
                report.decode = data.decode;
            }
            report.registrations.extend(data.registrations);
            report.inspections.extend(data.inspections);
            report.recalls.extend(data.recalls);
            report.encumbrances.extend(data.encumbrances);
        }
    }

    // Sort inspections chronologically descending
    report.inspections.sort_by(|a, b| b.date.cmp(&a.date));

    // Cache assembled report for 1 hour
    if let Ok(json) = serde_json::to_string(&report) {
        let _ = state
            .cache
            .set(
                &report_key,
                &json,
                std::time::Duration::from_secs(3600),
            )
            .await;
    }

    Ok(Json(report))
}
