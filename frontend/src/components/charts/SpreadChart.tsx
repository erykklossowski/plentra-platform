"use client";

import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ReferenceLine,
  ResponsiveContainer,
} from "recharts";
import type { SpreadHistoryEntry } from "@/types/api";

interface SpreadChartProps {
  data: SpreadHistoryEntry[];
}

function CustomTooltip({
  active,
  payload,
  label,
}: {
  active?: boolean;
  payload?: Array<{ name: string; value: number; color: string }>;
  label?: string;
}) {
  if (!active || !payload) return null;

  return (
    <div
      className="rounded-lg px-3 py-2 text-xs text-on-surface"
      style={{
        background: "rgba(49, 57, 77, 0.85)",
        backdropFilter: "blur(16px)",
      }}
    >
      <p className="text-on-surface-variant mb-1">{label}</p>
      {payload.map((entry) => (
        <p key={entry.name} style={{ color: entry.color }}>
          {entry.name}: {entry.value.toFixed(2)} EUR/MWh
        </p>
      ))}
    </div>
  );
}

export default function SpreadChart({ data }: SpreadChartProps) {
  return (
    <div className="bg-surface-container p-4 rounded-xl">
      <ResponsiveContainer width="100%" height={250}>
        <LineChart data={data}>
          <CartesianGrid stroke="#3e4949" strokeDasharray="4 4" opacity={0.3} />
          <XAxis
            dataKey="date"
            tick={{ fill: "#bdc9c8", fontSize: 10 }}
            axisLine={false}
            tickLine={false}
          />
          <YAxis
            tick={{ fill: "#bdc9c8", fontSize: 10 }}
            axisLine={false}
            tickLine={false}
          />
          <Tooltip content={<CustomTooltip />} />
          <ReferenceLine
            y={0}
            stroke="#3e4949"
            strokeDasharray="4 4"
            strokeWidth={1}
          />
          <Line
            type="monotone"
            dataKey="css"
            stroke="#76d6d5"
            strokeWidth={2}
            dot={false}
            name="CSS"
          />
          <Line
            type="monotone"
            dataKey="cds_42"
            stroke="#ffb692"
            strokeWidth={2}
            dot={false}
            name="CDS (η42)"
          />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}
