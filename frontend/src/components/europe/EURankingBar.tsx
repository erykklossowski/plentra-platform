"use client";

import type { EURankingEntry } from "@/types/api";

interface EURankingBarProps {
  data: EURankingEntry[];
}

export default function EURankingBar({ data }: EURankingBarProps) {
  return (
    <div className="space-y-3">
      {data.map((entry) => (
        <div
          key={entry.country_code}
          className={
            entry.is_focus
              ? "space-y-1.5 ring-2 ring-primary ring-offset-4 ring-offset-surface-container p-2 rounded"
              : "space-y-1.5 p-2"
          }
        >
          <div className="flex justify-between text-xs font-black text-on-surface">
            <span className="flex items-center gap-2">
              {entry.rank}. {entry.country_name} ({entry.country_code})
              {entry.is_focus && (
                <span className="text-[9px] bg-primary text-on-primary px-1 rounded">
                  FOCUS
                </span>
              )}
            </span>
            <span>€{entry.da_price_eur_mwh.toFixed(2)}</span>
          </div>
          <div className="w-full h-8 bg-surface-container-low rounded flex items-center px-1">
            <div
              className={`h-6 rounded-sm transition-all duration-500 ${
                entry.is_focus
                  ? "bg-primary shadow-[0_0_12px_rgba(118,214,213,0.3)]"
                  : "bg-surface-container-highest"
              }`}
              style={{ width: `${entry.bar_pct}%` }}
            />
          </div>
        </div>
      ))}
    </div>
  );
}
