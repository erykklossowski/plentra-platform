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

// Map our bidding zone codes to ISO alpha-3 country codes used in TopoJSON
const ZONE_TO_ISO: Record<string, string[]> = {
  PL: ["POL"],
  "DE-LU": ["DEU", "LUX"],
  FR: ["FRA"],
  ES: ["ESP"],
  NL: ["NLD"],
  BE: ["BEL"],
  AT: ["AUT"],
  CZ: ["CZE"],
  SK: ["SVK"],
  SE4: ["SWE"],
  DK1: ["DNK"],
  "IT-N": ["ITA"],
  HU: ["HUN"],
  RO: ["ROU"],
};

// Build reverse lookup: ISO -> zone data
function buildIsoLookup(
  rankings: EURankingEntry[]
): Map<string, EURankingEntry> {
  const map = new Map<string, EURankingEntry>();
  for (const entry of rankings) {
    const isos = ZONE_TO_ISO[entry.country_code];
    if (isos) {
      for (const iso of isos) {
        map.set(iso, entry);
      }
    }
  }
  return map;
}

function priceToColor(price: number, min: number, max: number): string {
  const range = max - min || 1;
  const t = Math.max(0, Math.min(1, (price - min) / range));

  // Gradient: emerald (#34d399) → primary (#76d6d5) → tertiary (#ffb692) → error (#ffb4ab)
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

  const isoLookup = useMemo(() => buildIsoLookup(data), [data]);
  const prices = data.map((d) => d.da_price_eur_mwh);
  const minPrice = Math.min(...prices);
  const maxPrice = Math.max(...prices);

  // European countries to show (even without data)
  const EUROPEAN_ISOS = new Set([
    "POL", "DEU", "FRA", "ESP", "ITA", "GBR", "NLD", "BEL", "AUT",
    "CZE", "SVK", "HUN", "ROU", "BGR", "HRV", "SVN", "SRB", "BIH",
    "MNE", "ALB", "MKD", "GRC", "PRT", "CHE", "SWE", "NOR", "FIN",
    "DNK", "EST", "LVA", "LTU", "IRL", "LUX",
  ]);

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
                .filter((geo) => {
                  const iso = geo.properties?.ISO_A3 ?? geo.id;
                  return EUROPEAN_ISOS.has(iso);
                })
                .map((geo) => {
                  const iso = geo.properties?.ISO_A3 ?? geo.id;
                  const entry = isoLookup.get(iso);

                  const fill = entry
                    ? priceToColor(entry.da_price_eur_mwh, minPrice, maxPrice)
                    : "#222a3d";

                  const stroke = entry?.is_focus ? "#76d6d5" : "#3e4949";
                  const strokeWidth = entry?.is_focus ? 2 : 0.5;

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
