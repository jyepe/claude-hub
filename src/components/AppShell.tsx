import { useCallback } from "react";
import { api } from "../lib/api";
import { usePoll } from "../lib/usePoll";
import type { Project, Stats } from "../lib/types";
import { HeaderStats } from "./HeaderStats";
import { ProjectCard } from "./ProjectCard";
import { RefreshButton } from "./RefreshButton";
import { HiddenProjectsManager } from "./HiddenProjectsManager";

export function AppShell() {
  const projectsFetcher = useCallback(() => api.listProjects(), []);
  const statsFetcher = useCallback(() => api.getStats(), []);
  const {
    data: projects,
    refresh: refreshProjects,
    lastRefresh,
    error: projectsError,
  } = usePoll<Project[]>(projectsFetcher);
  const { data: stats, refresh: refreshStats, error: statsError } =
    usePoll<Stats>(statsFetcher);

  const errorMessage =
    (projectsError as Error | null)?.toString() ??
    (statsError as Error | null)?.toString() ??
    null;

  const refreshAll = useCallback(() => {
    refreshProjects();
    refreshStats();
  }, [refreshProjects, refreshStats]);

  const visible = (projects ?? []).filter((p) => !p.hidden);

  return (
    <div className="min-h-screen flex flex-col gap-6 p-6 max-w-[1200px] mx-auto">
      <header className="flex items-end justify-between gap-4">
        <div>
          <h1 className="text-text-1 text-[22px] font-semibold tracking-tight">
            Claude Hub
          </h1>
          <p className="text-text-3 text-sm">
            Every Claude Code session on this machine.
          </p>
        </div>
        <div className="flex items-center gap-2">
          <HiddenProjectsManager onChange={refreshAll} />
          <RefreshButton onRefresh={refreshAll} lastRefresh={lastRefresh} />
        </div>
      </header>

      {errorMessage && (
        <div className="px-4 py-2 border border-danger rounded-md bg-surface text-danger text-sm">
          {errorMessage}
        </div>
      )}
      <HeaderStats stats={stats} />

      <main className="flex flex-col gap-3">
        {visible.length === 0 && (
          <div className="text-text-3 text-sm py-12 text-center border border-dashed border-border rounded-md">
            No projects to show. (Drop the noise threshold from "Hidden" if you've hidden everything.)
          </div>
        )}
        {visible.map((p) => (
          <ProjectCard key={p.path} project={p} onMutate={refreshAll} />
        ))}
      </main>
    </div>
  );
}
