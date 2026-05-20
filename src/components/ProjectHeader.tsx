import { useMemo } from "react";
import type { Project } from "../lib/types";
import { api } from "../lib/api";
import { activeToday, formatTimeAgo, formatProjectPath } from "../lib/format";

interface Props {
  project: Project;
}

export function ProjectHeader({ project }: Props) {
  const activeCount = useMemo(() => {
    const flat = [
      ...project.sessions,
      ...project.worktrees.flatMap((w) => w.sessions),
    ];
    return activeToday(flat);
  }, [project]);

  return (
    <div className="sticky top-0 z-10 bg-bg pt-2 pb-4 mb-2 border-b border-border">
      <div className="flex items-end justify-between gap-4">
        <h2 className="text-text-1 text-[22px] font-semibold tracking-tight truncate">
          {project.display_name}
        </h2>
        <button
          type="button"
          onClick={() => api.openSession(project.path).catch(console.error)}
          className="px-3 py-2 text-sm rounded-md bg-accent hover:bg-accent-hover text-text-1 whitespace-nowrap"
        >
          + New session
        </button>
      </div>
      <div className="font-mono text-[11.5px] text-text-3 truncate mt-1">
        {formatProjectPath(project.path)} · {project.session_count} sessions
        {" · "}{activeCount} active today
        {" · "}{formatTimeAgo(project.last_activity)}
      </div>
    </div>
  );
}
