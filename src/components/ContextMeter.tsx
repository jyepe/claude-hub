interface Props {
  tokens: number;
  window: number;
}

export function ContextMeter({ tokens, window: max }: Props) {
  const ratio = Math.min(1, tokens / max);
  const pct = Math.round(ratio * 100);
  const band = ratio < 0.6 ? "ok" : ratio < 0.85 ? "warn" : "danger";
  return (
    <div className="flex items-center gap-2 min-w-0">
      <div className="h-1.5 rounded-sm bg-surface-hi overflow-hidden flex-1 min-w-[80px]">
        <div
          className={`h-full transition-all duration-[400ms] meter-fill-${band}`}
          style={{ width: `${pct}%` }}
        />
      </div>
      <span className="text-text-3 font-mono text-xs tabular-nums">{pct}%</span>
    </div>
  );
}
