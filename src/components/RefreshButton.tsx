import { formatTimeAgo } from "../lib/format";

interface Props {
  onRefresh: () => void;
  lastRefresh: Date | null;
}

export function RefreshButton({ onRefresh, lastRefresh }: Props) {
  return (
    <button
      type="button"
      onClick={onRefresh}
      className="flex items-center gap-2 px-3 py-1 text-sm rounded-md border border-border bg-surface hover:bg-surface-hi text-text-2"
    >
      <span>Refresh</span>
      <span className="text-text-3 text-xs">
        {lastRefresh ? formatTimeAgo(lastRefresh.toISOString()) : ""}
      </span>
    </button>
  );
}
