import { VinSearchForm } from "@/components/vin-search-form";
import { CountryCoverage } from "@/components/country-coverage";

export default function Home() {
  return (
    <main className="flex flex-col flex-1">
      {/* Hero */}
      <section className="flex flex-col items-center justify-center gap-8 px-4 py-24 text-center bg-gradient-to-b from-blue-50 to-white dark:from-slate-900 dark:to-slate-950">
        <div className="flex flex-col items-center gap-3">
          <span className="text-xs font-semibold tracking-widest uppercase text-blue-600 dark:text-blue-400">
            100% Free · Open Source · No Account Required
          </span>
          <h1 className="text-5xl font-bold tracking-tight text-slate-900 dark:text-white max-w-2xl leading-tight">
            The car&apos;s past —{" "}
            <span className="text-blue-600 dark:text-blue-400">VINdicated</span>
          </h1>
          <p className="text-lg text-slate-600 dark:text-slate-400 max-w-xl">
            Instant vehicle history reports from official government sources
            across 40+ European countries. Mileage records, inspection history,
            recalls, and encumbrances — all free.
          </p>
        </div>

        <VinSearchForm />

        <p className="text-sm text-slate-400 dark:text-slate-500">
          Example:{" "}
          <a
            href="/report/WBAJB9109ED784158"
            className="underline hover:text-blue-600 transition-colors"
          >
            WBAJB9109ED784158
          </a>
        </p>
      </section>

      {/* What we check */}
      <section className="max-w-5xl mx-auto w-full px-4 py-16">
        <h2 className="text-2xl font-bold text-center mb-10 text-slate-900 dark:text-white">
          What VINdicate checks
        </h2>
        <div className="grid grid-cols-2 sm:grid-cols-3 gap-4">
          {[
            {
              icon: "📋",
              title: "Registration",
              desc: "Official national registration records",
            },
            {
              icon: "🔧",
              title: "Inspection history",
              desc: "MOT, APK, STK, CT and equivalent results",
            },
            {
              icon: "📏",
              title: "Mileage records",
              desc: "Odometer readings at every inspection",
            },
            {
              icon: "⚠️",
              title: "Safety recalls",
              desc: "NHTSA, EU Safety Gate, and OEM campaigns",
            },
            {
              icon: "🔒",
              title: "Encumbrances",
              desc: "Liens, seizures, stolen flags, and restrictions",
            },
            {
              icon: "🌍",
              title: "40+ countries",
              desc: "Pan-European coverage from official sources",
            },
          ].map((item) => (
            <div
              key={item.title}
              className="flex flex-col gap-1 rounded-xl border p-5 bg-white dark:bg-slate-900 shadow-sm"
            >
              <span className="text-2xl">{item.icon}</span>
              <span className="font-semibold text-slate-900 dark:text-white">
                {item.title}
              </span>
              <span className="text-sm text-slate-500 dark:text-slate-400">
                {item.desc}
              </span>
            </div>
          ))}
        </div>
      </section>

      {/* Country coverage */}
      <section className="bg-slate-50 dark:bg-slate-900/50 py-16">
        <div className="max-w-5xl mx-auto px-4">
          <h2 className="text-2xl font-bold text-center mb-2 text-slate-900 dark:text-white">
            Data sources by country
          </h2>
          <p className="text-center text-slate-500 dark:text-slate-400 mb-10">
            Official government APIs and open datasets — no scraping of paid
            services.
          </p>
          <CountryCoverage />
        </div>
      </section>

      {/* Footer */}
      <footer className="border-t py-8 text-center text-sm text-slate-400 dark:text-slate-500">
        <p>
          VINdicate is open source.{" "}
          <a
            href="https://github.com/cargopete/VINdicated"
            className="underline hover:text-blue-600 transition-colors"
            target="_blank"
            rel="noopener noreferrer"
          >
            View on GitHub
          </a>{" "}
          · Data sourced from official government registries. Not a substitute
          for professional advice.
        </p>
      </footer>
    </main>
  );
}
