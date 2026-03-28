import { cn } from "@/lib/utils";
import type { IndustrialSpread } from "@/types/api";

interface Props {
  spread: IndustrialSpread;
}

interface SpreadTileProps {
  label: string;
  value: number;
  changePct: number;
  unit: string;
  color: "primary" | "error";
}

function SpreadTile({ label, value, changePct, unit, color }: SpreadTileProps) {
  const isPositive = changePct >= 0;
  const barColor = color === "primary" ? "bg-primary" : "bg-error";
  const barWidth = Math.min(Math.abs(value) * 2, 100);

  return (
    <div className="bg-surface-container-lowest p-5 rounded-xl">
      <p className="text-[0.6875rem] uppercase tracking-widest text-on-surface-variant mb-3">
        {label}
      </p>
      <div className="flex items-baseline gap-2">
        <span className="font-headline text-2xl font-extrabold text-on-surface">
          €{Math.abs(value).toFixed(2)}
        </span>
        <span className="text-sm text-on-surface-variant">{unit}</span>
      </div>
      <span
        className={cn(
          "inline-flex items-center gap-1 mt-2 px-2 py-0.5 rounded-full text-xs font-medium",
          isPositive
            ? "bg-emerald-500/10 text-emerald-400"
            : "bg-error/10 text-error"
        )}
      >
        <span className="material-symbols-outlined text-[14px]">
          {isPositive ? "arrow_upward" : "arrow_downward"}
        </span>
        {isPositive ? "+" : ""}
        {changePct.toFixed(1)}%
      </span>
      <div className="mt-3 h-1 bg-surface-container-high rounded-full overflow-hidden">
        <div
          className={cn("h-full rounded-full", barColor)}
          style={{ width: `${barWidth}%` }}
        />
      </div>
    </div>
  );
}

export default function IndustrialSpreadMonitor({ spread }: Props) {
  return (
    <div className="bg-surface-container p-6 rounded-xl">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="font-headline text-lg font-bold text-on-surface">
            Industrial Spread Monitor
          </h2>
          <p className="text-sm text-on-surface-variant mt-0.5">
            Clean Spark & Dark Spreads
          </p>
        </div>
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-1.5">
            <span className="w-2 h-2 rounded-full bg-primary" />
            <span className="text-xs text-on-surface-variant">
              CSS Efficiency
            </span>
          </div>
          <div className="flex items-center gap-1.5">
            <span className="w-2 h-2 rounded-full bg-tertiary" />
            <span className="text-xs text-on-surface-variant">
              CDS Efficiency
            </span>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <SpreadTile
          label="Base-load Profitability"
          value={spread.baseload_eur_mwh}
          changePct={spread.baseload_change_pct}
          unit="/MWh"
          color="primary"
        />
        <SpreadTile
          label="Peak Load Advantage"
          value={spread.peak_eur_mwh}
          changePct={spread.peak_change_pct}
          unit="/MWh"
          color="primary"
        />
        <SpreadTile
          label="Carbon Impact Factor"
          value={spread.carbon_impact_eur_mwh}
          changePct={spread.carbon_change_pct}
          unit="/MWh"
          color="error"
        />
      </div>
    </div>
  );
}
