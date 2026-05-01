import type { VehicleReport } from "@/types/report";

const API_BASE = process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:8080";

export async function fetchReport(vin: string): Promise<VehicleReport> {
  const res = await fetch(`${API_BASE}/v1/vin/${vin}`, {
    next: { revalidate: 3600 },
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: "Unknown error" }));
    throw new Error(err.error ?? `HTTP ${res.status}`);
  }
  return res.json();
}
