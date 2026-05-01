export type RegistrationStatus = "active" | "deregistered" | "exported" | "stolen" | "unknown";
export type InspectionResult = "pass" | "fail" | "advisory" | "unknown";
export type RecallStatus = "open" | "remedied" | "unknown";
export type EncumbranceKind = "lien" | "seizure" | "stolen" | "tax_unpaid" | "export_restriction" | "insurance_flag" | "other";
export type SourceStatus = "ok" | "not_found" | "error" | "rate_limited" | "skipped";

export interface VinDecode {
  make?: string;
  model?: string;
  year?: number;
  body_style?: string;
  engine?: string;
  fuel_type?: string;
  transmission?: string;
  drive_type?: string;
  plant_country?: string;
  plant_city?: string;
  manufacturer?: string;
  wmi: string;
  series?: string;
  trim?: string;
}

export interface Registration {
  country: string;
  plate?: string;
  first_registered?: string;
  deregistered?: string;
  status: RegistrationStatus;
  color?: string;
  fuel?: string;
  body?: string;
  engine_cc?: number;
  power_kw?: number;
  seats?: number;
  weight_kg?: number;
  source: string;
}

export interface Inspection {
  country: string;
  date: string;
  result: InspectionResult;
  mileage_km?: number;
  defects: string[];
  advisories: string[];
  expiry?: string;
  test_number?: string;
  source: string;
}

export interface Recall {
  id: string;
  campaign_number?: string;
  date?: string;
  description: string;
  component?: string;
  remedy?: string;
  status: RecallStatus;
  source: string;
  url?: string;
}

export interface Encumbrance {
  kind: EncumbranceKind;
  description?: string;
  country: string;
  source: string;
}

export interface SourceResult {
  id: string;
  country: string;
  name: string;
  status: SourceStatus;
  queried_at: string;
  cached: boolean;
  error?: string;
}

export interface VehicleReport {
  vin: string;
  generated_at: string;
  decode?: VinDecode;
  registrations: Registration[];
  inspections: Inspection[];
  recalls: Recall[];
  encumbrances: Encumbrance[];
  sources: SourceResult[];
}
