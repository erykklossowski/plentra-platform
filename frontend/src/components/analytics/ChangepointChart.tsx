"use client";

import {
  ComposedChart,
  Line,
  ReferenceLine,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from "recharts";
import type { ChangepointsResponse } from "@/types/api";

interface ChangepointChartProps {
  data: ChangepointsResponse | null;
}

export default function ChangepointChart({ data }: ChangepointChartProps) {
  if (!data || !data.series?.length) {
    return (
      <div className="bg-surface-container p-6 rounded-xl text-center">
        <p className="text-sm text-on-surface-variant">
          {data?.status === "insufficient_data"
            ? `Za malo danych — potrzeba min. ${data.min_required} dni. Uruchom backfill PSE.`
            : "Dane niedostepne."}
        </p>
      </div>
    );
  }

  const { series, changepoints } = data;

  const formatDate = (d: string) => {
    const date = new Date(d);
    return `${date.getDate()} ${date.toLocaleString("pl", { month: "short" })}`;
  };

  return (
    <div className="bg-surface-container p-6 rounded-xl">
      <p className="text-sm font-medium text-on-surface mb-1">
        SDAC PLN/MWh — 90 dni z zaznaczonymi punktami zmiany
      </p>
      <p className="text-xs text-on-surface-variant mb-4">
        Pionowe linie oznaczaja statystycznie istotne zmiany rezimu cenowego
      </p>

      <ResponsiveContainer width="100%" height={280}>
        <ComposedChart
          data={series}
          margin={{ top: 8, right: 16, bottom: 8, left: 0 }}
        >
          <CartesianGrid
            strokeDasharray="3 3"
            stroke="rgba(255,255,255,0.05)"
          />
          <XAxis
            dataKey="date"
            tickFormatter={formatDate}
            tick={{ fontSize: 10, fill: "var(--color-on-surface-variant)" }}
            interval={6}
            tickLine={false}
            axisLine={false}
          />
          <YAxis
            tick={{ fontSize: 10, fill: "var(--color-on-surface-variant)" }}
            tickLine={false}
            axisLine={false}
            label={{
              value: "PLN/MWh",
              angle: -90,
              position: "insideLeft",
              style: {
                fontSize: 9,
                fill: "var(--color-on-surface-variant)",
              },
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
            formatter={(value) => [
              `${Number(value).toFixed(1)} PLN/MWh`,
              "SDAC",
            ]}
            labelFormatter={(d) => formatDate(String(d))}
          />

          <Line
            type="monotone"
            dataKey="value"
            stroke="#76d6d5"
            strokeWidth={1.5}
            dot={false}
            name="SDAC"
          />

          {changepoints?.map((cp) => (
            <ReferenceLine
              key={cp.date}
              x={cp.date}
              stroke={cp.direction === "up" ? "#10b981" : "#f87171"}
              strokeWidth={2}
              strokeDasharray="4 2"
              label={{
                value: `${cp.direction === "up" ? "\u25B2" : "\u25BC"} ${Math.abs(cp.magnitude_pct)}%`,
                position: "top",
                style: {
                  fontSize: 10,
                  fill:
                    cp.direction === "up" ? "#10b981" : "#f87171",
                },
              }}
            />
          ))}
        </ComposedChart>
      </ResponsiveContainer>

      {/* Changepoint details table */}
      {changepoints?.length > 0 && (
        <div className="mt-4 space-y-2">
          <p className="text-[10px] uppercase tracking-widest text-on-surface-variant">
            Szczegoly zmian
          </p>
          <div className="grid grid-cols-4 gap-2 text-[10px] uppercase tracking-widest text-on-surface-variant/60 px-2">
            <span>Data</span>
            <span>Przed</span>
            <span>Po</span>
            <span>Zmiana</span>
          </div>
          {changepoints.map((cp) => (
            <div
              key={cp.date}
              className="grid grid-cols-4 gap-2 px-2 py-1.5 bg-surface-container-low rounded-lg text-sm"
            >
              <span className="text-on-surface font-medium">{cp.date}</span>
              <span className="text-on-surface-variant">
                {cp.price_before.toFixed(1)} PLN
              </span>
              <span className="text-on-surface-variant">
                {cp.price_after.toFixed(1)} PLN
              </span>
              <span
                className={
                  cp.direction === "up"
                    ? "text-emerald-400"
                    : "text-red-400"
                }
              >
                {cp.direction === "up" ? "+" : ""}
                {cp.magnitude_pct}%
              </span>
            </div>
          ))}
        </div>
      )}

      {changepoints?.length === 0 && (
        <div className="mt-4 flex items-center gap-2 text-sm text-on-surface-variant">
          <span className="material-symbols-outlined text-emerald-400 text-sm">
            check_circle
          </span>
          Brak statystycznie istotnych zmian strukturalnych w ostatnich 90 dniach.
        </div>
      )}
    </div>
  );
}
