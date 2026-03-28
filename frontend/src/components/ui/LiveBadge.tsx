export default function LiveBadge() {
  return (
    <span className="bg-surface-container-high px-3 py-1.5 rounded text-on-surface-variant flex items-center gap-2 text-xs">
      <span className="w-2 h-2 rounded-full bg-primary animate-pulse" />
      Live Data Active
    </span>
  );
}
