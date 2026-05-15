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

// The JSONL only records the base model name (no `[1m]` suffix), so we infer
// the active context window from these signals, in priority order:
//   0. `liveWindow` — the truth from the statusline-cache wrapper (Option 5
//      in PROJECT.md: a wrapper script writes CC's actual used_percentage to
//      ~/.claude-hub/ctx-cache and the backend derives the window from it).
//   1. Explicit `[1m]` suffix in the model string  → 1M (definite).
//   2. Any assistant turn in the session whose prompt exceeded 200k → 1M
//      (observably impossible on the standard window).
//   3. The project's `lastModelUsage` in ~/.claude.json mentions a `[1m]`
//      variant → 1M (heuristic: this project is currently on the 1M beta).
// Fallback: model table, defaulting to 200k.
export function windowFor(
  model: string | null,
  maxPromptTokens: number,
  projectUsed1m: boolean,
  liveWindow: number | null = null,
): number {
  if (liveWindow && liveWindow > 0) return liveWindow;
  if (model && model.includes("[1m]")) return 1_000_000;
  if (maxPromptTokens > 200_000) return 1_000_000;
  if (projectUsed1m && model && /^claude-(opus|sonnet)-/.test(model)) {
    return 1_000_000;
  }
  if (!model) return 200_000;
  return MODEL_WINDOWS[model] ?? 200_000;
}

interface Props {
  session: Session;
  cwd: string;
  projectUsed1m: boolean;
}

export function SessionRow({ session, cwd, projectUsed1m }: Props) {
  const window = windowFor(
    session.model,
    session.max_prompt_tokens,
    projectUsed1m,
    session.live_context_window,
  );
  const displayModel = session.live_model_id ?? session.model;
  return (
    <div className="grid grid-cols-[1fr_auto_220px_auto] items-center gap-3 py-2 px-3 border-t border-border hover:bg-surface-hi">
      <div className="min-w-0">
        <div className="truncate text-text-1 text-sm">
          {session.title ?? "(no prompt yet)"}
        </div>
        <div className="font-mono text-[11px] text-text-3 truncate">
          {displayModel ?? "—"} · {session.message_count} msgs · {formatTokens(session.tokens)} tok lifetime
        </div>
      </div>
      <span className="text-text-3 text-xs whitespace-nowrap">
        {formatTimeAgo(session.last_activity)}
      </span>
      <ContextMeter tokens={session.context_tokens} window={window} />
      {session.is_bg_agent ? (
        <button
          type="button"
          onClick={() => api.attachAgent(cwd, session.id).catch(console.error)}
          className="px-3 py-1 text-sm rounded-md bg-accent hover:bg-accent-hover text-text-1"
        >
          Attach
        </button>
      ) : (
        <button
          type="button"
          onClick={() => api.openSession(cwd, session.id).catch(console.error)}
          className="px-3 py-1 text-sm rounded-md bg-accent hover:bg-accent-hover text-text-1"
        >
          Open
        </button>
      )}
    </div>
  );
}
