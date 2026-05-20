import type { Session, LiveStatus } from "../lib/types";
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

// (unchanged) infer the active context window from several signals.
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
  onRefresh: () => void;
}

function StatusDot({ status }: { status: LiveStatus | null }) {
  if (!status) return <span aria-hidden className="w-2 h-2" />; // placeholder for grid alignment
  const cls = status === "busy" ? "bg-ok" : "bg-text-3";
  const label = status === "busy" ? "Busy" : "Idle";
  return <span aria-label={label} title={label} className={`w-2 h-2 rounded-full ${cls}`} />;
}

function BgStateDot({ state }: { state: string | null }) {
  const cls =
    state === "running" ? "bg-warn"
    : state === "done" ? "bg-ok"
    : state === "error" ? "bg-danger"
    : "bg-text-3";
  const label = state ?? "unknown";
  return <span aria-label={label} title={label} className={`w-2 h-2 rounded-full ${cls}`} />;
}

export function SessionRow({ session, cwd, projectUsed1m, onRefresh }: Props) {
  // Renamed from `window` to avoid shadowing the global `window` object
  // (we need `window.confirm` / `window.alert` below).
  const ctxWindow = windowFor(
    session.model,
    session.max_prompt_tokens,
    projectUsed1m,
    session.live_context_window,
  );
  const displayModel = session.live_model_id ?? session.model;
  const isLive = session.live_status !== null;

  async function onClose() {
    if (!window.confirm("Close this session? Unsaved work in the session may be lost.")) return;
    try {
      await api.closeSession(session.id);
    } catch (err) {
      window.alert(String(err));
    } finally {
      onRefresh();
    }
  }

  return (
    <div className="grid grid-cols-[auto_1fr_auto_220px_auto_auto] items-center gap-3 py-2 px-3 border-t border-border hover:bg-surface-hi">
      <StatusDot status={session.live_status} />
      <div className="min-w-0">
        {session.is_bg_agent ? (
          <>
            <div className="truncate text-text-1 text-sm">
              {session.bg_name ?? session.bg_intent ?? session.title ?? "(no name)"}
            </div>
            <div className="text-[11px] text-text-3 truncate flex items-center gap-2">
              <BgStateDot state={session.bg_state} />
              <span className="font-mono tracking-wide">
                {(session.bg_state ?? "unknown").toUpperCase()}
              </span>
              {session.bg_detail && (
                <>
                  <span aria-hidden>·</span>
                  <span className="truncate">{session.bg_detail}</span>
                </>
              )}
            </div>
          </>
        ) : (
          <>
            <div className="truncate text-text-1 text-sm">
              {session.title ?? "(no prompt yet)"}
            </div>
            <div className="font-mono text-[11px] text-text-3 truncate">
              {displayModel ?? "—"} · {session.message_count} msgs · {formatTokens(session.tokens)} tok lifetime
            </div>
          </>
        )}
      </div>
      <span className="text-text-3 text-xs whitespace-nowrap">
        {formatTimeAgo(session.last_activity)}
      </span>
      <ContextMeter tokens={session.context_tokens} window={ctxWindow} />
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
      {isLive ? (
        <button
          type="button"
          aria-label="Close session"
          title="Close session"
          onClick={onClose}
          className="px-2 py-1 text-sm rounded-md text-text-3 hover:bg-danger hover:text-text-1"
        >
          ×
        </button>
      ) : (
        <span aria-hidden className="w-0" />
      )}
    </div>
  );
}
