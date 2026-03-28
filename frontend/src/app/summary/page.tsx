import { getSummary, getFuels, getSpreads } from "@/lib/api";
import LiveBadge from "@/components/ui/LiveBadge";
import MetricCard from "@/components/ui/MetricCard";
import MarketSummaryModule from "@/components/summary/MarketSummaryModule";
import KeyIndicatorsTable from "@/components/summary/KeyIndicatorsTable";
import IndustrialSpreadMonitor from "@/components/summary/IndustrialSpreadMonitor";
import SpreadChart from "@/components/charts/SpreadChart";

export const revalidate = 900;

export default async function SummaryPage() {
  let summary, fuels, spreads;

  try {
    [summary, fuels, spreads] = await Promise.all([
      getSummary(),
      getFuels(),
      getSpreads(),
    ]);
  } catch {
    return (
      <div className="p-8">
        <div className="bg-surface-container p-6 rounded-xl text-center">
          <span className="material-symbols-outlined text-4xl text-error mb-2">
            error
          </span>
          <h2 className="font-headline text-lg font-bold text-on-surface">
            Unable to load market data
          </h2>
          <p className="text-sm text-on-surface-variant mt-2">
            Please ensure the backend is running and try refreshing the page.
          </p>
        </div>
      </div>
    );
  }

  const now = new Date();
  const monthName = now.toLocaleString("en-US", {
    month: "long",
    year: "numeric",
  });

  return (
    <div className="p-8 space-y-8">
      {/* Page Header */}
      <div className="flex items-start justify-between">
        <div>
          <h1 className="font-headline text-2xl font-bold text-on-surface">
            Market Intelligence {monthName} Summary
          </h1>
          <p className="text-sm text-on-surface-variant mt-1">
            Comprehensive analysis of European energy market dynamics
          </p>
        </div>
        <div className="flex items-center gap-3">
          <LiveBadge />
          <button className="bg-linear-to-br from-primary to-primary-container text-on-primary px-4 py-1.5 rounded-lg font-semibold hover:opacity-90 transition-opacity text-sm">
            Export PDF
          </button>
          <button className="bg-surface-container-high text-on-surface px-4 py-1.5 rounded-lg font-medium hover:opacity-90 transition-opacity text-sm">
            Share Report
          </button>
        </div>
      </div>

      {/* Monthly Market Summary - Two Column */}
      <div className="grid grid-cols-12 gap-6">
        {/* Left Column */}
        <div className="col-span-12 lg:col-span-7">
          <MarketSummaryModule summary={summary} />
        </div>

        {/* Right Column */}
        <div className="col-span-12 lg:col-span-5 space-y-6">
          {/* Fuel Price Cards */}
          <MetricCard
            label="Natural Gas (TTF)"
            value={`€${fuels.ttf_eur_mwh.toFixed(2)}`}
            unit="/MWh"
            delta={fuels.ttf_change_pct}
            history={fuels.ttf_history_30d}
          />
          <MetricCard
            label="Coal ARA"
            value={`$${fuels.ara_usd_tonne.toFixed(2)}`}
            unit="/tonne"
            delta={fuels.ara_change_pct}
            history={fuels.ara_history_30d}
          />
          <MetricCard
            label="EUA Carbon"
            value={`€${fuels.eua_eur_tonne.toFixed(2)}`}
            unit="/tonne"
            delta={fuels.eua_change_pct}
            history={fuels.eua_history_30d}
          />
        </div>
      </div>

      {/* Key Market Indicators */}
      <KeyIndicatorsTable indicators={summary.key_indicators} />

      {/* Spread Chart */}
      {spreads.history_30d.length > 0 && (
        <div className="bg-surface-container p-6 rounded-xl">
          <h2 className="font-headline text-lg font-bold text-on-surface mb-4">
            Clean Spark & Dark Spread History (30d)
          </h2>
          <SpreadChart data={spreads.history_30d} />
        </div>
      )}

      {/* Industrial Spread Monitor */}
      <IndustrialSpreadMonitor spread={summary.industrial_spread} />
    </div>
  );
}
