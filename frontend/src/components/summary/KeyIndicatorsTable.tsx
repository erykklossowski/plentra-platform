"use client";

import { useState } from "react";
import { cn } from "@/lib/utils";
import type { KeyIndicator } from "@/types/api";

interface Props {
  indicators: KeyIndicator[];
}

const commodityIcons: Record<string, string> = {
  ttf: "TTF",
  tge: "TGE",
  ara: "ARA",
  eua: "CO₂",
};

export default function KeyIndicatorsTable({ indicators }: Props) {
  const [view, setView] = useState<"daily" | "monthly">("daily");

  return (
    <div className="bg-surface-container p-6 rounded-xl">
      <div className="flex items-center justify-between mb-4">
        <h2 className="font-headline text-lg font-bold text-on-surface">
          Key Market Indicators
        </h2>
        <div className="flex gap-1 bg-surface-container-lowest rounded-lg p-1">
          <button
            onClick={() => setView("daily")}
            className={cn(
              "px-3 py-1 text-xs rounded-md transition-colors",
              view === "daily"
                ? "bg-surface-container-high text-on-surface"
                : "text-on-surface-variant hover:text-on-surface"
            )}
          >
            DAILY
          </button>
          <button
            onClick={() => setView("monthly")}
            className={cn(
              "px-3 py-1 text-xs rounded-md transition-colors",
              view === "monthly"
                ? "bg-surface-container-high text-on-surface"
                : "text-on-surface-variant hover:text-on-surface"
            )}
          >
            MONTHLY
          </button>
        </div>
      </div>

      {/* Table Header */}
      <div className="grid grid-cols-6 gap-4 px-3 py-2 text-[0.625rem] uppercase tracking-widest text-on-surface-variant/60">
        <span>Benchmark Fuel</span>
        <span>Spot Price</span>
        <span>Forward M+1</span>
        <span>MoM Δ</span>
        <span className="col-span-2">Spread / Spread Δ</span>
      </div>

      {/* Table Rows */}
      <div className="space-y-1 mt-1">
        {indicators.map((indicator) => (
          <div
            key={indicator.id}
            className="grid grid-cols-6 gap-4 px-3 py-3 rounded-lg hover:bg-surface-container-high transition-colors items-center"
          >
            {/* Commodity */}
            <div className="flex items-center gap-2">
              <span className="text-[0.5625rem] font-bold bg-surface-container-high text-on-surface-variant px-1.5 py-0.5 rounded">
                {commodityIcons[indicator.id] ?? indicator.id.toUpperCase()}
              </span>
              <span className="text-sm text-on-surface">{indicator.label}</span>
            </div>

            {/* Spot Price */}
            <span className="font-headline font-bold text-on-surface">
              {indicator.spot.toFixed(2)}
              <span className="text-xs text-on-surface-variant ml-1">
                {indicator.unit}
              </span>
            </span>

            {/* Forward M+1 */}
            <span className="text-sm text-on-surface">
              {indicator.forward_m1.toFixed(2)}
            </span>

            {/* MoM Delta */}
            <span
              className={cn(
                "text-sm font-medium",
                indicator.mom_delta_pct >= 0
                  ? "text-emerald-400"
                  : "text-error"
              )}
            >
              {indicator.mom_delta_pct >= 0 ? "+" : ""}
              {indicator.mom_delta_pct.toFixed(1)}%
            </span>

            {/* Spread */}
            <div className="col-span-2 flex items-center gap-2">
              <span
                className={cn(
                  "inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium",
                  indicator.spread_direction === "UP"
                    ? "bg-emerald-500/10 text-emerald-400"
                    : "bg-error/10 text-error"
                )}
              >
                {indicator.spread_label}
                <span className="font-bold">
                  {indicator.spread_value.toFixed(2)}
                </span>
                <span className="material-symbols-outlined text-[12px]">
                  {indicator.spread_direction === "UP"
                    ? "arrow_upward"
                    : "arrow_downward"}
                </span>
              </span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
