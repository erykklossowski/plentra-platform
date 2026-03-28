import { cn } from "@/lib/utils";

interface SignalChipProps {
  label: string;
  value: string | number;
  sentiment: "positive" | "negative" | "neutral" | "warning";
  className?: string;
}

const sentimentStyles = {
  positive: "bg-emerald-500/10 text-emerald-400",
  negative: "bg-error/10 text-error",
  warning: "bg-tertiary/10 text-tertiary",
  neutral: "bg-surface-container-high text-on-surface-variant",
} as const;

export default function SignalChip({
  label,
  value,
  sentiment,
  className,
}: SignalChipProps) {
  return (
    <span
      className={cn(
        "inline-flex items-center gap-2 px-3 py-1 rounded-full text-xs font-medium",
        sentimentStyles[sentiment],
        className
      )}
    >
      <span className="text-[0.625rem] uppercase tracking-widest opacity-80">
        {label}
      </span>
      <span className="font-semibold">{value}</span>
    </span>
  );
}
