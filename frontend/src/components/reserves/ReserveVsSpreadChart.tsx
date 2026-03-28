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
import type { SpreadHistoryEntry, ReserveDailyHistory } from "@/types/api";

interface ReserveVsSpreadChartProps {
  reserveDaily: ReserveDailyHistory[];
  spreadHistory: SpreadHistoryEntry[];
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
  reserveDaily,
  spreadHistory,
}: ReserveVsSpreadChartProps) {
  // Join by index — both are 30-day series, most recent last
  // Align from the end (both end at "today")
  const rLen = reserveDaily.length;
  const sLen = spreadHistory.length;
  const len = Math.max(rLen, sLen);
  const combined = Array.from({ length: len }, (_, i) => {
    const sIdx = sLen - len + i;
    const rIdx = rLen - len + i;
    const spread = sIdx >= 0 ? spreadHistory[sIdx] : undefined;
    const reserve = rIdx >= 0 ? reserveDaily[rIdx] : undefined;
    return {
      date: reserve?.date ?? spread?.date ?? `${i + 1}`,
      css: spread?.css ?? null,
      afrr_g: reserve?.afrr_g ?? null,
    };
  });

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
          connectNulls
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
