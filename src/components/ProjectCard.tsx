import { useState } from "react";
import type { Project } from "../lib/types";
import { SessionRow } from "./SessionRow";
import { api } from "../lib/api";
import { formatTimeAgo, formatProjectPath } from "../lib/format";

interface Props {
  project: Project;
  onMutate: () => void;
}

export function ProjectCard({ project, onMutate }: Props) {
  const [open, setOpen] = useState(false);

  const handleHide = async (e: React.MouseEvent) => {
    e.preventDefault();
    await api.hideProject(project.path);
    onMutate();
  };

  return (
    <div className="border border-border rounded-md bg-surface" onContextMenu={handleHide}>
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
      </button>

      {open && (
        <div>
          {project.sessions.map((s) => (
            <SessionRow key={s.id} session={s} cwd={project.path} projectUsed1m={project.used_1m_recently} />
          ))}
          {project.worktrees.map((w) => (
            <div key={w.path}>
              <div className="px-4 py-1 bg-surface-hi font-mono text-[11px] text-text-3 border-t border-border">
                worktree · {formatProjectPath(w.path)}
              </div>
              {w.sessions.map((s) => (
                <SessionRow key={s.id} session={s} cwd={w.path} projectUsed1m={project.used_1m_recently} />
              ))}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
