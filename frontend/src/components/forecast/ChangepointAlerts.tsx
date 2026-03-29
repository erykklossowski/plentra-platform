interface ChangepointAlert {
  ticker: string;
  alert: boolean;
  message: string;
  latest_break_index?: number;
}

interface Props {
  alert: ChangepointAlert | null;
}

export default function ChangepointAlerts({ alert }: Props) {
  if (alert) {
    return (
      <div className="bg-tertiary/10 rounded-xl p-4 flex items-start gap-3">
        <span className="material-symbols-outlined text-tertiary mt-0.5">
          warning
        </span>
        <div>
          <p className="text-sm font-bold text-tertiary">
            Structural Break Detected
          </p>
          <p className="text-xs text-on-surface-variant mt-1">
            {alert.message}
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex items-center gap-2">
      <span className="material-symbols-outlined text-emerald-400 text-sm">
        check_circle
      </span>
      <p className="text-xs text-on-surface-variant">
        No structural breaks detected in recent price series.
      </p>
    </div>
  );
}
