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
        <div className="flex items-center gap-2 mb-3">
          <span className="w-2 h-2 rounded-full bg-emerald-500" />
          <span className="text-[0.6875rem] uppercase tracking-widest text-on-surface-variant font-medium">
            Retrospective Analysis
          </span>
        </div>
        <p className="text-sm text-on-surface-variant leading-relaxed">
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
