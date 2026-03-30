import type {
  FuelsResponse,
  SpreadsResponse,
  SummaryResponse,
  ResidualResponse,
  GenerationResponse,
  CrossBorderResponse,
  EuropeResponse,
  CurtailmentResponse,
  ReservesResponse,
  ForecastResponse,
  SpreadsAnalyticsResponse,
  EveningAnalyticsResponse,
  PsePricesResponse,
  ChangepointsResponse,
} from "@/types/api";

const API_BASE = (process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:8080").trim();

async function apiFetch<T>(path: string, revalidate = 900): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    next: { revalidate },
  });
  if (!res.ok) throw new Error(`API error: ${res.status} ${path}`);
  return (await res.json()) as T;
}

export const getSummary = () => apiFetch<SummaryResponse>("/api/summary");
export const getFuels = () => apiFetch<FuelsResponse>("/api/fuels");
export const getSpreads = () => apiFetch<SpreadsResponse>("/api/spreads");
export const getResidual = () => apiFetch<ResidualResponse>("/api/residual");
export const getGeneration = () => apiFetch<GenerationResponse>("/api/generation");
export const getCrossBorder = () => apiFetch<CrossBorderResponse>("/api/crossborder");
export const getEurope = () => apiFetch<EuropeResponse>("/api/europe");
export const getCurtailment = () => apiFetch<CurtailmentResponse>("/api/curtailment");
export const getReserves = () => apiFetch<ReservesResponse>("/api/reserves");
export const getForecast = () => apiFetch<ForecastResponse>("/api/forecast", 3600);
export const getSpreadsAnalytics = () => apiFetch<SpreadsAnalyticsResponse>("/api/analytics/spreads");
export const getEveningAnalytics = () => apiFetch<EveningAnalyticsResponse>("/api/analytics/evening");
export const getChangepoints = () => apiFetch<ChangepointsResponse>("/api/analytics/changepoints", 3600);

export function getPsePrices(days = 30) {
  const to = new Date().toISOString().split("T")[0];
  const from = new Date(Date.now() - days * 86400_000).toISOString().split("T")[0];
  return apiFetch<PsePricesResponse>(`/api/history/prices?from=${from}&to=${to}`, 300);
}
