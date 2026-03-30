"use client";

import { useState, useEffect } from "react";
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  Tooltip,
  Brush,
  ResponsiveContainer,
} from "recharts";
import { DateRangeSelector } from "@/components/ui/DateRangeSelector";

interface PseHistoricalChartProps {
  title: string;
  yLabel: string;
  seriesKey: "cen" | "ckoeb" | "sdac";
  color: string;
  defaultDays?: number;
}

export default function PseHistoricalChart({
  title,
  yLabel,
  seriesKey,
  color,
  defaultDays = 30,
}: PseHistoricalChartProps) {
  const [range, setRange] = useState(() => {
    const to = new Date().toISOString().split("T")[0];
    const from = new Date(Date.now() - defaultDays * 86400_000)
      .toISOString()
      .split("T")[0];
    return { from, to };
  });
  const [data, setData] = useState<Array<{ ts: string; value: number | null }>>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const apiBase = (process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:8080").trim();

  useEffect(() => {
    setLoading(true);
    setError(null);
    const url = `${apiBase}/api/history/prices?from=${range.from}&to=${range.to}&resolution=daily`;

    fetch(url)
      .then((r) => {
        if (!r.ok) throw new Error(`HTTP ${r.status}`);
        return r.json();
      })
      .then((json) => {
        setData(json.series?.[seriesKey] ?? []);
        setLoading(false);
      })
      .catch((err) => {
        console.error(`PseHistoricalChart fetch error: ${err.message}`, url);
        setError(err.message);
        setLoading(false);
      });
  }, [range, apiBase, seriesKey]);

  return (
    <div className="bg-surface-container p-6 rounded-xl">
      <div className="flex items-center justify-between mb-4">
        <h2 className="font-headline text-base font-bold text-on-surface">
          {title}
        </h2>
        <DateRangeSelector value={range} onChange={setRange} />
      </div>

      {loading ? (
        <div className="h-48 flex items-center justify-center">
          <span className="material-symbols-outlined text-primary animate-spin">
            refresh
          </span>
        </div>
      ) : error ? (
        <div className="h-48 flex items-center justify-center">
          <p className="text-sm text-error">Failed to load data: {error}</p>
        </div>
      ) : data.length === 0 ? (
        <div className="h-48 flex items-center justify-center">
          <p className="text-sm text-on-surface-variant">
            No data for selected period. Run backfill to populate history.
          </p>
        </div>
      ) : (
        <ResponsiveContainer width="100%" height={220}>
          <LineChart
            data={data}
            margin={{ top: 4, right: 8, bottom: 4, left: 8 }}
          >
            <XAxis
              dataKey="ts"
              tick={{ fontSize: 10, fill: "var(--color-on-surface-variant)" }}
              tickFormatter={(ts) =>
                new Date(ts).toLocaleDateString("pl-PL", {
                  day: "numeric",
                  month: "short",
                })
              }
              tickLine={false}
              axisLine={false}
            />
            <YAxis
              tick={{ fontSize: 10, fill: "var(--color-on-surface-variant)" }}
              tickLine={false}
              axisLine={false}
              label={{
                value: yLabel,
                angle: -90,
                position: "insideLeft",
                style: { fontSize: 9, fill: "var(--color-on-surface-variant)" },
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
              labelFormatter={(ts) =>
                new Date(ts as string).toLocaleDateString("pl-PL", {
                  weekday: "short",
                  day: "numeric",
                  month: "short",
                  year: "numeric",
                })
              }
            />
            <Line
              type="monotone"
              dataKey="value"
              name={yLabel}
              stroke={color}
              strokeWidth={1.5}
              dot={false}
              activeDot={{ r: 3 }}
              connectNulls
            />
            <Brush
              dataKey="ts"
              height={20}
              stroke="var(--color-outline-variant)"
              fill="var(--color-surface-container-low)"
              travellerWidth={6}
              tickFormatter={(ts) =>
                new Date(ts).toLocaleDateString("pl-PL", {
                  day: "numeric",
                  month: "short",
                })
              }
            />
          </LineChart>
        </ResponsiveContainer>
      )}
    </div>
  );
}
