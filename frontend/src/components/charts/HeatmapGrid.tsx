"use client";

import { useState } from "react";
import type { HeatmapEntry } from "@/types/api";

interface HeatmapGridProps {
  data: HeatmapEntry[];
}

const MONTHS = ["JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC"];
const HOUR_LABELS = [0, 3, 6, 9, 12, 15, 18, 21];

function interpolateColor(t: number): string {
  // 4-stop gradient: surface-container-low → primary → tertiary → error
  const stops = [
    { r: 0x13, g: 0x1b, b: 0x2e }, // surface-container-low
    { r: 0x76, g: 0xd6, b: 0xd5 }, // primary
    { r: 0xff, g: 0xb6, b: 0x92 }, // tertiary
    { r: 0xff, g: 0xb4, b: 0xab }, // error
  ];

  const idx = Math.min(t * 3, 2.999);
  const i = Math.floor(idx);
  const frac = idx - i;

  const a = stops[i];
  const b = stops[i + 1];

  const r = Math.round(a.r + (b.r - a.r) * frac);
  const g = Math.round(a.g + (b.g - a.g) * frac);
  const bl = Math.round(a.b + (b.b - a.b) * frac);

  return `rgb(${r}, ${g}, ${bl})`;
}

export default function HeatmapGrid({ data }: HeatmapGridProps) {
  const [tooltip, setTooltip] = useState<{
    month: string;
    hour: number;
    value: number;
    x: number;
    y: number;
  } | null>(null);

  // Build lookup and find min/max
  const lookup = new Map<string, number>();
  let minVal = Infinity;
  let maxVal = -Infinity;

  for (const entry of data) {
    const key = `${entry.month}-${entry.hour}`;
    lookup.set(key, entry.value);
    minVal = Math.min(minVal, entry.value);
    maxVal = Math.max(maxVal, entry.value);
  }

  const range = maxVal - minVal || 1;

  return (
    <div className="relative">
      {/* Hour labels */}
      <div className="flex ml-10 mb-1">
        {Array.from({ length: 24 }, (_, h) => (
          <div key={h} className="flex-1 text-center">
            {HOUR_LABELS.includes(h) && (
              <span className="text-[0.5625rem] text-on-surface-variant">
                {h.toString().padStart(2, "0")}
              </span>
            )}
          </div>
        ))}
      </div>

      {/* Grid rows */}
      <div className="space-y-[2px]">
        {MONTHS.map((month) => (
          <div key={month} className="flex items-center gap-1">
            <span className="w-9 text-[0.5625rem] text-on-surface-variant text-right shrink-0">
              {month}
            </span>
            <div className="heatmap-grid flex-1">
              {Array.from({ length: 24 }, (_, h) => {
                const key = `${month}-${h}`;
                const value = lookup.get(key) ?? 0;
                const t = (value - minVal) / range;
                return (
                  <div
                    key={h}
                    className="aspect-square rounded-sm cursor-pointer transition-opacity hover:opacity-80"
                    style={{ backgroundColor: interpolateColor(t) }}
                    onMouseEnter={(e) => {
                      const rect = e.currentTarget.getBoundingClientRect();
                      setTooltip({
                        month,
                        hour: h,
                        value,
                        x: rect.left + rect.width / 2,
                        y: rect.top - 10,
                      });
                    }}
                    onMouseLeave={() => setTooltip(null)}
                  />
                );
              })}
            </div>
          </div>
        ))}
      </div>

      {/* Tooltip */}
      {tooltip && (
        <div
          className="fixed z-50 px-3 py-2 rounded-lg text-xs text-on-surface pointer-events-none"
          style={{
            left: tooltip.x,
            top: tooltip.y,
            transform: "translate(-50%, -100%)",
            background: "rgba(49, 57, 77, 0.85)",
            backdropFilter: "blur(16px)",
          }}
        >
          <span className="text-on-surface-variant">{tooltip.month}</span>
          <span className="mx-1 text-outline-variant">|</span>
          <span className="text-on-surface-variant">
            {tooltip.hour.toString().padStart(2, "0")}:00
          </span>
          <span className="mx-1 text-outline-variant">|</span>
          <span className="font-semibold">{tooltip.value.toFixed(1)} GW</span>
        </div>
      )}
    </div>
  );
}
