"use client";

import dynamic from "next/dynamic";
import type {
  SpreadHistoryPoint,
  SeasonalityPoint,
  PositiveDaysPoint,
  EveningDecompositionPoint,
  ChangepointsResponse,
} from "@/types/api";

const SpreadHistoryChart = dynamic(
  () => import("@/components/analytics/SpreadHistoryChart"),
  { ssr: false }
);
const SeasonalityChart = dynamic(
  () => import("@/components/analytics/SeasonalityChart"),
  { ssr: false }
);
const PositiveDaysChart = dynamic(
  () => import("@/components/analytics/PositiveDaysChart"),
  { ssr: false }
);
const EveningDecompositionChart = dynamic(
  () => import("@/components/analytics/EveningDecompositionChart"),
  { ssr: false }
);
const ChangepointChart = dynamic(
  () => import("@/components/analytics/ChangepointChart"),
  { ssr: false }
);

export function SpreadHistoryChartWrapper({ data }: { data: SpreadHistoryPoint[] }) {
  return <SpreadHistoryChart data={data} />;
}

export function SeasonalityChartWrapper({ data }: { data: SeasonalityPoint[] }) {
  return <SeasonalityChart data={data} />;
}

export function PositiveDaysChartWrapper({ data }: { data: PositiveDaysPoint[] }) {
  return <PositiveDaysChart data={data} />;
}

export function EveningDecompositionChartWrapper({ data }: { data: EveningDecompositionPoint[] }) {
  return <EveningDecompositionChart data={data} />;
}

export function ChangepointChartWrapper({ data }: { data: ChangepointsResponse | null }) {
  return <ChangepointChart data={data} />;
}
