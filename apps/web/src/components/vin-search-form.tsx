"use client";

import { useState, FormEvent } from "react";
import { useRouter } from "next/navigation";

export function VinSearchForm() {
  const router = useRouter();
  const [vin, setVin] = useState("");
  const [error, setError] = useState<string | null>(null);

  function handleSubmit(e: FormEvent) {
    e.preventDefault();
    const cleaned = vin.trim().toUpperCase();
    if (cleaned.length !== 17) {
      setError("A VIN is exactly 17 characters.");
      return;
    }
    if (/[IOQ]/.test(cleaned) || !/^[A-HJ-NPR-Z0-9]{17}$/.test(cleaned)) {
      setError("VIN contains invalid characters (I, O, Q not allowed).");
      return;
    }
    setError(null);
    router.push(`/report/${cleaned}`);
  }

  return (
    <form onSubmit={handleSubmit} className="w-full max-w-xl flex flex-col gap-2">
      <div className="flex gap-2">
        <input
          type="text"
          value={vin}
          onChange={(e) => {
            setVin(e.target.value.toUpperCase());
            setError(null);
          }}
          placeholder="Enter VIN — e.g. WBAJB9109ED784158"
          maxLength={17}
          spellCheck={false}
          className="flex-1 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 px-4 py-3 text-base font-mono tracking-widest text-slate-900 dark:text-white placeholder:text-slate-400 placeholder:tracking-normal placeholder:font-sans focus:outline-none focus:ring-2 focus:ring-blue-500 shadow-sm"
        />
        <button
          type="submit"
          className="rounded-xl bg-blue-600 hover:bg-blue-700 active:bg-blue-800 text-white font-semibold px-6 py-3 shadow-sm transition-colors"
        >
          Check
        </button>
      </div>
      {error && (
        <p className="text-sm text-red-600 dark:text-red-400 px-1">{error}</p>
      )}
    </form>
  );
}
