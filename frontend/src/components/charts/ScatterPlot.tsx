"use client";

import {
  ScatterChart,
  Scatter,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from "recharts";
import type { HourlyProfile } from "@/types/api";

interface ScatterPlotProps {
  data: HourlyProfile[];
  correlation: {
    r: number;
    r2: number;
    p: number;
  };
}

function CustomTooltip({
  active,
  payload,
}: {
  active?: boolean;
  payload?: Array<{ payload: { residual_gw: number; must_run_gw: number; hour: number } }>;
}) {
  if (!active || !payload?.[0]) return null;
  const d = payload[0].payload;

  return (
    <div
      className="rounded-lg px-3 py-2 text-xs text-on-surface"
      style={{
        background: "rgba(49, 57, 77, 0.85)",
        backdropFilter: "blur(16px)",
      }}
    >
      <p className="text-on-surface-variant mb-1">
        Hour {d.hour.toString().padStart(2, "0")}:00
      </p>
      <p>
        Residual: <span className="font-semibold">{d.residual_gw.toFixed(2)} GW</span>
      </p>
      <p>
        Must-run: <span className="font-semibold">{d.must_run_gw.toFixed(2)} GW</span>
      </p>
    </div>
  );
}

export default function ScatterPlot({ data, correlation }: ScatterPlotProps) {
  return (
    <div className="relative">
      <ResponsiveContainer width="100%" height={300}>
        <ScatterChart>
          <CartesianGrid stroke="#3e4949" strokeDasharray="4 4" opacity={0.3} />
          <XAxis
            dataKey="residual_gw"
            name="Residual Demand"
            tick={{ fill: "#bdc9c8", fontSize: 10 }}
            axisLine={false}
            tickLine={false}
            label={{
              value: "Residual Demand (GW)",
              position: "insideBottom",
              offset: -5,
              fill: "#879392",
              fontSize: 10,
            }}
          />
          <YAxis
            dataKey="must_run_gw"
            name="Must-Run Floor"
            tick={{ fill: "#bdc9c8", fontSize: 10 }}
            axisLine={false}
            tickLine={false}
            label={{
              value: "Must-Run (GW)",
              angle: -90,
              position: "insideLeft",
              fill: "#879392",
              fontSize: 10,
            }}
          />
          <Tooltip content={<CustomTooltip />} />
          <Scatter data={data} fill="#76d6d5" fillOpacity={0.8} r={5} />
        </ScatterChart>
      </ResponsiveContainer>

      {/* Correlation stats box */}
      <div className="absolute top-2 right-2 bg-surface-container-high/80 backdrop-blur-md px-3 py-2 rounded-lg">
        <p className="text-[0.5625rem] uppercase tracking-widest text-on-surface-variant mb-1">
          Correlation
        </p>
        <div className="space-y-0.5 text-xs">
          <p className="text-on-surface">
            r = <span className="font-semibold">{correlation.r.toFixed(2)}</span>
          </p>
          <p className="text-on-surface">
            R² = <span className="font-semibold">{correlation.r2.toFixed(2)}</span>
          </p>
          <p className="text-on-surface-variant">
            p = {correlation.p < 0.001 ? "< 0.001" : correlation.p.toFixed(3)}
          </p>
        </div>
      </div>
    </div>
  );
}
