"use client";

import { useState } from "react";
import type { JKZEntry } from "@/types/api";

interface JKZTableProps {
  data: JKZEntry[];
  daPrice: number;
}

type SortKey = "technology" | "jkz_eur_mwh" | "clean_spread_eur_mwh";

export default function JKZTable({ data, daPrice }: JKZTableProps) {
  const [sortKey, setSortKey] = useState<SortKey>("jkz_eur_mwh");
  const [sortAsc, setSortAsc] = useState(true);

  const sorted = [...data].sort((a, b) => {
    const av = sortKey === "technology" ? a.technology : a[sortKey];
    const bv = sortKey === "technology" ? b.technology : b[sortKey];
    if (typeof av === "string" && typeof bv === "string") {
      return sortAsc ? av.localeCompare(bv) : bv.localeCompare(av);
    }
    return sortAsc ? (av as number) - (bv as number) : (bv as number) - (av as number);
  });

  const handleSort = (key: SortKey) => {
    if (key === sortKey) {
      setSortAsc(!sortAsc);
    } else {
      setSortKey(key);
      setSortAsc(true);
    }
  };

  const SortIcon = ({ col }: { col: SortKey }) => {
    if (col !== sortKey)
      return (
        <span className="material-symbols-outlined text-[14px] text-outline opacity-40">
          unfold_more
        </span>
      );
    return (
      <span className="material-symbols-outlined text-[14px] text-primary">
        {sortAsc ? "expand_less" : "expand_more"}
      </span>
    );
  };

  return (
    <div className="bg-surface-container rounded-xl overflow-hidden">
      <div className="px-6 pt-6 pb-3 flex items-center justify-between">
        <div>
          <h3 className="font-headline text-lg font-bold text-on-surface">
            Unit Variable Costs (JKZ)
          </h3>
          <p className="text-xs text-on-surface-variant mt-0.5">
            Jednostkowe Koszty Zmienne — fuel + CO₂ cost per MWh vs. DA price €
            {daPrice.toFixed(2)}
          </p>
        </div>
      </div>
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="text-[0.625rem] uppercase tracking-widest text-on-surface-variant/60">
              <th
                className="text-left px-6 py-2 cursor-pointer select-none"
                onClick={() => handleSort("technology")}
              >
                <span className="flex items-center gap-1">
                  Technology <SortIcon col="technology" />
                </span>
              </th>
              <th className="text-right px-4 py-2">η</th>
              <th className="text-right px-4 py-2">Fuel Cost</th>
              <th className="text-right px-4 py-2">CO₂ Cost</th>
              <th
                className="text-right px-4 py-2 cursor-pointer select-none"
                onClick={() => handleSort("jkz_eur_mwh")}
              >
                <span className="flex items-center justify-end gap-1">
                  JKZ <SortIcon col="jkz_eur_mwh" />
                </span>
              </th>
              <th
                className="text-right px-4 py-2 cursor-pointer select-none"
                onClick={() => handleSort("clean_spread_eur_mwh")}
              >
                <span className="flex items-center justify-end gap-1">
                  Spread <SortIcon col="clean_spread_eur_mwh" />
                </span>
              </th>
              <th className="text-center px-4 py-2">Status</th>
            </tr>
          </thead>
          <tbody>
            {sorted.map((entry) => (
              <tr
                key={entry.technology}
                className="hover:bg-surface-container-high transition-colors"
              >
                <td className="px-6 py-3 font-medium text-on-surface">
                  {entry.technology}
                </td>
                <td className="text-right px-4 py-3 text-on-surface-variant">
                  {(entry.efficiency * 100).toFixed(0)}%
                </td>
                <td className="text-right px-4 py-3 text-on-surface-variant">
                  €{entry.fuel_cost_eur_mwh.toFixed(2)}
                </td>
                <td className="text-right px-4 py-3 text-on-surface-variant">
                  €{entry.co2_cost_eur_mwh.toFixed(2)}
                </td>
                <td className="text-right px-4 py-3 font-bold text-on-surface">
                  €{entry.jkz_eur_mwh.toFixed(2)}
                </td>
                <td
                  className={`text-right px-4 py-3 font-bold ${
                    entry.clean_spread_eur_mwh >= 0
                      ? "text-emerald-400"
                      : "text-error"
                  }`}
                >
                  {entry.clean_spread_eur_mwh >= 0 ? "+" : ""}
                  {entry.clean_spread_eur_mwh.toFixed(2)}
                </td>
                <td className="text-center px-4 py-3">
                  <StatusBadge status={entry.dispatch_status} />
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function StatusBadge({ status }: { status: string }) {
  const config: Record<string, string> = {
    IN_MERIT: "bg-emerald-500/10 text-emerald-400",
    OUT_OF_MERIT: "bg-error/10 text-error",
    MUST_RUN: "bg-primary/10 text-primary",
  };

  const labels: Record<string, string> = {
    IN_MERIT: "In Merit",
    OUT_OF_MERIT: "Out of Merit",
    MUST_RUN: "Must Run",
  };

  return (
    <span
      className={`inline-flex px-2 py-0.5 rounded-full text-[0.625rem] font-medium ${
        config[status] ?? "bg-surface-container-high text-on-surface-variant"
      }`}
    >
      {labels[status] ?? status}
    </span>
  );
}
