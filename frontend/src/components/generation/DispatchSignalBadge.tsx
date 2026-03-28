"use client";

const SIGNAL_CONFIG: Record<string, { label: string; color: string; icon: string }> = {
  GAS_MARGINAL: {
    label: "Gas Marginal",
    color: "bg-primary/10 text-primary",
    icon: "local_fire_department",
  },
  COAL_MARGINAL: {
    label: "Coal Marginal",
    color: "bg-tertiary/10 text-tertiary",
    icon: "factory",
  },
  NEGATIVE_SPREADS: {
    label: "Negative Spreads",
    color: "bg-error/10 text-error",
    icon: "warning",
  },
};

interface DispatchSignalBadgeProps {
  signal: string;
  daPrice: number;
  eurUsd: number;
}

export default function DispatchSignalBadge({
  signal,
  daPrice,
  eurUsd,
}: DispatchSignalBadgeProps) {
  const config = SIGNAL_CONFIG[signal] ?? SIGNAL_CONFIG.NEGATIVE_SPREADS;

  return (
    <div className="bg-surface-container p-6 rounded-xl">
      <div className="flex items-center justify-between flex-wrap gap-4">
        <div className="flex items-center gap-4">
          <div
            className={`flex items-center gap-2 px-4 py-2 rounded-lg ${config.color}`}
          >
            <span className="material-symbols-outlined text-xl">
              {config.icon}
            </span>
            <div>
              <p className="text-[0.625rem] uppercase tracking-widest opacity-60">
                Dispatch Signal
              </p>
              <p className="font-headline font-bold">{config.label}</p>
            </div>
          </div>
        </div>
        <div className="flex items-center gap-6">
          <div className="text-right">
            <p className="text-[0.625rem] uppercase tracking-widest text-on-surface-variant">
              DA Price (RDN)
            </p>
            <p className="font-headline text-lg font-bold text-on-surface">
              €{daPrice.toFixed(2)}
              <span className="text-sm text-on-surface-variant font-normal">
                {" "}
                /MWh
              </span>
            </p>
          </div>
          <div className="text-right">
            <p className="text-[0.625rem] uppercase tracking-widest text-on-surface-variant">
              EUR/USD
            </p>
            <p className="font-headline text-lg font-bold text-on-surface">
              {eurUsd.toFixed(4)}
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
