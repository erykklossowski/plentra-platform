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
        {summary.retrospective_text ? (
          <p className="text-sm text-on-surface leading-relaxed font-body">
            {summary.retrospective_text}
          </p>
        ) : (
          <p className="text-sm text-on-surface-variant/50 italic">
            Generating analysis…
          </p>
        )}
        <div className="mt-4">
          <SignalChip
            label="Avg System Margin"
            value={`${summary.average_system_margin_pct}%`}
            sentiment={marginSentiment}
          />
        </div>
      </div>

      {/* Model Insights — only rendered when signals are present */}
      {summary.model_insights && (
        <div className="pt-5 border-t border-outline-variant/10">
          <div className="flex items-center gap-3 mb-3 flex-wrap">
            <span className="text-[10px] uppercase tracking-widest text-on-surface-variant font-label">
              ● Model Signals
            </span>
            {summary.model_insights_generated_at && (
              <span className="text-[9px] bg-surface-container-high text-on-surface-variant/70 px-2 py-0.5 rounded-lg">
                Augurs ·{" "}
                {new Date(summary.model_insights_generated_at).toLocaleDateString(
                  "en-GB",
                  { day: "numeric", month: "short" }
                )}
              </span>
            )}
            {summary.signals_summary?.map((sig: string) => {
              const [sigType] = sig.split(":");
              const colors: Record<string, string> = {
                residual_anomaly: "bg-tertiary/10 text-tertiary",
                structural_break: "bg-error/10 text-error",
                forecast_miss: "bg-tertiary/10 text-tertiary",
                dtw_analog: "bg-emerald-500/10 text-emerald-400",
              };
              const labels: Record<string, string> = {
                residual_anomaly: "Residual anomaly",
                structural_break: "Regime change",
                forecast_miss: "Forecast miss",
                dtw_analog: "Historical analog",
              };
              return (
                <span
                  key={sig}
                  className={`text-[9px] px-2 py-0.5 rounded-full font-medium ${
                    colors[sigType] ??
                    "bg-surface-container-high text-on-surface-variant"
                  }`}
                >
                  {labels[sigType] ?? sigType}
                </span>
              );
            })}
          </div>
          <p className="text-sm text-on-surface leading-relaxed font-body">
            {summary.model_insights}
          </p>
        </div>
      )}

      {/* Forward Risk Outlook */}
      <div className={summary.model_insights ? "pt-5 border-t border-outline-variant/10" : ""}>
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
