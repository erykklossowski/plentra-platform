"use client";

import {
  ComposedChart,
  Line,
  Area,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
} from "recharts";

interface FanChartProps {
  ticker: string;
  lastHistorical: number;
  trainingPoints: number;
  pointForecast: number[];
  lower80: number[];
  upper80: number[];
  lower95: number[];
  upper95: number[];
  history30d?: number[];
}

export default function FanChart({
  ticker,
  lastHistorical,
  trainingPoints,
  pointForecast,
  lower80,
  upper80,
  lower95,
  upper95,
  history30d,
}: FanChartProps) {
  // Build combined data: last 30d of history + 14d forecast
  const data: Record<string, unknown>[] = [];

  // Historical points (last 30 or available)
  const hist = history30d ?? [];
  for (let i = 0; i < hist.length; i++) {
    data.push({
      label: `H-${hist.length - i}`,
      historical: hist[i],
    });
  }

  // Forecast points
  for (let i = 0; i < pointForecast.length; i++) {
    data.push({
      label: `D+${i + 1}`,
      forecast: pointForecast[i],
      band95: lower95[i] !== undefined && upper95[i] !== undefined
        ? [lower95[i], upper95[i]]
        : undefined,
      band80: lower80[i] !== undefined && upper80[i] !== undefined
        ? [lower80[i], upper80[i]]
        : undefined,
      lower95: lower95[i],
      upper95: upper95[i],
      lower80: lower80[i],
      upper80: upper80[i],
    });
  }

  return (
    <div className="bg-surface-container p-5 rounded-xl">
      <div className="flex items-center justify-between mb-1">
        <h3 className="font-headline text-sm font-bold text-on-surface">
          {ticker}
        </h3>
        <span className="text-[9px] bg-surface-container-high text-on-surface-variant/70 px-2 py-0.5 rounded-lg">
          ETS · {trainingPoints} pts
        </span>
      </div>
      <p className="text-xl font-headline font-bold text-on-surface mb-3">
        {lastHistorical.toFixed(2)}
        <span className="text-xs text-on-surface-variant ml-1 font-normal">
          last close
        </span>
      </p>

      <ResponsiveContainer width="100%" height={180}>
        <ComposedChart data={data} margin={{ top: 4, right: 8, bottom: 4, left: 8 }}>
          <XAxis
            dataKey="label"
            tick={{ fontSize: 8, fill: "var(--color-on-surface-variant)" }}
            tickLine={false}
            axisLine={false}
            interval="preserveStartEnd"
          />
          <YAxis
            tick={{ fontSize: 9, fill: "var(--color-on-surface-variant)" }}
            tickLine={false}
            axisLine={false}
            domain={["auto", "auto"]}
          />
          <Tooltip
            contentStyle={{
              background: "rgba(49,57,77,0.92)",
              backdropFilter: "blur(16px)",
              border: "none",
              borderRadius: "0.5rem",
              fontSize: "0.7rem",
            }}
          />

          {/* 95% CI band */}
          <Area
            dataKey="upper95"
            stroke="none"
            fill="#76d6d5"
            fillOpacity={0.08}
            type="monotone"
          />
          <Area
            dataKey="lower95"
            stroke="none"
            fill="var(--color-surface-container)"
            fillOpacity={1}
            type="monotone"
          />

          {/* 80% CI band */}
          <Area
            dataKey="upper80"
            stroke="none"
            fill="#76d6d5"
            fillOpacity={0.15}
            type="monotone"
          />
          <Area
            dataKey="lower80"
            stroke="none"
            fill="var(--color-surface-container)"
            fillOpacity={1}
            type="monotone"
          />

          {/* Historical line */}
          <Line
            dataKey="historical"
            stroke="#76d6d5"
            strokeWidth={1.5}
            dot={false}
            type="monotone"
            name="Historical"
          />

          {/* Forecast line */}
          <Line
            dataKey="forecast"
            stroke="#76d6d5"
            strokeWidth={1.5}
            strokeDasharray="4 4"
            dot={false}
            type="monotone"
            name="Forecast"
          />
        </ComposedChart>
      </ResponsiveContainer>
    </div>
  );
}
