import type {
  FuelsResponse,
  SpreadsResponse,
  SummaryResponse,
  ResidualResponse,
} from "@/types/api";

const API_BASE = process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:8080";

async function apiFetch<T>(path: string): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    next: { revalidate: 900 },
  });
  if (!res.ok) throw new Error(`API error: ${res.status} ${path}`);
  const data = await res.json();
  if (data.error) throw new Error(`API error: ${data.error}`);
  return data as T;
}

export const getSummary = () => apiFetch<SummaryResponse>("/api/summary");
export const getFuels = () => apiFetch<FuelsResponse>("/api/fuels");
export const getSpreads = () => apiFetch<SpreadsResponse>("/api/spreads");
export const getResidual = () => apiFetch<ResidualResponse>("/api/residual");
