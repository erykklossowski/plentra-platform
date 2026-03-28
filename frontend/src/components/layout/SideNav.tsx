"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { cn } from "@/lib/utils";

const navItems = [
  { href: "/summary", icon: "dashboard", label: "Summary" },
  { href: "/generation", icon: "bolt", label: "Generation" },
  { href: "/stability", icon: "timeline", label: "Stability" },
  { href: "/reserves", icon: "speed", label: "Reserves" },
  { href: "/crossborder", icon: "swap_horiz", label: "Cross-Border" },
  { href: "/europe", icon: "public", label: "Europe" },
];

export default function SideNav() {
  const pathname = usePathname();

  return (
    <aside className="fixed left-0 top-16 h-[calc(100vh-64px)] w-64 bg-surface-container-low flex flex-col py-4 z-40 no-print">
      <div className="px-6 mb-6">
        <p className="text-primary font-bold">Market Insights</p>
        <p className="text-slate-500 text-[10px]">Plentra Intelligence</p>
      </div>
      <div className="flex-1 space-y-1">
        {navItems.map((item) => {
          const isActive = pathname === item.href;
          return (
            <Link
              key={item.href}
              href={item.href}
              className={cn(
                "flex items-center gap-4 px-6 py-3 transition-all",
                isActive
                  ? "bg-linear-to-r from-primary/10 to-transparent text-primary border-l-4 border-primary translate-x-1"
                  : "text-slate-400 hover:text-slate-200 hover:bg-surface-container-high"
              )}
            >
              <span className="material-symbols-outlined">{item.icon}</span>
              {item.label}
            </Link>
          );
        })}
      </div>
      <div className="px-6 py-4 space-y-3 mt-auto border-t border-outline-variant/10">
        <a
          className="flex items-center gap-4 text-slate-400 hover:text-slate-200 transition-colors"
          href="#"
        >
          <span className="material-symbols-outlined">settings</span>
          Settings
        </a>
        <a
          className="flex items-center gap-4 text-slate-400 hover:text-slate-200 transition-colors"
          href="#"
        >
          <span className="material-symbols-outlined">help_outline</span>
          Help Center
        </a>
      </div>
    </aside>
  );
}
