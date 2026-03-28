"use client";

import { useState, useMemo } from "react";
import {
  ComposableMap,
  Geographies,
  Geography,
  ZoomableGroup,
} from "react-simple-maps";
import type { EURankingEntry } from "@/types/api";

const GEO_URL = "https://cdn.jsdelivr.net/npm/world-atlas@2/countries-50m.json";

// world-atlas uses ISO 3166-1 numeric codes as geo.id
// Map our bidding zone codes to those numeric IDs
const ZONE_TO_NUMERIC: Record<string, string[]> = {
  PL: ["616"],
  "DE-LU": ["276", "442"],
  FR: ["250"],
  ES: ["724"],
  NL: ["528"],
  BE: ["056"],
  AT: ["040"],
  CZ: ["203"],
  SK: ["703"],
  SE4: ["752"],
  DK1: ["208"],
  "IT-N": ["380"],
  HU: ["348"],
  RO: ["642"],
};

// European country numeric IDs to render (even without price data)
const EUROPEAN_IDS = new Set([
  "616", "276", "250", "724", "380", "826", "528", "056", "040",
  "203", "703", "348", "642", "100", "191", "705", "688", "070",
  "499", "008", "807", "300", "620", "756", "752", "578", "246",
  "208", "233", "428", "440", "372", "442",
]);

function buildNumericLookup(
  rankings: EURankingEntry[]
): Map<string, EURankingEntry> {
  const map = new Map<string, EURankingEntry>();
  for (const entry of rankings) {
    const ids = ZONE_TO_NUMERIC[entry.country_code];
    if (ids) {
      for (const id of ids) {
        map.set(id, entry);
      }
    }
  }
  return map;
}

function priceToColor(price: number, min: number, max: number): string {
  const range = max - min || 1;
  const t = Math.max(0, Math.min(1, (price - min) / range));

  if (t < 0.33) {
    const s = t / 0.33;
    return lerpColor([0x34, 0xd3, 0x99], [0x76, 0xd6, 0xd5], s);
  } else if (t < 0.66) {
    const s = (t - 0.33) / 0.33;
    return lerpColor([0x76, 0xd6, 0xd5], [0xff, 0xb6, 0x92], s);
  } else {
    const s = (t - 0.66) / 0.34;
    return lerpColor([0xff, 0xb6, 0x92], [0xff, 0xb4, 0xab], s);
  }
}

function lerpColor(a: number[], b: number[], t: number): string {
  const r = Math.round(a[0] + (b[0] - a[0]) * t);
  const g = Math.round(a[1] + (b[1] - a[1]) * t);
  const bl = Math.round(a[2] + (b[2] - a[2]) * t);
  return `rgb(${r}, ${g}, ${bl})`;
}

interface EuropeMapProps {
  data: EURankingEntry[];
}

export default function EuropeMap({ data }: EuropeMapProps) {
  const [tooltip, setTooltip] = useState<{
    entry: EURankingEntry;
    x: number;
    y: number;
  } | null>(null);

  const numericLookup = useMemo(() => buildNumericLookup(data), [data]);
  const prices = data.map((d) => d.da_price_eur_mwh);
  const minPrice = Math.min(...prices);
  const maxPrice = Math.max(...prices);

  return (
    <div className="relative">
      <ComposableMap
        projection="geoAzimuthalEqualArea"
        projectionConfig={{
          rotate: [-10, -52, 0],
          scale: 700,
        }}
        width={800}
        height={500}
        style={{ width: "100%", height: "auto" }}
      >
        <ZoomableGroup center={[10, 52]} zoom={1}>
          <Geographies geography={GEO_URL}>
            {({ geographies }) =>
              geographies
                .filter((geo) => EUROPEAN_IDS.has(geo.id))
                .map((geo) => {
                  const entry = numericLookup.get(geo.id);

                  const fill = entry
                    ? priceToColor(entry.da_price_eur_mwh, minPrice, maxPrice)
                    : "#222a3d";

                  const stroke = entry?.is_focus ? "#76d6d5" : "#3e4949";
                  const strokeWidth = entry?.is_focus ? 2.5 : 0.5;

                  return (
                    <Geography
                      key={geo.rsmKey}
                      geography={geo}
                      fill={fill}
                      stroke={stroke}
                      strokeWidth={strokeWidth}
                      style={{
                        default: { outline: "none" },
                        hover: {
                          outline: "none",
                          fill: entry ? fill : "#2d3449",
                          strokeWidth: 1.5,
                          stroke: "#76d6d5",
                        },
                        pressed: { outline: "none" },
                      }}
                      onMouseEnter={(evt) => {
                        if (entry) {
                          const rect = (
                            evt.target as SVGElement
                          ).closest("svg")?.getBoundingClientRect();
                          setTooltip({
                            entry,
                            x: evt.clientX - (rect?.left ?? 0),
                            y: evt.clientY - (rect?.top ?? 0),
                          });
                        }
                      }}
                      onMouseLeave={() => setTooltip(null)}
                    />
                  );
                })
            }
          </Geographies>
        </ZoomableGroup>
      </ComposableMap>

      {/* Tooltip */}
      {tooltip && (
        <div
          className="absolute pointer-events-none rounded-lg px-3 py-2 text-xs text-on-surface z-10"
          style={{
            left: tooltip.x + 12,
            top: tooltip.y - 40,
            background: "rgba(49, 57, 77, 0.92)",
            backdropFilter: "blur(16px)",
          }}
        >
          <p className="font-bold text-sm">
            {tooltip.entry.is_focus && (
              <span className="text-[9px] bg-primary text-on-primary px-1 rounded mr-1">
                FOCUS
              </span>
            )}
            {tooltip.entry.country_name}
          </p>
          <p className="text-on-surface-variant mt-0.5">
            #{tooltip.entry.rank} — €
            {tooltip.entry.da_price_eur_mwh.toFixed(2)}/MWh
          </p>
        </div>
      )}

      {/* Color Legend */}
      <div className="flex items-center justify-center gap-2 mt-4 text-xs text-on-surface-variant">
        <span>€{minPrice.toFixed(0)}</span>
        <div
          className="h-3 w-40 rounded"
          style={{
            background: `linear-gradient(to right,
              rgb(52, 211, 153),
              rgb(118, 214, 213),
              rgb(255, 182, 146),
              rgb(255, 180, 171))`,
          }}
        />
        <span>€{maxPrice.toFixed(0)}</span>
        <span className="ml-4 text-on-surface-variant/60">EUR/MWh</span>
      </div>
    </div>
  );
}
