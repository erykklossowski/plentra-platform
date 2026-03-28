"use client";

import type { ReservePrices, ReserveMonthlyHistory } from "@/types/api";

interface ReservePriceTableProps {
  prices: ReservePrices;
  history: ReserveMonthlyHistory[];
}

interface ReserveRow {
  product: string;
  direction: string;
  price: number;
  lastMonth: number;
}

function getPriceColor(price: number): string {
  if (price > 150) return "text-error";
  if (price < 80) return "text-emerald-400";
  return "text-on-surface";
}

function getDeltaText(current: number, lastMonth: number): string {
  if (lastMonth === 0) return "—";
  const pct = ((current - lastMonth) / lastMonth) * 100;
  const sign = pct >= 0 ? "+" : "";
  return `${sign}${pct.toFixed(1)}%`;
}

function getDeltaColor(current: number, lastMonth: number): string {
  if (lastMonth === 0) return "text-on-surface-variant";
  const pct = ((current - lastMonth) / lastMonth) * 100;
  if (pct > 10) return "text-error";
  if (pct < -10) return "text-emerald-400";
  return "text-on-surface-variant";
}

export default function ReservePriceTable({
  prices,
  history,
}: ReservePriceTableProps) {
  // Get last month's averages for comparison
  const lastMonth =
    history.length >= 2 ? history[history.length - 2] : undefined;

  const rows: ReserveRow[] = [
    {
      product: "FCR",
      direction: "↓",
      price: prices.fcr_d_pln_mw,
      lastMonth: lastMonth?.fcr_d ?? 0,
    },
    {
      product: "FCR",
      direction: "↑",
      price: prices.fcr_g_pln_mw,
      lastMonth: lastMonth?.fcr_g ?? 0,
    },
    {
      product: "aFRR",
      direction: "↓",
      price: prices.afrr_d_pln_mw,
      lastMonth: lastMonth?.afrr_d ?? 0,
    },
    {
      product: "aFRR",
      direction: "↑",
      price: prices.afrr_g_pln_mw,
      lastMonth: lastMonth?.afrr_g ?? 0,
    },
    {
      product: "mFRRd",
      direction: "↓",
      price: prices.mfrrd_d_pln_mw,
      lastMonth: lastMonth?.mfrrd_d ?? 0,
    },
    {
      product: "mFRRd",
      direction: "↑",
      price: prices.mfrrd_g_pln_mw,
      lastMonth: lastMonth?.mfrrd_g ?? 0,
    },
    {
      product: "RR",
      direction: "↑",
      price: prices.rr_g_pln_mw,
      lastMonth: lastMonth?.rr_g ?? 0,
    },
  ];

  return (
    <div className="overflow-x-auto">
      <table className="w-full text-sm">
        <thead>
          <tr className="border-b border-outline-variant/20">
            <th className="text-left py-3 px-4 text-[0.6875rem] uppercase tracking-widest text-on-surface-variant font-label">
              Product
            </th>
            <th className="text-center py-3 px-4 text-[0.6875rem] uppercase tracking-widest text-on-surface-variant font-label">
              Direction
            </th>
            <th className="text-right py-3 px-4 text-[0.6875rem] uppercase tracking-widest text-on-surface-variant font-label">
              Price (PLN/MW)
            </th>
            <th className="text-right py-3 px-4 text-[0.6875rem] uppercase tracking-widest text-on-surface-variant font-label">
              vs. Last Month
            </th>
          </tr>
        </thead>
        <tbody>
          {rows.map((row, i) => (
            <tr
              key={`${row.product}-${row.direction}`}
              className={
                i < rows.length - 1
                  ? "border-b border-outline-variant/10"
                  : ""
              }
            >
              <td className="py-3 px-4 font-headline font-bold text-on-surface">
                {row.product}
              </td>
              <td className="py-3 px-4 text-center text-on-surface-variant">
                {row.direction}
              </td>
              <td
                className={`py-3 px-4 text-right font-headline font-extrabold ${getPriceColor(row.price)}`}
              >
                {row.price.toFixed(1)}
              </td>
              <td
                className={`py-3 px-4 text-right text-xs ${getDeltaColor(row.price, row.lastMonth)}`}
              >
                {getDeltaText(row.price, row.lastMonth)}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
