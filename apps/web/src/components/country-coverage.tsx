const COUNTRIES = [
  { code: "NL", name: "Netherlands", tier: "S", source: "RDW Open Data" },
  { code: "GB", name: "United Kingdom", tier: "S", source: "DVLA + DVSA MOT" },
  { code: "DK", name: "Denmark", tier: "S", source: "DMR Motorregister" },
  { code: "SE", name: "Sweden", tier: "S", source: "Transportstyrelsen" },
  { code: "NO", name: "Norway", tier: "S", source: "Statens vegvesen" },
  { code: "PL", name: "Poland", tier: "S", source: "HistoriaPojazdu" },
  { code: "CZ", name: "Czech Republic", tier: "S", source: "Kontrola Tachometru" },
  { code: "SK", name: "Slovakia", tier: "S", source: "STKonline" },
  { code: "SI", name: "Slovenia", tier: "S", source: "Avtolog" },
  { code: "EE", name: "Estonia", tier: "S", source: "Transpordiamet" },
  { code: "LV", name: "Latvia", tier: "S", source: "CSDD" },
  { code: "UA", name: "Ukraine", tier: "S", source: "HSC Open Data" },
  { code: "HR", name: "Croatia", tier: "A", source: "HAK/CVH" },
  { code: "LT", name: "Lithuania", tier: "A", source: "Regitra" },
  { code: "IS", name: "Iceland", tier: "A", source: "Samgöngustofa" },
  { code: "ES", name: "Spain", tier: "A", source: "DGT" },
  { code: "IT", name: "Italy", tier: "A", source: "ACI / Portale Automobilista" },
  { code: "FR", name: "France", tier: "B", source: "HistoVec (owner-share)" },
  { code: "DE", name: "Germany", tier: "B", source: "KBA Recalls" },
  { code: "BG", name: "Bulgaria", tier: "B", source: "Marketplace data" },
];

const TIER_LABEL: Record<string, string> = {
  S: "Full coverage",
  A: "Good coverage",
  B: "Partial coverage",
};

const TIER_COLOR: Record<string, string> = {
  S: "bg-emerald-100 text-emerald-800 dark:bg-emerald-900/40 dark:text-emerald-300",
  A: "bg-blue-100 text-blue-800 dark:bg-blue-900/40 dark:text-blue-300",
  B: "bg-amber-100 text-amber-800 dark:bg-amber-900/40 dark:text-amber-300",
};

export function CountryCoverage() {
  return (
    <div className="space-y-4">
      <div className="flex gap-3 flex-wrap justify-center mb-6">
        {Object.entries(TIER_LABEL).map(([tier, label]) => (
          <span
            key={tier}
            className={`text-xs font-semibold px-3 py-1 rounded-full ${TIER_COLOR[tier]}`}
          >
            Tier {tier} — {label}
          </span>
        ))}
      </div>
      <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-3">
        {COUNTRIES.map((c) => (
          <div
            key={c.code}
            className="flex flex-col gap-1 rounded-xl border bg-white dark:bg-slate-900 p-3 shadow-sm"
          >
            <div className="flex items-center justify-between">
              <span className="font-semibold text-slate-900 dark:text-white text-sm">
                {c.name}
              </span>
              <span
                className={`text-xs font-bold px-2 py-0.5 rounded-full ${TIER_COLOR[c.tier]}`}
              >
                {c.tier}
              </span>
            </div>
            <span className="text-xs text-slate-500 dark:text-slate-400">
              {c.source}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}
