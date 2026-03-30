"use client";

import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  Tooltip,
  ReferenceLine,
  ResponsiveContainer,
  Brush,
} from "recharts";
import type { SpreadHistoryPoint } from "@/types/api";

interface SpreadHistoryChartProps {
  data: SpreadHistoryPoint[];
}

export default function SpreadHistoryChart({ data }: SpreadHistoryChartProps) {
  // Pivot data by date: { date, css, cds, css_30d, cds_30d }
  const byDate = new Map<string, Record<string, number | null>>();
  for (const p of data) {
    if (!byDate.has(p.date)) byDate.set(p.date, { date: null } as never);
    const row = byDate.get(p.date)!;
    (row as Record<string, unknown>).date = p.date;
    if (p.spread_type === "rolling_3m_css") {
      (row as Record<string, unknown>).css = p.value;
      (row as Record<string, unknown>).css_30d = p.rolling_30d_avg;
    } else if (p.spread_type === "rolling_3m_cds") {
      (row as Record<string, unknown>).cds = p.value;
      (row as Record<string, unknown>).cds_30d = p.rolling_30d_avg;
    }
  }
  const chartData = Array.from(byDate.values());

  if (chartData.length === 0) {
    return (
      <div className="h-48 flex items-center justify-center">
        <p className="text-sm text-on-surface-variant">No spread data available</p>
      </div>
    );
  }

  return (
    <ResponsiveContainer width="100%" height={280}>
      <LineChart data={chartData} margin={{ top: 4, right: 8, bottom: 4, left: 8 }}>
        <XAxis
          dataKey="date"
          tick={{ fontSize: 10, fill: "var(--color-on-surface-variant)" }}
          tickFormatter={(d) =>
            new Date(d).toLocaleDateString("pl-PL", { day: "numeric", month: "short" })
          }
          tickLine={false}
          axisLine={false}
        />
        <YAxis
          tick={{ fontSize: 10, fill: "var(--color-on-surface-variant)" }}
          tickLine={false}
          axisLine={false}
          label={{
            value: "EUR/MWh",
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
          labelFormatter={(d) =>
            new Date(d as string).toLocaleDateString("pl-PL", {
              weekday: "short",
              day: "numeric",
              month: "short",
              year: "numeric",
            })
          }
        />
        <ReferenceLine y={0} stroke="var(--color-outline-variant)" strokeDasharray="4 4" />
        <Line type="monotone" dataKey="css" name="CSS" stroke="#76d6d5" strokeWidth={1.5} dot={false} activeDot={{ r: 3 }} />
        <Line type="monotone" dataKey="cds" name="CDS" stroke="#ffb692" strokeWidth={1.5} strokeDasharray="4 4" dot={false} activeDot={{ r: 3 }} />
        <Line type="monotone" dataKey="css_30d" name="CSS 30d avg" stroke="#76d6d5" strokeWidth={1} strokeOpacity={0.35} dot={false} />
        <Line type="monotone" dataKey="cds_30d" name="CDS 30d avg" stroke="#ffb692" strokeWidth={1} strokeOpacity={0.35} dot={false} />
        <Brush
          dataKey="date"
          height={20}
          stroke="var(--color-outline-variant)"
          fill="var(--color-surface-container-low)"
          travellerWidth={6}
          tickFormatter={(d) =>
            new Date(d).toLocaleDateString("pl-PL", { day: "numeric", month: "short" })
          }
        />
      </LineChart>
    </ResponsiveContainer>
  );
}
