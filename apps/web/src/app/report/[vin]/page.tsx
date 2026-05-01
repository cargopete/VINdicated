import { fetchReport } from "@/lib/api";
import { VehicleReport, Inspection } from "@/types/report";
import Link from "next/link";

interface Props {
  params: Promise<{ vin: string }>;
}

export async function generateMetadata({ params }: Props) {
  const { vin } = await params;
  return {
    title: `${vin} — VINdicate`,
    description: `Free vehicle history report for VIN ${vin}`,
  };
}

function Badge({
  label,
  variant = "default",
}: {
  label: string;
  variant?: "default" | "success" | "warn" | "error" | "muted";
}) {
  const cls = {
    default: "bg-slate-100 text-slate-700 dark:bg-slate-800 dark:text-slate-300",
    success: "bg-emerald-100 text-emerald-800 dark:bg-emerald-900/40 dark:text-emerald-300",
    warn: "bg-amber-100 text-amber-800 dark:bg-amber-900/40 dark:text-amber-300",
    error: "bg-red-100 text-red-800 dark:bg-red-900/40 dark:text-red-300",
    muted: "bg-slate-50 text-slate-500 dark:bg-slate-900 dark:text-slate-400",
  }[variant];
  return (
    <span className={`text-xs font-semibold px-2.5 py-1 rounded-full ${cls}`}>
      {label}
    </span>
  );
}

function Card({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <section className="rounded-2xl border bg-white dark:bg-slate-900 shadow-sm overflow-hidden">
      <div className="px-6 py-4 border-b bg-slate-50 dark:bg-slate-800/50">
        <h2 className="font-semibold text-slate-900 dark:text-white">{title}</h2>
      </div>
      <div className="px-6 py-4">{children}</div>
    </section>
  );
}

function inspectionVariant(r: Inspection["result"]) {
  return r === "pass" ? "success" : r === "fail" ? "error" : "warn";
}

export default async function ReportPage({ params }: Props) {
  const { vin } = await params;
  const upperVin = vin.toUpperCase();

  let report: VehicleReport | null = null;
  let fetchError: string | null = null;

  try {
    report = await fetchReport(upperVin);
  } catch (e) {
    fetchError = e instanceof Error ? e.message : "Unknown error";
  }

  if (fetchError) {
    return (
      <main className="max-w-3xl mx-auto px-4 py-16 text-center">
        <h1 className="text-3xl font-bold text-red-600 mb-3">Lookup failed</h1>
        <p className="text-slate-600 dark:text-slate-400 mb-6">{fetchError}</p>
        <Link
          href="/"
          className="underline text-blue-600 hover:text-blue-800 dark:text-blue-400"
        >
          ← Back to search
        </Link>
      </main>
    );
  }

  if (!report) return null;

  const d = report.decode;
  const successSources = report.sources.filter((s) => s.status === "ok").length;
  const totalSources = report.sources.length;

  return (
    <main className="max-w-4xl mx-auto px-4 py-12 flex flex-col gap-6">
      {/* Back */}
      <Link
        href="/"
        className="text-sm text-slate-500 hover:text-blue-600 transition-colors"
      >
        ← New search
      </Link>

      {/* Header */}
      <div className="rounded-2xl border bg-white dark:bg-slate-900 shadow-sm p-6 flex flex-col gap-4">
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div>
            <p className="text-xs text-slate-400 dark:text-slate-500 font-mono uppercase tracking-widest mb-1">
              VIN
            </p>
            <h1 className="text-2xl font-bold font-mono tracking-wider text-slate-900 dark:text-white">
              {upperVin}
            </h1>
          </div>
          <div className="text-right">
            <p className="text-xs text-slate-400 mb-1">Sources queried</p>
            <p className="font-semibold text-slate-900 dark:text-white">
              {successSources}/{totalSources}
            </p>
          </div>
        </div>

        {d && (
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 pt-2 border-t">
            {[
              { label: "Make", value: d.make },
              { label: "Model", value: d.model },
              { label: "Year", value: d.year },
              { label: "Body", value: d.body_style },
              { label: "Fuel", value: d.fuel_type },
              { label: "Transmission", value: d.transmission },
              { label: "Plant", value: [d.plant_city, d.plant_country].filter(Boolean).join(", ") },
              { label: "WMI", value: d.wmi },
            ]
              .filter((f) => f.value)
              .map((f) => (
                <div key={f.label}>
                  <p className="text-xs text-slate-400 dark:text-slate-500">{f.label}</p>
                  <p className="text-sm font-medium text-slate-900 dark:text-white">
                    {String(f.value)}
                  </p>
                </div>
              ))}
          </div>
        )}
      </div>

      {/* Recalls */}
      {report.recalls.length > 0 && (
        <Card title={`Safety Recalls (${report.recalls.length})`}>
          <ul className="divide-y">
            {report.recalls.map((r, i) => (
              <li key={i} className="py-3 first:pt-0 last:pb-0">
                <div className="flex flex-wrap items-start gap-2 mb-1">
                  <Badge
                    label={r.status === "open" ? "Open" : r.status === "remedied" ? "Remedied" : "Unknown"}
                    variant={r.status === "open" ? "error" : r.status === "remedied" ? "success" : "muted"}
                  />
                  {r.campaign_number && (
                    <span className="text-xs text-slate-400 font-mono">
                      {r.campaign_number}
                    </span>
                  )}
                </div>
                <p className="text-sm text-slate-700 dark:text-slate-300">
                  {r.description}
                </p>
                {r.component && (
                  <p className="text-xs text-slate-400 mt-0.5">
                    Component: {r.component}
                  </p>
                )}
              </li>
            ))}
          </ul>
        </Card>
      )}

      {/* Encumbrances */}
      {report.encumbrances.length > 0 && (
        <Card title="Encumbrances & Flags">
          <ul className="flex flex-col gap-2">
            {report.encumbrances.map((e, i) => (
              <li key={i} className="flex items-center gap-2">
                <Badge label={e.kind.replace("_", " ")} variant="error" />
                <span className="text-sm text-slate-600 dark:text-slate-400">
                  {e.description} ({e.country})
                </span>
              </li>
            ))}
          </ul>
        </Card>
      )}

      {/* Registrations */}
      {report.registrations.length > 0 && (
        <Card title="Registration Records">
          <div className="grid sm:grid-cols-2 gap-4">
            {report.registrations.map((r, i) => (
              <div key={i} className="rounded-xl border p-4 flex flex-col gap-1">
                <div className="flex items-center justify-between">
                  <span className="font-semibold text-slate-900 dark:text-white">
                    {r.country}
                    {r.plate && (
                      <span className="ml-2 font-mono text-sm text-blue-600 dark:text-blue-400">
                        {r.plate}
                      </span>
                    )}
                  </span>
                  <Badge
                    label={r.status}
                    variant={
                      r.status === "active"
                        ? "success"
                        : r.status === "stolen"
                        ? "error"
                        : "muted"
                    }
                  />
                </div>
                {r.first_registered && (
                  <p className="text-xs text-slate-400">
                    First registered: {r.first_registered}
                  </p>
                )}
                {r.color && (
                  <p className="text-xs text-slate-400">Colour: {r.color}</p>
                )}
                {r.fuel && (
                  <p className="text-xs text-slate-400">Fuel: {r.fuel}</p>
                )}
              </div>
            ))}
          </div>
        </Card>
      )}

      {/* Inspections */}
      {report.inspections.length > 0 && (
        <Card title={`Inspection History (${report.inspections.length} tests)`}>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="text-left text-xs text-slate-400 border-b">
                  <th className="pb-2 pr-4 font-medium">Date</th>
                  <th className="pb-2 pr-4 font-medium">Country</th>
                  <th className="pb-2 pr-4 font-medium">Result</th>
                  <th className="pb-2 pr-4 font-medium">Mileage</th>
                  <th className="pb-2 font-medium">Issues</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100 dark:divide-slate-800">
                {report.inspections.map((ins, i) => (
                  <tr key={i} className="align-top">
                    <td className="py-2 pr-4 tabular-nums text-slate-700 dark:text-slate-300">
                      {ins.date}
                    </td>
                    <td className="py-2 pr-4 text-slate-500">{ins.country}</td>
                    <td className="py-2 pr-4">
                      <Badge
                        label={ins.result}
                        variant={inspectionVariant(ins.result)}
                      />
                    </td>
                    <td className="py-2 pr-4 tabular-nums text-slate-600 dark:text-slate-400">
                      {ins.mileage_km != null
                        ? `${ins.mileage_km.toLocaleString()} km`
                        : "—"}
                    </td>
                    <td className="py-2 text-slate-500">
                      {ins.defects.length > 0
                        ? ins.defects.slice(0, 2).join("; ")
                        : "—"}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Card>
      )}

      {/* Sources */}
      <Card title="Data Sources">
        <div className="grid grid-cols-2 sm:grid-cols-3 gap-2">
          {report.sources.map((s) => (
            <div key={s.id} className="flex items-center gap-2">
              <span
                className={`w-2 h-2 rounded-full flex-shrink-0 ${
                  s.status === "ok"
                    ? "bg-emerald-500"
                    : s.status === "not_found"
                    ? "bg-slate-300"
                    : s.status === "skipped"
                    ? "bg-slate-200"
                    : s.status === "rate_limited"
                    ? "bg-amber-400"
                    : "bg-red-400"
                }`}
              />
              <span className="text-xs text-slate-600 dark:text-slate-400 truncate">
                {s.name}
                {s.cached && (
                  <span className="ml-1 text-slate-300 dark:text-slate-600">
                    ↩
                  </span>
                )}
              </span>
            </div>
          ))}
        </div>
      </Card>
    </main>
  );
}
