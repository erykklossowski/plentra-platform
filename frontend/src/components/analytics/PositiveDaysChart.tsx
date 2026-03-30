"use client";

import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  Tooltip,
  ReferenceLine,
  ResponsiveContainer,
  Cell,
} from "recharts";
import type { PositiveDaysPoint } from "@/types/api";

interface PositiveDaysChartProps {
  data: PositiveDaysPoint[];
}

export default function PositiveDaysChart({ data }: PositiveDaysChartProps) {
  const cssData = data
    .filter((d) => d.spread_type === "rolling_3m_css")
    .map((d) => ({
      month: new Date(d.month).toLocaleDateString("pl-PL", { month: "short" }),
      positive_pct: d.positive_pct,
      positive_days: d.positive_days,
      total_days: d.total_days,
    }));

  if (cssData.length === 0) {
    return (
      <div className="h-48 flex items-center justify-center">
        <p className="text-sm text-on-surface-variant">No positive days data available</p>
      </div>
    );
  }

  return (
    <ResponsiveContainer width="100%" height={200}>
      <BarChart data={cssData} margin={{ top: 8, right: 8, bottom: 4, left: 8 }}>
        <XAxis
          dataKey="month"
          tick={{ fontSize: 10, fill: "var(--color-on-surface-variant)" }}
          tickLine={false}
          axisLine={false}
        />
        <YAxis
          domain={[0, 100]}
          tick={{ fontSize: 10, fill: "var(--color-on-surface-variant)" }}
          tickLine={false}
          axisLine={false}
          label={{
            value: "%",
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
          formatter={(value, _, item) => {
            const d = item.payload;
            return [`${d.positive_days}/${d.total_days} dni (${value}%)`, "CSS > 0"];
          }}
        />
        <ReferenceLine
          y={50}
          stroke="var(--color-outline-variant)"
          strokeDasharray="4 4"
          label={{
            value: "50%",
            position: "right",
            style: { fontSize: 9, fill: "var(--color-on-surface-variant)" },
          }}
        />
        <Bar dataKey="positive_pct" name="% CSS > 0" radius={[4, 4, 0, 0]}>
          {cssData.map((entry, i) => (
            <Cell
              key={i}
              fill={entry.positive_pct >= 50 ? "rgba(118,214,213,0.7)" : "rgba(255,180,171,0.5)"}
            />
          ))}
        </Bar>
      </BarChart>
    </ResponsiveContainer>
  );
}
