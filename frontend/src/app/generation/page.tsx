import { getGeneration, getSpreads } from "@/lib/api";
import DispatchSignalBadge from "@/components/generation/DispatchSignalBadge";
import JKZTable from "@/components/generation/JKZTable";
import SpreadChart from "@/components/charts/SpreadChart";
import HistoricalChart from "@/components/charts/HistoricalChart";
import DataLoadingCard from "@/components/ui/DataLoadingCard";

export const revalidate = 300;

export default async function GenerationPage() {
  const [genResult, spreadResult] = await Promise.allSettled([
    getGeneration(),
    getSpreads(),
  ]);

  const gen = genResult.status === "fulfilled" ? genResult.value : null;
  const spreads = spreadResult.status === "fulfilled" ? spreadResult.value : null;

  if (!gen || gen.data_status === "unavailable") {
    return (
      <DataLoadingCard
        section="Generation"
        message={gen?.message ?? "Fetching from live sources — reload in 30s"}
      />
    );
  }

  return (
    <div className="p-8 space-y-8">
      {/* Header */}
      <div className="flex items-start justify-between">
        <div>
          <h1 className="font-headline text-2xl font-bold text-on-surface">
            Generation Economics
          </h1>
          <p className="text-sm text-on-surface-variant mt-1">
            Unit variable costs, dispatch signals, and clean spread analysis
          </p>
        </div>
        <div className="flex items-center gap-3">
          <span className="bg-surface-container-high px-3 py-1.5 rounded text-on-surface-variant flex items-center gap-2 text-xs">
            <span className="w-2 h-2 rounded-full bg-primary animate-pulse" />
            Live Data Active
          </span>
          <button className="bg-linear-to-br from-primary to-primary-container text-on-primary px-4 py-1.5 rounded-lg font-semibold hover:opacity-90 transition-opacity text-sm">
            Export PDF
          </button>
        </div>
      </div>

      {/* Dispatch Signal + Market Context */}
      <DispatchSignalBadge
        signal={gen.dispatch_signal}
        daPrice={gen.rdn_eur_mwh}
        eurUsd={gen.eur_usd_rate}
      />

      {/* JKZ Table */}
      <JKZTable data={gen.jkz_table} daPrice={gen.rdn_eur_mwh} />

      {/* Spread KPIs */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <div className="bg-surface-container p-5 rounded-xl">
          <p className="text-[0.625rem] uppercase tracking-widest text-on-surface-variant">
            Clean Spark Spread
          </p>
          <p
            className={`font-headline text-xl font-bold mt-2 ${gen.css_spot >= 0 ? "text-emerald-400" : "text-error"}`}
          >
            €{gen.css_spot.toFixed(2)}
          </p>
          <p className="text-xs text-on-surface-variant mt-1">/MWh</p>
        </div>
        <div className="bg-surface-container p-5 rounded-xl">
          <p className="text-[0.625rem] uppercase tracking-widest text-on-surface-variant">
            Clean Dark Spread (η42)
          </p>
          <p
            className={`font-headline text-xl font-bold mt-2 ${gen.cds_spot_eta42 >= 0 ? "text-emerald-400" : "text-error"}`}
          >
            €{gen.cds_spot_eta42.toFixed(2)}
          </p>
          <p className="text-xs text-on-surface-variant mt-1">/MWh</p>
        </div>
        <div className="bg-surface-container p-5 rounded-xl">
          <p className="text-[0.625rem] uppercase tracking-widest text-on-surface-variant">
            DA Price (RDN)
          </p>
          <p className="font-headline text-xl font-bold mt-2 text-on-surface">
            €{gen.rdn_eur_mwh.toFixed(2)}
          </p>
          <p className="text-xs text-on-surface-variant mt-1">/MWh</p>
        </div>
        <div className="bg-surface-container p-5 rounded-xl">
          <p className="text-[0.625rem] uppercase tracking-widest text-on-surface-variant">
            EUR/USD Rate
          </p>
          <p className="font-headline text-xl font-bold mt-2 text-on-surface">
            {gen.eur_usd_rate.toFixed(4)}
          </p>
          <p className="text-xs text-on-surface-variant mt-1">live FX</p>
        </div>
      </div>

      {/* Spread History Chart */}
      {spreads && spreads.history_30d.length > 0 && (
        <div className="bg-surface-container p-6 rounded-xl">
          <h2 className="font-headline text-lg font-bold text-on-surface mb-4">
            Clean Spark & Dark Spread History (30d)
          </h2>
          <SpreadChart data={spreads.history_30d} />
        </div>
      )}

      {/* TTF Fuel Price History */}
      <HistoricalChart
        endpoint="/api/history/fuels?ticker=TTF"
        title="Natural Gas (TTF)"
        yLabel="EUR/MWh"
        series={[{ key: "avg", label: "TTF Close", color: "#76d6d5" }]}
        defaultDays={90}
      />
    </div>
  );
}
