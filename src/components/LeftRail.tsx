import type { Project, Session } from "../lib/types";
import { PinnedRow } from "./PinnedRow";
import { ProjectRow } from "./ProjectRow";

interface Props {
  projects: Project[];
  pinnedSessions: Session[];
  pinnedProjectPaths: Map<string, string>;
  selectedProjectPath: string | null;
  searchQuery: string;
  hiddenCount: number;
  onSearchChange: (q: string) => void;
  onSelectProject: (path: string) => void;
  onSelectSession: (projectPath: string, sessionId: string) => void;
  onHideProject: (project: Project) => void;
  onOpenHidden: () => void;
  onMutate: () => void;
}

function matchesQuery(s: Session, q: string): boolean {
  if (!q) return true;
  const needle = q.toLowerCase();
  const hay = [
    s.title ?? "",
    s.recent_excerpt ?? "",
    s.bg_name ?? "",
  ].join(" ").toLowerCase();
  return hay.includes(needle);
}

export function LeftRail({
  projects,
  pinnedSessions,
  pinnedProjectPaths,
  selectedProjectPath,
  searchQuery,
  hiddenCount,
  onSearchChange,
  onSelectProject,
  onSelectSession,
  onHideProject,
  onOpenHidden,
  onMutate,
}: Props) {
  const visibleProjects = projects.filter((p) => !p.hidden);

  return (
    <aside className="w-60 shrink-0 bg-surface border-r border-border flex flex-col">
      <div className="p-4">
        <input
          type="search"
          value={searchQuery}
          onChange={(e) => onSearchChange(e.target.value)}
          placeholder="Search sessions..."
          className="w-full h-8 px-2 rounded-sm bg-bg border border-border text-text-1 text-[13px] placeholder:text-text-3 focus:outline-none focus:border-accent"
        />
      </div>

      <div className="flex-1 overflow-y-auto px-2 pb-2 flex flex-col gap-4">
        {pinnedSessions.length > 0 && (
          <section aria-label="PINNED">
            <h3 className="text-[11px] font-semibold uppercase tracking-[0.08em] text-text-3 px-2 mb-1">
              PINNED
            </h3>
            <div className="flex flex-col">
              {pinnedSessions.map((s) => (
                <PinnedRow
                  key={s.id}
                  session={s}
                  projectPath={pinnedProjectPaths.get(s.id) ?? ""}
                  dimmed={!matchesQuery(s, searchQuery)}
                  onClick={onSelectSession}
                  onMutate={onMutate}
                />
              ))}
            </div>
          </section>
        )}

        <section aria-label="PROJECTS">
          <h3 className="text-[11px] font-semibold uppercase tracking-[0.08em] text-text-3 px-2 mb-1">
            PROJECTS
          </h3>
          <div className="flex flex-col">
            {visibleProjects.map((p) => (
              <ProjectRow
                key={p.path}
                project={p}
                selected={p.path === selectedProjectPath}
                onSelect={onSelectProject}
                onHide={onHideProject}
              />
            ))}
          </div>
        </section>
      </div>

      {hiddenCount > 0 && (
        <div className="border-t border-border p-3">
          <button
            type="button"
            onClick={onOpenHidden}
            className="text-[12px] text-text-3 hover:text-text-1"
          >
            Manage hidden ({hiddenCount})
          </button>
        </div>
      )}
    </aside>
  );
}
