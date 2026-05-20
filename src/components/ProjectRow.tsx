import { useEffect, useMemo, useRef, useState } from "react";
import type { Project } from "../lib/types";

interface Props {
  project: Project;
  selected: boolean;
  onSelect: (projectPath: string) => void;
  onHide: (project: Project) => void;
}

function anyLive(project: Project): boolean {
  if (project.sessions.some((s) => s.live_status !== null)) return true;
  for (const w of project.worktrees) {
    if (w.sessions.some((s) => s.live_status !== null)) return true;
  }
  return false;
}

function RowKebab({ onHide }: { onHide: () => void }) {
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
        className="w-6 h-6 inline-flex items-center justify-center rounded-sm text-text-3 hover:bg-border opacity-0 group-hover:opacity-100"
      >
        <span aria-hidden className="text-base leading-none">⋯</span>
      </button>
      {open && (
        <div
          role="menu"
          onClick={(e) => e.stopPropagation()}
          className="absolute right-0 top-full mt-1 z-20 min-w-[160px] bg-surface-hi border border-border rounded-md p-1 shadow-[0_1px_2px_rgba(0,0,0,0.12),0_8px_24px_rgba(0,0,0,0.28)]"
        >
          <button
            type="button"
            role="menuitem"
            onClick={() => { setOpen(false); onHide(); }}
            className="w-full text-left px-3 py-2 text-sm text-danger hover:bg-border rounded-sm"
          >
            Hide project
          </button>
        </div>
      )}
    </div>
  );
}

export function ProjectRow({ project, selected, onSelect, onHide }: Props) {
  const live = useMemo(() => anyLive(project), [project]);

  return (
    <div
      data-selected={selected}
      className={`group relative flex items-center gap-2 h-8 pr-1 pl-3 rounded-sm hover:bg-surface-hi ${selected ? "bg-surface-hi" : ""}`}
    >
      {selected && <span aria-hidden className="absolute left-0 top-1 bottom-1 w-0.5 bg-accent rounded-full" />}
      <button
        type="button"
        onClick={() => onSelect(project.path)}
        className="flex-1 min-w-0 text-left truncate text-[13px] text-text-1"
        aria-label={project.display_name}
      >
        {project.display_name}
      </button>
      {live && (
        <span
          data-testid="project-row-live-dot"
          aria-label="Live session"
          className="w-1.5 h-1.5 rounded-full bg-danger shrink-0"
        />
      )}
      <RowKebab onHide={() => onHide(project)} />
    </div>
  );
}
