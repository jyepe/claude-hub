import { useEffect, useRef, useState } from "react";
import type { Session, LiveStatus } from "../lib/types";
import { ContextMeter } from "./ContextMeter";
import { api } from "../lib/api";
import { formatTimeAgo, formatTokens } from "../lib/format";
import { sessionLabel } from "../lib/sessionDisplay";
import { windowFor } from "./windowFor";

interface Props {
  session: Session;
  cwd: string;
  projectUsed1m: boolean;
  isPinned: boolean;
  focused: boolean;
  worktreeLeaf?: string | null;
  onMutate: () => void;
}

type ChipKind = "running" | "idle" | "done" | "error" | null;

function chipFor(session: Session): ChipKind {
  if (session.is_bg_agent) {
    const s = session.bg_state?.toLowerCase() ?? null;
    if (s === "running") return "running";
    if (s === "done") return "done";
    if (s === "error") return "error";
    return null;
  }
  const live = session.live_status as LiveStatus | null;
  if (live === "busy") return "running";
  if (live === "idle") return "idle";
  return null;
}

const CHIP_CLASS: Record<NonNullable<ChipKind>, string> = {
  running: "bg-warn/15 text-warn",
  idle: "bg-text-3/15 text-text-2",
  done: "bg-ok/15 text-ok",
  error: "bg-danger/15 text-text-1",
};

function StatusChip({ kind }: { kind: ChipKind }) {
  if (!kind) return <span aria-hidden className="h-[22px]" />;
  return (
    <span
      className={`inline-flex items-center gap-1.5 h-[22px] px-2 rounded-sm text-[11px] font-medium ${CHIP_CLASS[kind]}`}
    >
      <span aria-hidden className="w-1.5 h-1.5 rounded-full bg-current" />
      {kind}
    </span>
  );
}

function CardMenu({
  isPinned,
  isLive,
  onPin,
  onUnpin,
  onClose,
}: {
  isPinned: boolean;
  isLive: boolean;
  onPin: () => void;
  onUnpin: () => void;
  onClose: () => void;
}) {
  const [open, setOpen] = useState(false);
  const wrapRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const onMouseDown = (e: MouseEvent) => {
      if (!wrapRef.current?.contains(e.target as Node)) setOpen(false);
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setOpen(false);
    };
    window.addEventListener("mousedown", onMouseDown);
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("mousedown", onMouseDown);
      window.removeEventListener("keydown", onKey);
    };
  }, [open]);

  return (
    <div ref={wrapRef} className="relative">
      <button
        type="button"
        aria-label="More actions"
        aria-haspopup="menu"
        aria-expanded={open}
        onClick={(e) => {
          e.stopPropagation();
          setOpen((o) => !o);
        }}
        className="w-7 h-7 inline-flex items-center justify-center rounded-md border border-border bg-surface text-text-2 hover:bg-surface-hi transition-colors duration-[120ms] opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 focus:opacity-100"
      >
        <span aria-hidden className="text-base leading-none">⋯</span>
      </button>
      {open && (
        <div
          role="menu"
          onClick={(e) => e.stopPropagation()}
          className="absolute right-0 top-full mt-1 z-10 min-w-[160px] bg-surface-hi border border-border rounded-md p-1 shadow-[0_1px_2px_rgba(0,0,0,0.12),0_8px_24px_rgba(0,0,0,0.28)]"
        >
          {isPinned ? (
            <button
              type="button"
              role="menuitem"
              onClick={() => { setOpen(false); onUnpin(); }}
              className="w-full text-left px-3 py-2 text-sm text-text-1 hover:bg-border rounded-sm transition-colors duration-[120ms]"
            >
              Unpin
            </button>
          ) : (
            <button
              type="button"
              role="menuitem"
              onClick={() => { setOpen(false); onPin(); }}
              className="w-full text-left px-3 py-2 text-sm text-text-1 hover:bg-border rounded-sm transition-colors duration-[120ms]"
            >
              Pin
            </button>
          )}
          {isLive && (
            <button
              type="button"
              role="menuitem"
              onClick={() => { setOpen(false); onClose(); }}
              className="w-full text-left px-3 py-2 text-sm text-danger hover:bg-border rounded-sm transition-colors duration-[120ms]"
            >
              Close
            </button>
          )}
        </div>
      )}
    </div>
  );
}

export function SessionCard({
  session,
  cwd,
  projectUsed1m,
  isPinned,
  focused,
  worktreeLeaf,
  onMutate,
}: Props) {
  const ctxWindow = windowFor(
    session.model,
    session.max_prompt_tokens,
    projectUsed1m,
    session.live_context_window,
  );
  const displayModel = session.live_model_id ?? session.model ?? "—";
  const isLive = session.live_status !== null;
  const kind = chipFor(session);

  const title = sessionLabel(session);

  const body = session.recent_excerpt ?? (session.is_bg_agent ? session.bg_detail : null);

  async function handlePin() {
    try { await api.pinSession(session.id); } catch (e) { console.error(e); }
    onMutate();
  }
  async function handleUnpin() {
    try { await api.unpinSession(session.id); } catch (e) { console.error(e); }
    onMutate();
  }
  async function handleClose() {
    if (!window.confirm("Close this session? Unsaved work in the session may be lost.")) return;
    try { await api.closeSession(session.id); } catch (e) { window.alert(String(e)); }
    onMutate();
  }

  const focusClass = focused ? "outline outline-2 outline-accent outline-offset-2" : "";

  return (
    <div
      data-testid="session-card"
      data-session-id={session.id}
      className={`group flex flex-col gap-3 p-4 border border-border rounded-md bg-surface hover:border-text-3 min-h-[200px] ${focusClass}`}
    >
      <div className="flex items-start justify-between min-h-[24px]">
        <StatusChip kind={kind} />
        <CardMenu
          isPinned={isPinned}
          isLive={isLive}
          onPin={handlePin}
          onUnpin={handleUnpin}
          onClose={handleClose}
        />
      </div>

      <div className="text-text-1 text-sm font-semibold leading-snug line-clamp-2">
        {title}
      </div>

      {body && (
        <div className="text-text-2 text-[13px] leading-snug line-clamp-3">
          {body}
        </div>
      )}

      <div className="mt-auto">
        <ContextMeter tokens={session.context_tokens} window={ctxWindow} />
      </div>

      <div className="flex items-center justify-between pt-2 border-t border-border">
        <div className="font-mono text-[11px] text-text-3 truncate">
          {displayModel} · {formatTokens(session.tokens)} tok · {formatTimeAgo(session.last_activity)}
          {worktreeLeaf ? ` · worktree: ${worktreeLeaf}` : ""}
        </div>
        {session.is_bg_agent ? (
          <button
            type="button"
            onClick={() => api.attachAgent(cwd, session.id).catch(console.error)}
            className="text-accent text-[13px] hover:text-accent-hover whitespace-nowrap ml-3"
          >
            Attach →
          </button>
        ) : isLive ? (
          <span className="text-text-3 text-[13px] whitespace-nowrap ml-3">
            Already open
          </span>
        ) : (
          <button
            type="button"
            onClick={() => api.openSession(cwd, session.id).catch(console.error)}
            className="text-accent text-[13px] hover:text-accent-hover whitespace-nowrap ml-3"
          >
            Open session →
          </button>
        )}
      </div>
    </div>
  );
}
