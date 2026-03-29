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
  ReferenceLine,
} from "recharts";
import { DateRangeSelector } from "@/components/ui/DateRangeSelector";

interface HistoricalChartProps {
  endpoint: string;
  title: string;
  yLabel: string;
  series: Array<{
    key: string;
    label: string;
    color: string;
    isDashed?: boolean;
  }>;
  showZeroLine?: boolean;
  defaultDays?: number;
}

export default function HistoricalChart({
  endpoint,
  title,
  yLabel,
  series,
  showZeroLine = false,
  defaultDays = 30,
}: HistoricalChartProps) {
  const [range, setRange] = useState(() => {
    const to = new Date().toISOString().split("T")[0];
    const from = new Date(Date.now() - defaultDays * 86400_000)
      .toISOString()
      .split("T")[0];
    return { from, to };
  });
  const [data, setData] = useState<Record<string, unknown>[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Trim whitespace/newlines from env var (Vercel sometimes appends \n)
  const apiBase = (process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:8080").trim();

  useEffect(() => {
    setLoading(true);
    setError(null);
    const separator = endpoint.includes("?") ? "&" : "?";
    const url = `${apiBase}${endpoint}${separator}from=${range.from}&to=${range.to}&resolution=daily`;

    fetch(url)
      .then((r) => {
        if (!r.ok) throw new Error(`HTTP ${r.status}`);
        return r.json();
      })
      .then((json) => {
        setData(json.points ?? []);
        setLoading(false);
      })
      .catch((err) => {
        console.error(`HistoricalChart fetch error: ${err.message}`, url);
        setError(err.message);
        setLoading(false);
      });
  }, [endpoint, range, apiBase]);

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
          <p className="text-sm text-error">
            Failed to load data: {error}
          </p>
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
              tick={{
                fontSize: 10,
                fill: "var(--color-on-surface-variant)",
              }}
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
              tick={{
                fontSize: 10,
                fill: "var(--color-on-surface-variant)",
              }}
              tickLine={false}
              axisLine={false}
              label={{
                value: yLabel,
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
              labelFormatter={(ts) =>
                new Date(ts as string).toLocaleDateString("pl-PL", {
                  weekday: "short",
                  day: "numeric",
                  month: "short",
                  year: "numeric",
                })
              }
            />
            {showZeroLine && (
              <ReferenceLine
                y={0}
                stroke="var(--color-outline-variant)"
                strokeDasharray="4 4"
              />
            )}
            {series.map((s) => (
              <Line
                key={s.key}
                type="monotone"
                dataKey={s.key}
                name={s.label}
                stroke={s.color}
                strokeWidth={1.5}
                strokeDasharray={s.isDashed ? "4 4" : undefined}
                dot={false}
                activeDot={{ r: 3 }}
              />
            ))}
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
