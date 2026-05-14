import type { Stats } from "../lib/types";
import { formatTokens } from "../lib/format";

interface Props {
  stats: Stats | null;
}

const Tile = ({ label, value }: { label: string; value: string }) => (
  <div className="flex flex-col gap-1 px-5 py-4 border-r border-border last:border-r-0 min-w-[140px]">
    <span className="text-text-3 text-[11px] font-semibold uppercase tracking-[0.08em]">
      {label}
    </span>
    <span className="font-mono text-text-1 text-[22px] font-semibold tabular-nums">
      {value}
    </span>
  </div>
);

export function HeaderStats({ stats }: Props) {
  return (
    <div className="flex bg-surface border border-border rounded-md">
      <Tile label="Projects" value={stats ? String(stats.project_count) : "—"} />
      <Tile label="Sessions" value={stats ? String(stats.session_count) : "—"} />
      <Tile label="Tokens 7d" value={stats ? formatTokens(stats.tokens_7d) : "—"} />
      <Tile label="Tokens all-time" value={stats ? formatTokens(stats.tokens_all_time) : "—"} />
    </div>
  );
}
