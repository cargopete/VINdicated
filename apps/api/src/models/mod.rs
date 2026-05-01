use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// Top-level response for a VIN lookup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleReport {
    pub vin: String,
    pub generated_at: DateTime<Utc>,
    pub decode: Option<VinDecode>,
    pub registrations: Vec<Registration>,
    pub inspections: Vec<Inspection>,
    pub recalls: Vec<Recall>,
    pub encumbrances: Vec<Encumbrance>,
    pub sources: Vec<SourceResult>,
}

impl VehicleReport {
    pub fn new(vin: impl Into<String>) -> Self {
        Self {
            vin: vin.into(),
            generated_at: Utc::now(),
            decode: None,
            registrations: vec![],
            inspections: vec![],
            recalls: vec![],
            encumbrances: vec![],
            sources: vec![],
        }
    }
}

/// Structural VIN decode (NHTSA vPIC + WMI fallback).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VinDecode {
    pub make: Option<String>,
    pub model: Option<String>,
    pub year: Option<u16>,
    pub body_style: Option<String>,
    pub engine: Option<String>,
    pub fuel_type: Option<String>,
    pub transmission: Option<String>,
    pub drive_type: Option<String>,
    pub plant_country: Option<String>,
    pub plant_city: Option<String>,
    pub manufacturer: Option<String>,
    pub wmi: String,
    pub series: Option<String>,
    pub trim: Option<String>,
}

/// A registration record from a national authority.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registration {
    pub country: String,
    pub plate: Option<String>,
    pub first_registered: Option<NaiveDate>,
    pub deregistered: Option<NaiveDate>,
    pub status: RegistrationStatus,
    pub color: Option<String>,
    pub fuel: Option<String>,
    pub body: Option<String>,
    pub engine_cc: Option<u32>,
    pub power_kw: Option<u32>,
    pub seats: Option<u8>,
    pub weight_kg: Option<u32>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegistrationStatus {
    Active,
    Deregistered,
    Exported,
    Stolen,
    Unknown,
}

/// A periodic-inspection event (MOT / APK / STK / CT / ITV etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inspection {
    pub country: String,
    pub date: NaiveDate,
    pub result: InspectionResult,
    pub mileage_km: Option<u64>,
    pub defects: Vec<String>,
    pub advisories: Vec<String>,
    pub expiry: Option<NaiveDate>,
    pub test_number: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InspectionResult {
    Pass,
    Fail,
    Advisory,
    Unknown,
}

/// A safety recall campaign.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recall {
    pub id: String,
    pub campaign_number: Option<String>,
    pub date: Option<NaiveDate>,
    pub description: String,
    pub component: Option<String>,
    pub remedy: Option<String>,
    pub status: RecallStatus,
    pub source: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecallStatus {
    Open,
    Remedied,
    Unknown,
}

/// A financial or administrative encumbrance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Encumbrance {
    pub kind: EncumbranceKind,
    pub description: Option<String>,
    pub country: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EncumbranceKind {
    Lien,
    Seizure,
    Stolen,
    TaxUnpaid,
    ExportRestriction,
    InsuranceFlag,
    Other,
}

/// Metadata about a source query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceResult {
    pub id: String,
    pub country: String,
    pub name: String,
    pub status: SourceStatus,
    pub queried_at: DateTime<Utc>,
    pub cached: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceStatus {
    Ok,
    NotFound,
    Error,
    RateLimited,
    Skipped,
}

/// Data returned by a single source adapter — merged into VehicleReport.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SourceData {
    pub registrations: Vec<Registration>,
    pub inspections: Vec<Inspection>,
    pub recalls: Vec<Recall>,
    pub encumbrances: Vec<Encumbrance>,
    pub decode: Option<VinDecode>,
}
