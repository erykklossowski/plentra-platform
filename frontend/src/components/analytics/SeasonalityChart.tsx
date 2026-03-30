"use client";

import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
  ReferenceLine,
  ErrorBar,
  Cell,
} from "recharts";
import type { SeasonalityPoint } from "@/types/api";

interface SeasonalityChartProps {
  data: SeasonalityPoint[];
}

export default function SeasonalityChart({ data }: SeasonalityChartProps) {
  // Filter CSS only and format for chart
  const cssData = data
    .filter((d) => d.spread_type === "rolling_3m_css")
    .map((d) => {
      const median = d.median ?? 0;
      const q1 = d.q1 ?? 0;
      const q3 = d.q3 ?? 0;
      const min = d.min ?? 0;
      const max = d.max ?? 0;
      return {
        month: new Date(d.month).toLocaleDateString("pl-PL", { month: "short" }),
        median,
        q1,
        q3,
        min,
        max,
        // Error bar: distance from median to extremes
        errorUp: max - median,
        errorDown: median - min,
        // Bar height: Q3-Q1 range, offset from Q1
        barBase: q1,
        barHeight: q3 - q1,
        isPositive: median > 0,
        n_days: d.n_days,
      };
    });

  if (cssData.length === 0) {
    return (
      <div className="h-48 flex items-center justify-center">
        <p className="text-sm text-on-surface-variant">No seasonality data available</p>
      </div>
    );
  }

  return (
    <ResponsiveContainer width="100%" height={240}>
      <BarChart data={cssData} margin={{ top: 8, right: 8, bottom: 4, left: 8 }}>
        <XAxis
          dataKey="month"
          tick={{ fontSize: 10, fill: "var(--color-on-surface-variant)" }}
          tickLine={false}
          axisLine={false}
        />
        <YAxis
          tick={{ fontSize: 10, fill: "var(--color-on-surface-variant)" }}
          tickLine={false}
          axisLine={false}
          label={{
            value: "EUR/MWh",
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
          formatter={(_, name, item) => {
            const d = item.payload;
            return [
              `Median: ${d.median?.toFixed(2)} | Q1-Q3: ${d.q1?.toFixed(2)}–${d.q3?.toFixed(2)} | Range: ${d.min?.toFixed(2)}–${d.max?.toFixed(2)} (${d.n_days}d)`,
              "CSS",
            ];
          }}
        />
        <ReferenceLine y={0} stroke="var(--color-outline-variant)" strokeDasharray="4 4" />
        <Bar dataKey="median" name="Median CSS" radius={[4, 4, 0, 0]}>
          {cssData.map((entry, i) => (
            <Cell
              key={i}
              fill={entry.isPositive ? "rgba(118,214,213,0.6)" : "rgba(255,180,171,0.6)"}
              stroke={entry.isPositive ? "#76d6d5" : "#ffb4ab"}
              strokeWidth={1}
            />
          ))}
          <ErrorBar
            dataKey="errorUp"
            direction="y"
            width={4}
            stroke="var(--color-on-surface-variant)"
            strokeWidth={1}
          />
        </Bar>
      </BarChart>
    </ResponsiveContainer>
  );
}
