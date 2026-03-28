import { cn } from "@/lib/utils";
import type { ForwardSignal } from "@/types/api";

interface Props {
  signals: ForwardSignal[];
}

function DirectionArrow({ direction }: { direction: string }) {
  const config = {
    UP: { icon: "arrow_upward", color: "text-emerald-400" },
    DOWN: { icon: "arrow_downward", color: "text-error" },
    FLAT: { icon: "arrow_forward", color: "text-on-surface-variant" },
  }[direction] ?? { icon: "arrow_forward", color: "text-on-surface-variant" };

  return (
    <span className={cn("material-symbols-outlined text-[18px]", config.color)}>
      {config.icon}
    </span>
  );
}

function ConvictionStars({ conviction }: { conviction: number }) {
  return (
    <div className="flex gap-0.5">
      {Array.from({ length: 5 }, (_, i) => (
        <span
          key={i}
          className={cn(
            "material-symbols-outlined text-[14px]",
            i < conviction ? "text-primary" : "text-outline-variant"
          )}
        >
          star
        </span>
      ))}
    </div>
  );
}

export default function ForwardSignalTable({ signals }: Props) {
  return (
    <div className="space-y-1">
      {/* Header */}
      <div className="grid grid-cols-4 gap-4 px-3 py-2 text-[0.625rem] uppercase tracking-widest text-on-surface-variant/60">
        <span>Commodity</span>
        <span>Direction</span>
        <span>Conviction</span>
        <span>Horizon</span>
      </div>
      {/* Rows */}
      {signals.map((signal) => (
        <div
          key={signal.commodity}
          className="grid grid-cols-4 gap-4 px-3 py-2.5 rounded-lg hover:bg-surface-container-high transition-colors"
        >
          <span className="text-sm text-on-surface font-medium">
            {signal.commodity}
          </span>
          <DirectionArrow direction={signal.direction} />
          <ConvictionStars conviction={signal.conviction} />
          <span className="text-xs text-on-surface-variant bg-surface-container-high px-2 py-0.5 rounded w-fit">
            {signal.horizon}
          </span>
        </div>
      ))}
    </div>
  );
}
