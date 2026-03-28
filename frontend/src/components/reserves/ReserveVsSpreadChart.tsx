"use client";

import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from "recharts";
import type { SpreadHistoryEntry, ReserveMonthlyHistory } from "@/types/api";

interface ReserveVsSpreadChartProps {
  reserveHistory: ReserveMonthlyHistory[];
  spreadHistory: SpreadHistoryEntry[];
  currentAfrrG: number;
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
          {entry.name}: {entry.value.toFixed(2)}{" "}
          {entry.name.includes("aFRR") ? "PLN/MW" : "EUR/MWh"}
        </p>
      ))}
    </div>
  );
}

export default function ReserveVsSpreadChart({
  spreadHistory,
  currentAfrrG,
}: ReserveVsSpreadChartProps) {
  // Build combined dataset from spread history with constant aFRR_G line
  const combined = spreadHistory.map((s) => ({
    date: s.date,
    css: s.css,
    afrr_g: currentAfrrG,
  }));

  return (
    <ResponsiveContainer width="100%" height={280}>
      <LineChart data={combined}>
        <CartesianGrid
          stroke="#3e4949"
          strokeDasharray="4 4"
          opacity={0.3}
        />
        <XAxis
          dataKey="date"
          tick={{ fill: "#bdc9c8", fontSize: 10 }}
          axisLine={false}
          tickLine={false}
        />
        <YAxis
          yAxisId="left"
          tick={{ fill: "#76d6d5", fontSize: 10 }}
          axisLine={false}
          tickLine={false}
          label={{
            value: "PLN/MW",
            angle: -90,
            position: "insideLeft",
            fill: "#76d6d5",
            fontSize: 10,
          }}
        />
        <YAxis
          yAxisId="right"
          orientation="right"
          tick={{ fill: "#ffb692", fontSize: 10 }}
          axisLine={false}
          tickLine={false}
          label={{
            value: "EUR/MWh",
            angle: 90,
            position: "insideRight",
            fill: "#ffb692",
            fontSize: 10,
          }}
        />
        <Tooltip content={<CustomTooltip />} />
        <Line
          yAxisId="left"
          type="monotone"
          dataKey="afrr_g"
          stroke="#76d6d5"
          strokeWidth={2}
          dot={false}
          name="aFRR ↑ (PLN/MW)"
          strokeDasharray="6 3"
        />
        <Line
          yAxisId="right"
          type="monotone"
          dataKey="css"
          stroke="#ffb692"
          strokeWidth={2}
          dot={false}
          name="CSS (EUR/MWh)"
        />
      </LineChart>
    </ResponsiveContainer>
  );
}
