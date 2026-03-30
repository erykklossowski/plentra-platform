import { getSummary, getFuels, getSpreads, getPsePrices } from "@/lib/api";
import LiveBadge from "@/components/ui/LiveBadge";
import MetricCard from "@/components/ui/MetricCard";
import PrintButton from "@/components/ui/PrintButton";
import MarketSummaryModule from "@/components/summary/MarketSummaryModule";
import KeyIndicatorsTable from "@/components/summary/KeyIndicatorsTable";
import IndustrialSpreadMonitor from "@/components/summary/IndustrialSpreadMonitor";
import SpreadChart from "@/components/charts/SpreadChart";
import HistoricalChart from "@/components/charts/HistoricalChart";
import PseHistoricalChart from "@/components/charts/PseHistoricalChart";
import DataLoadingCard from "@/components/ui/DataLoadingCard";
import type { SummaryResponse, FuelsResponse, SpreadsResponse, ForwardPrice, PsePricesResponse } from "@/types/api";

export const revalidate = 900;

export default async function SummaryPage() {
  const [summaryResult, fuelsResult, spreadsResult, psePricesResult] = await Promise.allSettled([
    getSummary(),
    getFuels(),
    getSpreads(),
    getPsePrices(30),
  ]);

  const summary = summaryResult.status === "fulfilled" ? summaryResult.value : null;
  const fuels = fuelsResult.status === "fulfilled" ? fuelsResult.value : null;
  const spreads = spreadsResult.status === "fulfilled" ? spreadsResult.value : null;
  const psePrices = psePricesResult.status === "fulfilled" ? psePricesResult.value : null;

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

      {/* Polish Day-Ahead Market */}
      <PolishMarketSection psePrices={psePrices} />

      {/* Key Market Indicators */}
      {summary.key_indicators.length > 0 && (
        <KeyIndicatorsTable indicators={summary.key_indicators} />
      )}

      {/* Spread Chart — interactive history with date range selector */}
      <HistoricalChart
        endpoint="/api/history/spreads"
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
        defaultDays={365}
      />

      {/* Industrial Spread Monitor */}
      {summary.industrial_spread.baseload_eur_mwh !== undefined && (
        <IndustrialSpreadMonitor spread={summary.industrial_spread} />
      )}
    </div>
  );
}

/* ─── Polish Market Section (server component helper) ─── */

function PolishMarketSection({ psePrices }: { psePrices: PsePricesResponse | null }) {
  if (!psePrices || !psePrices.series) {
    return (
      <div className="bg-surface-container p-6 rounded-xl text-center">
        <span className="material-symbols-outlined text-outline text-3xl">analytics</span>
        <p className="text-sm text-on-surface-variant mt-2">
          Dane w trakcie generowania. Sprawdź ponownie za chwilę.
        </p>
      </div>
    );
  }

  const cenPoints = psePrices.series.cen ?? [];
  const ckoebPoints = psePrices.series.ckoeb ?? [];
  const sdacPoints = psePrices.series.sdac ?? [];

  // Latest values (last non-null)
  const lastCen = [...cenPoints].reverse().find(p => p.value != null)?.value;
  const lastCkoeb = [...ckoebPoints].reverse().find(p => p.value != null)?.value;
  const lastSdac = [...sdacPoints].reverse().find(p => p.value != null)?.value;

  // 7-day ago values (approximate: 7 days × 24 hours = 168 points back for daily buckets)
  const cenOld = cenPoints.length > 7 ? cenPoints[cenPoints.length - 8]?.value : null;
  const ckoebOld = ckoebPoints.length > 7 ? ckoebPoints[ckoebPoints.length - 8]?.value : null;
  const sdacOld = sdacPoints.length > 7 ? sdacPoints[sdacPoints.length - 8]?.value : null;

  const pctChange = (latest: number | null | undefined, old: number | null | undefined) => {
    if (!latest || !old || old === 0) return null;
    return ((latest - old) / Math.abs(old)) * 100;
  };

  const cenDelta = pctChange(lastCen, cenOld);
  const ckoebDelta = pctChange(lastCkoeb, ckoebOld);
  const sdacDelta = pctChange(lastSdac, sdacOld);

  return (
    <div className="bg-surface-container p-6 rounded-xl space-y-6">
      <div>
        <h2 className="font-headline text-lg font-bold text-on-surface">
          Polish Day-Ahead Market
        </h2>
        <p className="text-[10px] uppercase tracking-widest text-on-surface-variant mt-1">
          CEN · CKOEB · SDAC — źródło: PSE api.raporty.pse.pl
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <PseCard label="CEN (Rozliczenie)" value={lastCen} delta={cenDelta} color="primary" />
        <PseCard label="CKOEB (Bilansowanie)" value={lastCkoeb} delta={ckoebDelta} color="amber" />
        <PseCard label="SDAC (Coupling)" value={lastSdac} delta={sdacDelta} color="slate" />
      </div>

      <PseHistoricalChart
        title="CEN — Cena Rozliczeniowa DA"
        yLabel="PLN/MWh"
        seriesKey="cen"
        color="#76d6d5"
        defaultDays={30}
      />
    </div>
  );
}

function PseCard({
  label,
  value,
  delta,
  color,
}: {
  label: string;
  value: number | null | undefined;
  delta: number | null;
  color: string;
}) {
  const colorMap: Record<string, string> = {
    primary: "border-l-primary",
    amber: "border-l-amber-500",
    slate: "border-l-slate-400",
  };

  return (
    <div className={`bg-surface-container-low p-4 rounded-lg border-l-4 ${colorMap[color] ?? "border-l-primary"}`}>
      <p className="text-[10px] uppercase tracking-widest text-on-surface-variant font-label">
        {label}
      </p>
      <div className="mt-2 flex items-baseline gap-2">
        <span className="font-headline text-xl font-bold text-on-surface">
          {value != null ? value.toFixed(1) : "—"}
        </span>
        <span className="text-sm text-on-surface-variant">PLN/MWh</span>
      </div>
      {delta != null && (
        <span
          className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium mt-1 ${
            delta > 0.05
              ? "bg-emerald-500/10 text-emerald-400"
              : delta < -0.05
              ? "bg-error/10 text-error"
              : "bg-surface-container-high text-on-surface-variant"
          }`}
        >
          <span className="material-symbols-outlined text-[14px]">
            {delta > 0.05 ? "arrow_upward" : delta < -0.05 ? "arrow_downward" : "arrow_forward"}
          </span>
          {delta > 0 ? "+" : ""}{delta.toFixed(1)}% 7d
        </span>
      )}
    </div>
  );
}
