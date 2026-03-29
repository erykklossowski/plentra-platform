import { getSummary, getFuels, getSpreads } from "@/lib/api";
import LiveBadge from "@/components/ui/LiveBadge";
import MetricCard from "@/components/ui/MetricCard";
import PrintButton from "@/components/ui/PrintButton";
import MarketSummaryModule from "@/components/summary/MarketSummaryModule";
import KeyIndicatorsTable from "@/components/summary/KeyIndicatorsTable";
import IndustrialSpreadMonitor from "@/components/summary/IndustrialSpreadMonitor";
import SpreadChart from "@/components/charts/SpreadChart";
import HistoricalChart from "@/components/charts/HistoricalChart";
import DataLoadingCard from "@/components/ui/DataLoadingCard";
import type { SummaryResponse, FuelsResponse, SpreadsResponse, ForwardPrice } from "@/types/api";

export const revalidate = 900;

export default async function SummaryPage() {
  const [summaryResult, fuelsResult, spreadsResult] = await Promise.allSettled([
    getSummary(),
    getFuels(),
    getSpreads(),
  ]);

  const summary = summaryResult.status === "fulfilled" ? summaryResult.value : null;
  const fuels = fuelsResult.status === "fulfilled" ? fuelsResult.value : null;
  const spreads = spreadsResult.status === "fulfilled" ? spreadsResult.value : null;

  if (!summary || summary.data_status === "unavailable") {
    return (
      <DataLoadingCard
        section="Summary"
        message={summary?.message ?? "Fetching from live sources — reload in 30s"}
      />
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
        <div className="flex items-center gap-3 no-print">
          <LiveBadge />
          <PrintButton />
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
          {fuels ? (
            <>
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
            </>
          ) : (
            <div className="bg-surface-container p-6 rounded-xl text-center">
              <span className="material-symbols-outlined text-2xl text-on-surface-variant mb-2">
                cloud_off
              </span>
              <p className="text-sm text-on-surface-variant">
                Fuel price data temporarily unavailable
              </p>
            </div>
          )}
        </div>
      </div>

      {/* Forward Term Prices */}
      {summary.forward_prices && summary.forward_prices.length > 0 && (
        <div className="bg-surface-container p-6 rounded-xl">
          <h2 className="font-headline text-lg font-bold text-on-surface mb-4">
            Forward Term Prices
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {summary.forward_prices.map((price: ForwardPrice) => (
              <div
                key={price.label}
                className="flex items-center justify-between p-4 bg-surface-container-low rounded-lg"
              >
                <div>
                  <p className="text-[10px] uppercase tracking-widest text-on-surface-variant font-label">
                    {price.label}
                  </p>
                  <p className="text-[9px] text-on-surface-variant/60">
                    {price.sublabel}
                  </p>
                </div>
                {price.available ? (
                  <div className="text-right">
                    <p className="text-xl font-headline font-bold text-on-surface">
                      {price.value_pln_mwh
                        ? `${price.value_pln_mwh.toFixed(1)} PLN`
                        : price.value_eur_mwh
                        ? `€${price.value_eur_mwh.toFixed(2)}`
                        : "—"}
                      <span className="text-sm text-on-surface-variant ml-1">
                        /MWh
                      </span>
                    </p>
                    {price.change_pct !== null && (
                      <span
                        className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium ${
                          price.change_pct > 0.05
                            ? "bg-emerald-500/10 text-emerald-400"
                            : price.change_pct < -0.05
                            ? "bg-error/10 text-error"
                            : "bg-surface-container-high text-on-surface-variant"
                        }`}
                      >
                        <span className="material-symbols-outlined text-[14px]">
                          {price.change_pct > 0.05
                            ? "arrow_upward"
                            : price.change_pct < -0.05
                            ? "arrow_downward"
                            : "arrow_forward"}
                        </span>
                        {price.change_pct > 0 ? "+" : ""}
                        {price.change_pct.toFixed(1)}%
                      </span>
                    )}
                  </div>
                ) : (
                  <p className="text-on-surface-variant text-xs">N/A</p>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Key Market Indicators */}
      {summary.key_indicators.length > 0 && (
        <KeyIndicatorsTable indicators={summary.key_indicators} />
      )}

      {/* Spread Chart — interactive history with date range selector */}
      <HistoricalChart
        endpoint="/api/history/spreads?"
        title="Clean Spark & Dark Spread History"
        yLabel="EUR/MWh"
        series={[
          { key: "css", label: "CSS (Gas η=60%)", color: "#76d6d5" },
          {
            key: "cds_42",
            label: "CDS (Coal η=42%)",
            color: "#ffb692",
            isDashed: true,
          },
        ]}
        showZeroLine
        defaultDays={30}
      />

      {/* Industrial Spread Monitor */}
      {summary.industrial_spread.baseload_eur_mwh !== undefined && (
        <IndustrialSpreadMonitor spread={summary.industrial_spread} />
      )}
    </div>
  );
}
