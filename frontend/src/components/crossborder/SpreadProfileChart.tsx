"use client";

import {
  AreaChart,
  Area,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ReferenceLine,
  ResponsiveContainer,
} from "recharts";
import type { CrossBorderHourly } from "@/types/api";

interface SpreadProfileChartProps {
  data: CrossBorderHourly[];
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
      <p className="text-on-surface-variant mb-1">Hour {label}</p>
      {payload.map((entry) => (
        <p key={entry.name} style={{ color: entry.color }}>
          {entry.name}: €{entry.value.toFixed(2)}/MWh
        </p>
      ))}
    </div>
  );
}

export default function SpreadProfileChart({
  data,
}: SpreadProfileChartProps) {
  return (
    <div className="bg-surface-container p-4 rounded-xl">
      <ResponsiveContainer width="100%" height={300}>
        <AreaChart data={data}>
          <CartesianGrid
            stroke="#3e4949"
            strokeDasharray="4 4"
            opacity={0.3}
          />
          <XAxis
            dataKey="hour"
            tick={{ fill: "#bdc9c8", fontSize: 10 }}
            axisLine={false}
            tickLine={false}
            tickFormatter={(h: number) => `${h}:00`}
          />
          <YAxis
            tick={{ fill: "#bdc9c8", fontSize: 10 }}
            axisLine={false}
            tickLine={false}
            tickFormatter={(v: number) => `€${v}`}
          />
          <Tooltip content={<CustomTooltip />} />
          <ReferenceLine y={0} stroke="#3e4949" strokeDasharray="4 4" />
          <Area
            type="monotone"
            dataKey="spread"
            fill="#76d6d5"
            fillOpacity={0.1}
            stroke="transparent"
            name="Spread"
          />
          <Line
            type="monotone"
            dataKey="pl"
            stroke="#76d6d5"
            strokeWidth={2}
            dot={false}
            name="Poland"
          />
          <Line
            type="monotone"
            dataKey="de"
            stroke="#ffb692"
            strokeWidth={2}
            dot={false}
            name="Germany"
          />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );
}
