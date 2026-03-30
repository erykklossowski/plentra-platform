"use client";

import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
  Brush,
} from "recharts";
import type { EveningDecompositionPoint } from "@/types/api";

interface EveningDecompositionChartProps {
  data: EveningDecompositionPoint[];
}

export default function EveningDecompositionChart({
  data,
}: EveningDecompositionChartProps) {
  if (data.length === 0) {
    return (
      <div className="h-48 flex items-center justify-center">
        <p className="text-sm text-on-surface-variant">No decomposition data available</p>
      </div>
    );
  }

  return (
    <div>
      <ResponsiveContainer width="100%" height={300}>
        <AreaChart data={data} margin={{ top: 4, right: 8, bottom: 4, left: 8 }}>
          <XAxis
            dataKey="date"
            tick={{ fontSize: 10, fill: "var(--color-on-surface-variant)" }}
            tickFormatter={(d) =>
              new Date(d).toLocaleDateString("pl-PL", { day: "numeric", month: "short" })
            }
            tickLine={false}
            axisLine={false}
          />
          <YAxis
            tick={{ fontSize: 10, fill: "var(--color-on-surface-variant)" }}
            tickLine={false}
            axisLine={false}
            label={{
              value: "PLN/MWh",
              angle: -90,
              position: "insideLeft",
              style: { fontSize: 9, fill: "var(--color-on-surface-variant)" },
            }}
          />
          <Tooltip
            contentStyle={{
              background: "rgba(49,57,77,0.92)",
              backdropFilter: "blur(16px)",
              border: "none",
              borderRadius: "0.5rem",
              fontSize: "0.75rem",
            }}
            labelFormatter={(d) =>
              new Date(d as string).toLocaleDateString("pl-PL", {
                weekday: "short",
                day: "numeric",
                month: "short",
                year: "numeric",
              })
            }
            formatter={(value, name) => [
              `${Number(value).toFixed(1)} PLN/MWh`,
              String(name),
            ]}
          />
          <Area
            type="monotone"
            dataKey="baseline_pln"
            name="Baseline"
            stackId="a"
            stroke="#4a9e9d"
            fill="#4a9e9d"
            fillOpacity={0.6}
          />
          <Area
            type="monotone"
            dataKey="delta_fuel_pln"
            name="Paliwa (CSS)"
            stackId="a"
            stroke="#f59e0b"
            fill="#f59e0b"
            fillOpacity={0.5}
          />
          <Area
            type="monotone"
            dataKey="delta_oze_pln"
            name="OZE"
            stackId="a"
            stroke="#22c55e"
            fill="#22c55e"
            fillOpacity={0.4}
          />
          <Area
            type="monotone"
            dataKey="residual_pln"
            name="Residual"
            stackId="a"
            stroke="#94a3b8"
            fill="#94a3b8"
            fillOpacity={0.3}
          />
          <Brush
            dataKey="date"
            height={20}
            stroke="var(--color-outline-variant)"
            fill="var(--color-surface-container-low)"
            travellerWidth={6}
            tickFormatter={(d) =>
              new Date(d).toLocaleDateString("pl-PL", { day: "numeric", month: "short" })
            }
          />
        </AreaChart>
      </ResponsiveContainer>

      {/* Legend */}
      <div className="mt-4 grid grid-cols-2 md:grid-cols-4 gap-3 text-xs">
        <LegendItem color="#4a9e9d" label="Baseline" desc="7-dniowa srednia krocząca ceny 17-21h" />
        <LegendItem color="#f59e0b" label="Paliwa (CSS)" desc="wpływ cen gazu/wegla (pass-through 65%)" />
        <LegendItem color="#22c55e" label="OZE" desc="premia za wypychanie konwencji przez OZE" />
        <LegendItem color="#94a3b8" label="Residual" desc="import, awarie, demand response" />
      </div>
    </div>
  );
}

function LegendItem({ color, label, desc }: { color: string; label: string; desc: string }) {
  return (
    <div className="flex items-start gap-2">
      <div className="w-3 h-3 rounded-sm mt-0.5 shrink-0" style={{ backgroundColor: color }} />
      <div>
        <p className="font-medium text-on-surface">{label}</p>
        <p className="text-on-surface-variant text-[10px]">{desc}</p>
      </div>
    </div>
  );
}
