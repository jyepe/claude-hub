import { useEffect, useRef, useState } from "react";
import type { Project } from "../lib/types";
import { SessionRow } from "./SessionRow";
import { api } from "../lib/api";
import { formatTimeAgo, formatProjectPath } from "../lib/format";

interface Props {
  project: Project;
  onMutate: () => void;
  onHide: (project: Project) => void;
}

function KebabMenu({
  onHide,
}: {
  onHide: () => void;
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
          <button
            type="button"
            role="menuitem"
            onClick={() => {
              setOpen(false);
              onHide();
            }}
            className="w-full text-left px-3 py-2 text-sm text-danger hover:bg-border rounded-sm transition-colors duration-[120ms]"
          >
            Hide project
          </button>
        </div>
      )}
    </div>
  );
}

export function ProjectCard({ project, onMutate, onHide }: Props) {
  const [open, setOpen] = useState(false);

  return (
    <div data-testid="project-card" className="group border border-border rounded-md bg-surface">
      <button
        type="button"
        onClick={() => setOpen((o) => !o)}
        className="w-full flex items-center justify-between gap-3 px-4 py-3 text-left hover:bg-surface-hi"
      >
        <div className="min-w-0 flex-1">
          <div className="text-text-1 text-base font-semibold truncate">
            {project.display_name}
          </div>
          <div className="font-mono text-[11px] text-text-3 truncate">
            {formatProjectPath(project.path)}
          </div>
        </div>
        <div className="text-text-2 text-sm whitespace-nowrap">
          {project.session_count} sessions · {formatTimeAgo(project.last_activity)}
        </div>
        <button
          type="button"
          onClick={(e) => {
            e.stopPropagation();
            api.openSession(project.path).catch(console.error);
          }}
          className="px-3 py-1 text-sm rounded-md border border-border hover:bg-surface-hi"
        >
          New session
        </button>
        <KebabMenu onHide={() => onHide(project)} />
      </button>

      {open && (
        <div>
          {project.sessions.map((s) => (
            <SessionRow
              key={s.id}
              session={s}
              cwd={project.path}
              projectUsed1m={project.used_1m_recently}
              onRefresh={onMutate}
            />
          ))}
          {project.worktrees.map((w) => (
            <div key={w.path}>
              <div className="px-4 py-1 bg-surface-hi font-mono text-[11px] text-text-3 border-t border-border">
                worktree · {formatProjectPath(w.path)}
              </div>
              {w.sessions.map((s) => (
                <SessionRow
                  key={s.id}
                  session={s}
                  cwd={w.path}
                  projectUsed1m={project.used_1m_recently}
                  onRefresh={onMutate}
                />
              ))}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
