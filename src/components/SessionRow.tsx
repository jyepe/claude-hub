import type { Session } from "../lib/types";
import { ContextMeter } from "./ContextMeter";
import { formatTimeAgo, formatTokens } from "../lib/format";
import { api } from "../lib/api";

const MODEL_WINDOWS: Record<string, number> = {
  "claude-opus-4-7": 200_000,
  "claude-opus-4-7[1m]": 1_000_000,
  "claude-sonnet-4-6": 200_000,
  "claude-sonnet-4-6[1m]": 1_000_000,
  "claude-haiku-4-5-20251001": 200_000,
};

function windowFor(model: string | null): number {
  if (!model) return 200_000;
  return MODEL_WINDOWS[model] ?? 200_000;
}

interface Props {
  session: Session;
  cwd: string;
}

export function SessionRow({ session, cwd }: Props) {
  const window = windowFor(session.model);
  return (
    <div className="grid grid-cols-[1fr_auto_220px_auto] items-center gap-3 py-2 px-3 border-t border-border hover:bg-surface-hi">
      <div className="min-w-0">
        <div className="truncate text-text-1 text-sm">
          {session.title ?? "(no prompt yet)"}
        </div>
        <div className="font-mono text-[11px] text-text-3 truncate">
          {session.model ?? "—"} · {session.message_count} msgs · {formatTokens(session.tokens)} tok
        </div>
      </div>
      <span className="text-text-3 text-xs whitespace-nowrap">
        {formatTimeAgo(session.last_activity)}
      </span>
      <ContextMeter tokens={session.tokens} window={window} />
      <button
        type="button"
        onClick={() => api.openSession(cwd, session.id).catch(console.error)}
        className="px-3 py-1 text-sm rounded-md bg-accent hover:bg-accent-hover text-white"
      >
        Open
      </button>
    </div>
  );
}
