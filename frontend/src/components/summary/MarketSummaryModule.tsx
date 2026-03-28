import SignalChip from "@/components/ui/SignalChip";
import ForwardSignalTable from "./ForwardSignalTable";
import type { SummaryResponse } from "@/types/api";

interface Props {
  summary: SummaryResponse;
}

export default function MarketSummaryModule({ summary }: Props) {
  const marginSentiment =
    summary.system_margin_signal === "STABLE" ? "positive" : "warning";

  return (
    <div className="bg-surface-container p-6 rounded-xl space-y-6">
      {/* Retrospective Analysis */}
      <div>
        <div className="flex items-center gap-3 mb-4">
          <span className="text-[10px] uppercase tracking-widest text-on-surface-variant font-label">
            ● Retrospective Analysis
          </span>
          {summary.retrospective_generated_at && (
            <span className="text-[9px] bg-surface-container-high text-on-surface-variant/70 px-2 py-0.5 rounded-lg">
              AI · {new Date(summary.retrospective_generated_at).toLocaleDateString("en-GB", { day: "numeric", month: "short" })}
            </span>
          )}
          {summary.retrospective_stale && (
            <span className="text-[9px] text-tertiary">
              (cached)
            </span>
          )}
        </div>
        <p className="text-sm text-on-surface leading-relaxed font-body">
          {summary.retrospective_text}
        </p>
        <div className="mt-4">
          <SignalChip
            label="Avg System Margin"
            value={`${summary.average_system_margin_pct}%`}
            sentiment={marginSentiment}
          />
        </div>
      </div>

      {/* Forward Risk Outlook */}
      <div>
        <div className="flex items-center gap-2 mb-3">
          <span className="w-2 h-2 rounded-full bg-primary" />
          <span className="text-[0.6875rem] uppercase tracking-widest text-on-surface-variant font-medium">
            Forward Risk Outlook
          </span>
        </div>
        <ForwardSignalTable signals={summary.forward_signals} />
      </div>
    </div>
  );
}
