interface DataLoadingCardProps {
  section: string;
  message?: string;
}

export default function DataLoadingCard({
  section,
  message,
}: DataLoadingCardProps) {
  return (
    <div className="p-8">
      <div className="bg-surface-container-high/50 rounded-xl p-6 flex items-center gap-4">
        <span className="material-symbols-outlined text-tertiary">sync</span>
        <div>
          <p className="text-sm font-medium text-on-surface">
            Data refresh in progress
          </p>
          <p className="text-xs text-on-surface-variant mt-1">
            {message ?? `${section} data is being loaded — reload in 30s`}
          </p>
        </div>
      </div>
    </div>
  );
}
