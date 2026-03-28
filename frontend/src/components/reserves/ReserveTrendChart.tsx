"use client";

import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from "recharts";
import type { ReserveMonthlyHistory } from "@/types/api";

interface ReserveTrendChartProps {
  history: ReserveMonthlyHistory[];
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
          {entry.name}: {entry.value.toFixed(1)} PLN/MW
        </p>
      ))}
    </div>
  );
}

export default function ReserveTrendChart({
  history,
}: ReserveTrendChartProps) {
  return (
    <ResponsiveContainer width="100%" height={300}>
      <LineChart data={history}>
        <CartesianGrid
          stroke="#3e4949"
          strokeDasharray="4 4"
          opacity={0.3}
        />
        <XAxis
          dataKey="month"
          tick={{ fill: "#bdc9c8", fontSize: 10 }}
          axisLine={false}
          tickLine={false}
        />
        <YAxis
          tick={{ fill: "#bdc9c8", fontSize: 10 }}
          axisLine={false}
          tickLine={false}
          label={{
            value: "PLN/MW",
            angle: -90,
            position: "insideLeft",
            fill: "#bdc9c8",
            fontSize: 10,
          }}
        />
        <Tooltip content={<CustomTooltip />} />
        <Legend
          wrapperStyle={{
            fontSize: 11,
            color: "#bdc9c8",
          }}
        />
        <Line
          type="monotone"
          dataKey="afrr_g"
          stroke="#76d6d5"
          strokeWidth={2}
          dot={false}
          name="aFRR ↑"
        />
        <Line
          type="monotone"
          dataKey="mfrrd_g"
          stroke="#ffb692"
          strokeWidth={2}
          dot={false}
          name="mFRRd ↑"
        />
        <Line
          type="monotone"
          dataKey="fcr_g"
          stroke="#bdc9c8"
          strokeWidth={1.5}
          dot={false}
          name="FCR ↑"
          strokeDasharray="4 4"
        />
      </LineChart>
    </ResponsiveContainer>
  );
}
