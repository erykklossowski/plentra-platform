import { cn } from "@/lib/utils";
import SparklineBar from "@/components/charts/SparklineBar";

interface MetricCardProps {
  label: string;
  sublabel?: string;
  value: string;
  unit?: string;
  delta: number;
  history?: number[];
  className?: string;
}

export default function MetricCard({
  label,
  sublabel,
  value,
  unit,
  delta,
  history,
  className,
}: MetricCardProps) {
  return (
    <div className={cn("bg-surface-container p-6 rounded-xl", className)}>
      <p className="text-[0.6875rem] uppercase tracking-widest text-on-surface-variant">
        {label}
      </p>
      {sublabel && (
        <p className="text-[0.625rem] uppercase tracking-widest text-on-surface-variant/60 mt-0.5">
          {sublabel}
        </p>
      )}
      <div className="mt-3 flex items-baseline gap-2">
        <span className="font-headline text-2xl font-extrabold text-on-surface">
          {value}
        </span>
        {unit && (
          <span className="text-sm text-on-surface-variant">{unit}</span>
        )}
      </div>
      <div className="mt-2">
        <DeltaChip value={delta} />
      </div>
      {history && history.length > 0 && (
        <div className="mt-4">
          <SparklineBar data={history} />
        </div>
      )}
    </div>
  );
}

function DeltaChip({ value }: { value: number }) {
  const isPositive = value >= 0;
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium",
        isPositive
          ? "bg-emerald-500/10 text-emerald-400"
          : "bg-error/10 text-error"
      )}
    >
      <span className="material-symbols-outlined text-[14px]">
        {isPositive ? "arrow_upward" : "arrow_downward"}
      </span>
      {isPositive ? "+" : ""}
      {value.toFixed(1)}%
    </span>
  );
}
