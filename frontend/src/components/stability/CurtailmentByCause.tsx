"use client";

import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from "recharts";
import SectionModule from "@/components/ui/SectionModule";
import type { CurtailmentHourly } from "@/types/api";

interface CurtailmentByCauseProps {
  hourlyProfile: CurtailmentHourly[];
}

export default function CurtailmentByCause({
  hourlyProfile,
}: CurtailmentByCauseProps) {
  const windData = hourlyProfile.map((h) => ({
    hour: `${String(h.hour).padStart(2, "0")}:00`,
    Balance: h.wind_balance_mwh,
    Network: h.wind_network_mwh,
  }));

  const solarData = hourlyProfile.map((h) => ({
    hour: `${String(h.hour).padStart(2, "0")}:00`,
    Balance: h.pv_balance_mwh,
    Network: h.pv_network_mwh,
  }));

  return (
    <SectionModule
      title="Curtailment by Cause"
      subtitle="Balance vs network constraint curtailment — 24h hourly breakdown (MWh)"
    >
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Wind */}
        <div>
          <p className="text-xs uppercase tracking-widest text-on-surface-variant mb-3 font-label">
            Wind Curtailment
          </p>
          <ResponsiveContainer width="100%" height={250}>
            <BarChart data={windData}>
              <CartesianGrid
                strokeDasharray="3 3"
                stroke="var(--color-outline-variant)"
                opacity={0.3}
              />
              <XAxis
                dataKey="hour"
                tick={{ fill: "var(--color-on-surface-variant)", fontSize: 10 }}
                interval={3}
              />
              <YAxis
                tick={{ fill: "var(--color-on-surface-variant)", fontSize: 10 }}
                label={{
                  value: "MWh",
                  angle: -90,
                  position: "insideLeft",
                  fill: "var(--color-on-surface-variant)",
                  fontSize: 10,
                }}
              />
              <Tooltip
                contentStyle={{
                  backgroundColor: "var(--color-surface-container-high)",
                  border: "1px solid var(--color-outline-variant)",
                  borderRadius: "8px",
                  color: "var(--color-on-surface)",
                  fontSize: 12,
                }}
              />
              <Legend
                wrapperStyle={{ fontSize: 11, color: "var(--color-on-surface-variant)" }}
              />
              <Bar
                dataKey="Balance"
                stackId="a"
                fill="var(--color-tertiary)"
                radius={[0, 0, 0, 0]}
              />
              <Bar
                dataKey="Network"
                stackId="a"
                fill="var(--color-error)"
                radius={[2, 2, 0, 0]}
              />
            </BarChart>
          </ResponsiveContainer>
        </div>

        {/* Solar */}
        <div>
          <p className="text-xs uppercase tracking-widest text-on-surface-variant mb-3 font-label">
            Solar Curtailment
          </p>
          <ResponsiveContainer width="100%" height={250}>
            <BarChart data={solarData}>
              <CartesianGrid
                strokeDasharray="3 3"
                stroke="var(--color-outline-variant)"
                opacity={0.3}
              />
              <XAxis
                dataKey="hour"
                tick={{ fill: "var(--color-on-surface-variant)", fontSize: 10 }}
                interval={3}
              />
              <YAxis
                tick={{ fill: "var(--color-on-surface-variant)", fontSize: 10 }}
                label={{
                  value: "MWh",
                  angle: -90,
                  position: "insideLeft",
                  fill: "var(--color-on-surface-variant)",
                  fontSize: 10,
                }}
              />
              <Tooltip
                contentStyle={{
                  backgroundColor: "var(--color-surface-container-high)",
                  border: "1px solid var(--color-outline-variant)",
                  borderRadius: "8px",
                  color: "var(--color-on-surface)",
                  fontSize: 12,
                }}
              />
              <Legend
                wrapperStyle={{ fontSize: 11, color: "var(--color-on-surface-variant)" }}
              />
              <Bar
                dataKey="Balance"
                stackId="a"
                fill="var(--color-tertiary)"
                radius={[0, 0, 0, 0]}
              />
              <Bar
                dataKey="Network"
                stackId="a"
                fill="var(--color-error)"
                radius={[2, 2, 0, 0]}
              />
            </BarChart>
          </ResponsiveContainer>
        </div>
      </div>
    </SectionModule>
  );
}
