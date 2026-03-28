"use client";

import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ReferenceLine,
  ResponsiveContainer,
} from "recharts";
import type { HourlyProfile } from "@/types/api";

interface ResidualDemandChartProps {
  data: HourlyProfile[];
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
          {entry.name}: {entry.value.toFixed(2)} GW
        </p>
      ))}
    </div>
  );
}

export default function ResidualDemandChart({ data }: ResidualDemandChartProps) {
  const chartData = data.map((d) => ({
    ...d,
    label: `${d.hour.toString().padStart(2, "0")}:00`,
  }));

  // Find average must-run for reference line
  const avgMustRun =
    data.reduce((sum, d) => sum + d.must_run_gw, 0) / (data.length || 1);

  return (
    <ResponsiveContainer width="100%" height={280}>
      <AreaChart data={chartData}>
        <CartesianGrid stroke="#3e4949" strokeDasharray="4 4" opacity={0.3} />
        <XAxis
          dataKey="label"
          tick={{ fill: "#bdc9c8", fontSize: 10 }}
          axisLine={false}
          tickLine={false}
        />
        <YAxis
          tick={{ fill: "#bdc9c8", fontSize: 10 }}
          axisLine={false}
          tickLine={false}
          label={{
            value: "GW",
            angle: -90,
            position: "insideLeft",
            fill: "#879392",
            fontSize: 10,
          }}
        />
        <Tooltip content={<CustomTooltip />} />
        <ReferenceLine
          y={avgMustRun}
          stroke="#ffb692"
          strokeDasharray="6 3"
          strokeWidth={1.5}
          label={{
            value: `Must-run floor: ${avgMustRun.toFixed(1)} GW`,
            position: "right",
            fill: "#ffb692",
            fontSize: 10,
          }}
        />
        <Area
          type="monotone"
          dataKey="residual_gw"
          stroke="#76d6d5"
          strokeWidth={2}
          fill="#76d6d5"
          fillOpacity={0.15}
          name="Residual Demand"
        />
      </AreaChart>
    </ResponsiveContainer>
  );
}
