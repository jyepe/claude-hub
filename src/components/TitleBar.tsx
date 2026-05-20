import { useMemo } from "react";
import type { Session } from "../lib/types";
import { activeToday } from "../lib/format";
import { RefreshButton } from "./RefreshButton";
import wordmark from "../assets/claude-hub-wordmark.svg";
import pkg from "../../package.json";

interface Props {
  allSessions: Session[];
  onRefresh: () => void;
  lastRefresh: Date | null;
}

export function TitleBar({ allSessions, onRefresh, lastRefresh }: Props) {
  const active = useMemo(() => activeToday(allSessions), [allSessions]);
  return (
    <header className="h-11 px-4 flex items-center justify-between gap-3 bg-surface border-b border-border">
      <div className="flex items-center gap-3 min-w-0">
        <img src={wordmark} alt="Claude Hub" className="h-6" />
        <span className="text-[11px] font-mono text-text-3 px-1.5 py-0.5 rounded-sm border border-border">
          v{pkg.version}
        </span>
      </div>
      <div className="flex items-center gap-3">
        <div className="text-[11px] font-mono text-text-2 whitespace-nowrap">
          <span className="font-semibold text-text-1 tabular-nums">{active}</span> active today
        </div>
        <RefreshButton onRefresh={onRefresh} lastRefresh={lastRefresh} />
      </div>
    </header>
  );
}
