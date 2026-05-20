import type { Session } from "../lib/types";
import { api } from "../lib/api";

interface Props {
  session: Session;
  projectPath: string;
  dimmed: boolean;
  onClick: (projectPath: string, sessionId: string) => void;
  onMutate: () => void;
}

function shortModel(model: string | null): string {
  if (!model) return "";
  const m = model.toLowerCase();
  if (m.includes("opus")) return "opus";
  if (m.includes("sonnet")) return "sonnet";
  if (m.includes("haiku")) return "haiku";
  return model;
}

export function PinnedRow({ session, projectPath, dimmed, onClick, onMutate }: Props) {
  const label = session.title ?? session.bg_name ?? "(no title)";
  const model = shortModel(session.live_model_id ?? session.model);

  async function handleUnpin(e: React.MouseEvent) {
    e.stopPropagation();
    try { await api.unpinSession(session.id); } catch (err) { console.error(err); }
    onMutate();
  }

  return (
    <div className="group relative w-full">
      <button
        type="button"
        onClick={() => onClick(projectPath, session.id)}
        aria-label={label}
        className={`w-full flex items-center gap-2 h-7 px-2 rounded-sm text-left hover:bg-surface-hi ${dimmed ? "opacity-40" : ""}`}
      >
        <span aria-hidden className="w-1 h-1 rounded-full bg-accent shrink-0" />
        <span className="flex-1 truncate text-[13px] text-text-1">{label}</span>
        <span className="font-mono text-[11px] text-text-3 uppercase tracking-wide shrink-0 group-hover:hidden">
          {model}
        </span>
        <span aria-hidden className="hidden group-hover:inline-block w-5 h-5 shrink-0" />
      </button>
      <button
        type="button"
        onClick={handleUnpin}
        aria-label="Unpin"
        className="absolute right-2 top-1/2 -translate-y-1/2 hidden group-hover:inline-flex items-center justify-center w-5 h-5 text-text-3 hover:text-text-1"
      >
        −
      </button>
    </div>
  );
}
