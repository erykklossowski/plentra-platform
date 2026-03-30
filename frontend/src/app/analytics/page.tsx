import { getSpreadsAnalytics, getEveningAnalytics } from "@/lib/api";
import {
  SpreadHistoryChartWrapper as SpreadHistoryChart,
  SeasonalityChartWrapper as SeasonalityChart,
  PositiveDaysChartWrapper as PositiveDaysChart,
  EveningDecompositionChartWrapper as EveningDecompositionChart,
} from "@/components/analytics/AnalyticsCharts";
import type {
  SpreadsAnalyticsResponse,
  EveningAnalyticsResponse,
} from "@/types/api";

export const revalidate = 900;

export default async function AnalyticsPage() {
  const [spreadsResult, eveningResult] = await Promise.allSettled([
    getSpreadsAnalytics(),
    getEveningAnalytics(),
  ]);

  const spreads = spreadsResult.status === "fulfilled" ? spreadsResult.value : null;
  const evening = eveningResult.status === "fulfilled" ? eveningResult.value : null;

  return (
    <div className="p-8 space-y-12">
      {/* Page Header */}
      <div>
        <h1 className="font-headline text-2xl font-bold text-on-surface">
          Analytics
        </h1>
        <p className="text-sm text-on-surface-variant mt-1">
          Analiza spreadów CSS/CDS oraz dekompozycja ceny wieczornej DA
        </p>
      </div>

      {/* Section A: CSS/CDS Analytics */}
      <SpreadAnalyticsSection data={spreads} />

      {/* Section B: Evening Decomposition */}
      <EveningDecompositionSection data={evening} />
    </div>
  );
}

/* ─── Section A: CSS/CDS Analytics ─── */

function SpreadAnalyticsSection({ data }: { data: SpreadsAnalyticsResponse | null }) {
  if (!data) {
    return <EmptyState icon="analytics" message="Spread analytics unavailable — run backfill" />;
  }

  // KPI: latest CSS and CDS values
  const cssHistory = data.history_90d.filter((d) => d.spread_type === "rolling_3m_css");
  const cdsHistory = data.history_90d.filter((d) => d.spread_type === "rolling_3m_cds");
  const latestCss = cssHistory.at(-1);
  const latestCds = cdsHistory.at(-1);

  // % positive last 30 days
  const recentPositive = data.positive_days
    .filter((d) => d.spread_type === "rolling_3m_css")
    .at(-1);

  const cssCdsSpread =
    latestCss && latestCds ? latestCss.value - latestCds.value : null;

  return (
    <div className="space-y-6">
      <div>
        <h2 className="font-headline text-lg font-bold text-on-surface">
          Clean Spark & Dark Spread Analytics
        </h2>
        <p className="text-[10px] uppercase tracking-widest text-on-surface-variant mt-1">
          Historyczna analiza spreadów dla inwestorów w aktywa gazowe
        </p>
      </div>

      {/* KPI Cards */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <KpiCard
          label="CSS (latest)"
          value={latestCss?.value}
          sub={latestCss?.rolling_30d_avg != null ? `30d avg: ${latestCss.rolling_30d_avg.toFixed(2)}` : undefined}
          unit="EUR/MWh"
          positive={latestCss ? latestCss.value > 0 : null}
        />
        <KpiCard
          label="CDS (latest)"
          value={latestCds?.value}
          sub={latestCds?.rolling_30d_avg != null ? `30d avg: ${latestCds.rolling_30d_avg.toFixed(2)}` : undefined}
          unit="EUR/MWh"
          positive={latestCds ? latestCds.value > 0 : null}
        />
        <KpiCard
          label="CCGT rentowny (CSS > 0)"
          value={recentPositive?.positive_pct}
          sub={recentPositive
            ? recentPositive.positive_pct === 0
              ? `CCGT poza merit order — 0/${recentPositive.total_days} dni`
              : `${recentPositive.positive_days}/${recentPositive.total_days} dni`
            : undefined}
          unit="%"
          positive={recentPositive ? recentPositive.positive_pct >= 50 : null}
        />
        <KpiCard
          label="CSS - CDS delta"
          value={cssCdsSpread}
          sub="spread między spreadami"
          unit="EUR/MWh"
          positive={cssCdsSpread != null ? cssCdsSpread > 0 : null}
        />
      </div>

      {/* A1: 90-day time series */}
      <div className="bg-surface-container p-6 rounded-xl">
        <h3 className="font-headline text-base font-bold text-on-surface mb-4">
          CSS & CDS — 90-dniowa historia
        </h3>
        <SpreadHistoryChart data={data.history_90d} />
      </div>

      {/* A2: Monthly seasonality */}
      <div className="bg-surface-container p-6 rounded-xl">
        <h3 className="font-headline text-base font-bold text-on-surface mb-4">
          Sezonowość CSS — ostatnie 12 miesięcy
        </h3>
        <SeasonalityChart data={data.seasonality} />
      </div>

      {/* A3: % days CSS positive */}
      <div className="bg-surface-container p-6 rounded-xl">
        <h3 className="font-headline text-base font-bold text-on-surface mb-4">
          % dni z dodatnim CSS (CCGT rentowny)
        </h3>
        <PositiveDaysChart data={data.positive_days} />
      </div>
    </div>
  );
}

/* ─── Section B: Evening Decomposition ─── */

function EveningDecompositionSection({ data }: { data: EveningAnalyticsResponse | null }) {
  if (!data || data.decomposition.length === 0) {
    return <EmptyState icon="nightlight" message="Evening decomposition unavailable — awaiting price + generation data" />;
  }

  const decomp = data.decomposition;
  const last30 = decomp.slice(-30);

  const avgEvening = avg(last30.map((d) => d.evening_avg_pln));
  const avgFuel = avg(last30.map((d) => d.delta_fuel_pln));
  const avgOze = avg(last30.map((d) => d.delta_oze_pln));
  const avgResidual = avg(last30.map((d) => d.residual_pln));

  return (
    <div className="space-y-6">
      <div>
        <h2 className="font-headline text-lg font-bold text-on-surface">
          Dekompozycja Ceny Wieczornej (17:00–21:00 CET)
        </h2>
        <p className="text-[10px] uppercase tracking-widest text-on-surface-variant mt-1">
          Składowe ceny szczytowej: baseline · paliwa · OZE displacement · residual
        </p>
      </div>

      {/* KPI Summary */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <KpiCard label="Avg evening price (30d)" value={avgEvening} unit="PLN/MWh" positive={null} />
        <KpiCard
          label="Fuel contribution"
          value={data.summary.avg_css_contribution_pct}
          unit="% of price"
          positive={null}
        />
        <KpiCard label="OZE displacement (30d)" value={avgOze} unit="PLN/MWh" positive={null} />
        <KpiCard label="Residual (30d)" value={avgResidual} unit="PLN/MWh" positive={null} />
      </div>

      {/* Stacked area chart */}
      <div className="bg-surface-container p-6 rounded-xl">
        <h3 className="font-headline text-base font-bold text-on-surface mb-4">
          Składowe ceny wieczornej — 90 dni
        </h3>
        <EveningDecompositionChart data={decomp} />
      </div>
    </div>
  );
}

/* ─── Shared helpers ─── */

function EmptyState({ icon, message }: { icon: string; message: string }) {
  return (
    <div className="bg-surface-container p-6 rounded-xl text-center">
      <span className="material-symbols-outlined text-outline text-3xl">{icon}</span>
      <p className="text-sm text-on-surface-variant mt-2">{message}</p>
    </div>
  );
}

function KpiCard({
  label,
  value,
  sub,
  unit,
  positive,
}: {
  label: string;
  value: number | null | undefined;
  sub?: string;
  unit: string;
  positive: boolean | null;
}) {
  const bgColor =
    positive === true
      ? "bg-emerald-500/5 border-l-emerald-500"
      : positive === false
      ? "bg-error/5 border-l-error"
      : "bg-surface-container-low border-l-primary";

  return (
    <div className={`p-4 rounded-lg border-l-4 ${bgColor}`}>
      <p className="text-[10px] uppercase tracking-widest text-on-surface-variant font-label">
        {label}
      </p>
      <p className="font-headline text-xl font-bold text-on-surface mt-1">
        {value != null ? value.toFixed(2) : "—"}
        <span className="text-sm text-on-surface-variant ml-1">{unit}</span>
      </p>
      {sub && (
        <p className="text-[10px] text-on-surface-variant/60 mt-0.5">{sub}</p>
      )}
    </div>
  );
}

function avg(arr: number[]): number {
  if (arr.length === 0) return 0;
  return arr.reduce((a, b) => a + b, 0) / arr.length;
}
