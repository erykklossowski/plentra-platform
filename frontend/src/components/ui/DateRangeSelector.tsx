"use client";

interface DateRangeSelectorProps {
  value: { from: string; to: string };
  onChange: (range: { from: string; to: string }) => void;
  className?: string;
}

const PRESETS = [
  { label: "7D", days: 7 },
  { label: "30D", days: 30 },
  { label: "90D", days: 90 },
  { label: "6M", days: 182 },
  { label: "1Y", days: 365 },
  { label: "YTD", days: null },
  { label: "All", days: 730 },
] as const;

export function DateRangeSelector({
  value,
  onChange,
  className,
}: DateRangeSelectorProps) {
  const activePreset = detectActivePreset(value);

  function setPreset(days: number | null) {
    const to = new Date().toISOString().split("T")[0];
    const from =
      days === null
        ? `${new Date().getFullYear()}-01-01`
        : new Date(Date.now() - days * 86400_000).toISOString().split("T")[0];
    onChange({ from, to });
  }

  return (
    <div className={`flex items-center gap-1 ${className ?? ""}`}>
      {PRESETS.map((p) => (
        <button
          key={p.label}
          onClick={() => setPreset(p.days)}
          className={`px-2.5 py-1 text-xs rounded-lg font-medium
                      transition-colors font-label tracking-wide
                      ${
                        activePreset === p.label
                          ? "bg-primary/20 text-primary"
                          : "text-on-surface-variant hover:bg-surface-container-high"
                      }`}
        >
          {p.label}
        </button>
      ))}
    </div>
  );
}

function detectActivePreset(range: { from: string; to: string }): string | null {
  const today = new Date().toISOString().split("T")[0];
  if (range.to !== today) return null;
  const days = Math.round(
    (Date.now() - new Date(range.from).getTime()) / 86400_000
  );

  // Check YTD
  const ytdFrom = `${new Date().getFullYear()}-01-01`;
  if (range.from === ytdFrom) return "YTD";

  const preset = [7, 30, 90, 182, 365, 730].find(
    (d) => Math.abs(d - days) < 2
  );
  if (!preset) return null;
  const labels: Record<number, string> = {
    7: "7D",
    30: "30D",
    90: "90D",
    182: "6M",
    365: "1Y",
    730: "All",
  };
  return labels[preset] ?? null;
}
