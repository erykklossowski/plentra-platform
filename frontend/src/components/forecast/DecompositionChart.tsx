"use client";

import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  ResponsiveContainer,
  ReferenceLine,
} from "recharts";

interface DecompositionChartProps {
  trend: number[];
  seasonal7d: number[];
  residual: number[];
}

export default function DecompositionChart({
  trend,
  seasonal7d,
  residual,
}: DecompositionChartProps) {
  const trendData = trend.map((v, i) => ({ i, v }));
  const seasonalData = seasonal7d.map((v, i) => ({ i, v }));
  const residualData = residual.map((v, i) => ({ i, v }));

  const charts = [
    { title: "Trend", data: trendData, color: "#76d6d5", showZero: false },
    {
      title: "Weekly Seasonal",
      data: seasonalData,
      color: "#ffb692",
      showZero: true,
    },
    {
      title: "Residual",
      data: residualData,
      color: "#ffb4ab",
      showZero: true,
    },
  ];

  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
      {charts.map((chart) => (
        <div key={chart.title} className="bg-surface-container p-4 rounded-xl">
          <p className="text-[10px] uppercase tracking-widest text-on-surface-variant font-label mb-2">
            {chart.title}
          </p>
          {chart.data.length > 0 && (
            <p className="font-headline text-lg font-bold text-on-surface mb-2">
              {chart.data[chart.data.length - 1].v.toFixed(2)}
            </p>
          )}
          <ResponsiveContainer width="100%" height={100}>
            <LineChart data={chart.data} margin={{ top: 2, right: 4, bottom: 2, left: 4 }}>
              <XAxis dataKey="i" hide />
              <YAxis hide domain={["auto", "auto"]} />
              {chart.showZero && (
                <ReferenceLine
                  y={0}
                  stroke="var(--color-outline-variant)"
                  strokeDasharray="4 4"
                />
              )}
              <Line
                dataKey="v"
                stroke={chart.color}
                strokeWidth={1.5}
                dot={false}
                type="monotone"
              />
            </LineChart>
          </ResponsiveContainer>
        </div>
      ))}
      <div className="md:col-span-3">
        <p className="text-xs text-on-surface-variant/60 italic">
          Residual component reflects fundamental market drivers (supply shocks,
          demand anomalies) not captured by seasonality.
        </p>
      </div>
    </div>
  );
}
