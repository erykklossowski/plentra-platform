"use client";

import { useState } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { cn } from "@/lib/utils";

const navItems = [
  { href: "/summary", icon: "dashboard", label: "Summary" },
  { href: "/generation", icon: "bolt", label: "Generation" },
  { href: "/stability", icon: "timeline", label: "Stability" },
  { href: "/crossborder", icon: "swap_horiz", label: "Cross-Border" },
  { href: "/europe", icon: "public", label: "Europe" },
];

export default function MobileNav() {
  const [open, setOpen] = useState(false);
  const pathname = usePathname();

  return (
    <div className="md:hidden no-print">
      <button
        onClick={() => setOpen(!open)}
        className="fixed top-4 right-4 z-50 p-2 text-on-surface"
      >
        <span className="material-symbols-outlined">
          {open ? "close" : "menu"}
        </span>
      </button>
      {open && (
        <div className="fixed inset-0 top-16 z-40 bg-surface-container-low p-6 space-y-2">
          {navItems.map((item) => {
            const isActive = pathname === item.href;
            return (
              <Link
                key={item.href}
                href={item.href}
                onClick={() => setOpen(false)}
                className={cn(
                  "flex items-center gap-4 px-4 py-3 rounded-lg transition-all",
                  isActive
                    ? "bg-primary/10 text-primary"
                    : "text-slate-400 hover:text-slate-200"
                )}
              >
                <span className="material-symbols-outlined">{item.icon}</span>
                {item.label}
              </Link>
            );
          })}
        </div>
      )}
    </div>
  );
}
