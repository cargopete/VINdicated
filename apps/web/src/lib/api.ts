import type { VehicleReport } from "@/types/report";

// Server-side only — called from Server Components on Vercel.
// Set API_URL in Vercel environment variables.
const API_URL = process.env.API_URL ?? "http://localhost:8080";

export async function fetchReport(vin: string): Promise<VehicleReport> {
  const res = await fetch(`${API_URL}/v1/vin/${vin}`, {
    next: { revalidate: 3600 },
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: "Unknown error" }));
    throw new Error(err.error ?? `HTTP ${res.status}`);
  }
  return res.json();
}
