import { getCrossBorder } from "@/lib/api";
import SpreadProfileChart from "@/components/crossborder/SpreadProfileChart";
import DataLoadingCard from "@/components/ui/DataLoadingCard";

export const revalidate = 3600;

export default async function CrossBorderPage() {
  const [result] = await Promise.allSettled([getCrossBorder()]);
  const data = result.status === "fulfilled" ? result.value : null;

  if (!data || data.data_status === "unavailable") {
    return (
      <DataLoadingCard
        section="Cross-Border"
        message={data?.message ?? "Fetching from live sources — reload in 30s"}
      />
    );
  }

  const spreadColor =
    data.spread_eur_mwh >= 0 ? "text-error" : "text-emerald-400";
  const flowIcon =
    data.flow_direction === "IMPORT" ? "south_west" : "north_east";
  const flowLabel =
    data.flow_direction === "IMPORT"
      ? "Net Import (DE→PL)"
      : "Net Export (PL→DE)";

  return (
    <div className="p-8 space-y-8">
      {/* Header */}
      <div className="flex items-start justify-between">
        <div>
          <h1 className="font-headline text-2xl font-bold text-on-surface">
            Cross-Border Analysis
          </h1>
          <p className="text-sm text-on-surface-variant mt-1">
            Poland–Germany interconnector spread and flow analysis
          </p>
        </div>
        <div className="flex items-center gap-3">
          <span className="bg-surface-container-high px-3 py-1.5 rounded text-on-surface-variant flex items-center gap-2 text-xs">
            <span className="w-2 h-2 rounded-full bg-primary animate-pulse" />
            Live Data Active
          </span>
        </div>
      </div>

      {/* KPIs */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <div className="bg-surface-container p-5 rounded-xl">
          <p className="text-[0.625rem] uppercase tracking-widest text-on-surface-variant">
            Poland DA Price
          </p>
          <p className="font-headline text-xl font-bold mt-2 text-on-surface">
            €{data.pl_da_eur_mwh.toFixed(2)}
          </p>
          <p className="text-xs text-on-surface-variant mt-1">/MWh</p>
        </div>
        <div className="bg-surface-container p-5 rounded-xl">
          <p className="text-[0.625rem] uppercase tracking-widest text-on-surface-variant">
            Germany DA Price
          </p>
          <p className="font-headline text-xl font-bold mt-2 text-on-surface">
            €{data.de_da_eur_mwh.toFixed(2)}
          </p>
          <p className="text-xs text-on-surface-variant mt-1">/MWh</p>
        </div>
        <div className="bg-surface-container p-5 rounded-xl">
          <p className="text-[0.625rem] uppercase tracking-widest text-on-surface-variant">
            PL-DE Spread
          </p>
          <p className={`font-headline text-xl font-bold mt-2 ${spreadColor}`}>
            {data.spread_eur_mwh >= 0 ? "+" : ""}€
            {data.spread_eur_mwh.toFixed(2)}
          </p>
          <p className="text-xs text-on-surface-variant mt-1">
            {data.spread_direction.replace("_", " ")}
          </p>
        </div>
        <div className="bg-surface-container p-5 rounded-xl">
          <p className="text-[0.625rem] uppercase tracking-widest text-on-surface-variant">
            Interconnector Util.
          </p>
          <p className="font-headline text-xl font-bold mt-2 text-on-surface">
            {data.interconnector_utilization_pct.toFixed(1)}%
          </p>
          <p className="text-xs text-on-surface-variant mt-1">estimated</p>
        </div>
      </div>

      {/* Flow Direction */}
      <div className="bg-surface-container p-6 rounded-xl">
        <div className="flex items-center gap-4">
          <span className="material-symbols-outlined text-3xl text-primary">
            {flowIcon}
          </span>
          <div>
            <p className="text-[0.625rem] uppercase tracking-widest text-on-surface-variant">
              Dominant Flow Direction
            </p>
            <p className="font-headline text-lg font-bold text-on-surface">
              {flowLabel}
            </p>
          </div>
          <div className="ml-auto text-right">
            <p className="text-[0.625rem] uppercase tracking-widest text-on-surface-variant">
              30d Avg Spread
            </p>
            <p className="font-headline text-lg font-bold text-on-surface">
              €{data.avg_spread_30d.toFixed(2)}
            </p>
          </div>
        </div>
      </div>

      {/* Spread Profile Chart */}
      {data.hourly_profile.length > 0 && (
        <div className="bg-surface-container p-6 rounded-xl">
          <h2 className="font-headline text-lg font-bold text-on-surface mb-4">
            24-Hour DA Price Profile: Poland vs Germany
          </h2>
          <SpreadProfileChart data={data.hourly_profile} />
        </div>
      )}
    </div>
  );
}
