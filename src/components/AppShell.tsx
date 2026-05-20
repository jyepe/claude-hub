import { useCallback, useEffect, useMemo, useState } from "react";
import { api } from "../lib/api";
import { usePoll } from "../lib/usePoll";
import type { Prefs, Project, Session } from "../lib/types";
import { TitleBar } from "./TitleBar";
import { LeftRail } from "./LeftRail";
import { ProjectHeader } from "./ProjectHeader";
import { SessionCardGrid } from "./SessionCardGrid";
import { HiddenProjectsManager } from "./HiddenProjectsManager";
import { UndoToast } from "./UndoToast";

function flattenSessions(p: Project): Session[] {
  return [...p.sessions, ...p.worktrees.flatMap((w) => w.sessions)];
}

function flattenAll(projects: Project[]): Session[] {
  return projects.flatMap(flattenSessions);
}

function buildSessionProjectMap(projects: Project[]): Map<string, string> {
  const map = new Map<string, string>();
  for (const p of projects) {
    for (const s of p.sessions) map.set(s.id, p.path);
    for (const w of p.worktrees) for (const s of w.sessions) map.set(s.id, p.path);
  }
  return map;
}

function pickMostRecent(projects: Project[]): string | null {
  const visible = projects.filter((p) => !p.hidden);
  if (visible.length === 0) return null;
  let best = visible[0];
  let bestTs = Date.parse(best.last_activity ?? "") || 0;
  for (const p of visible.slice(1)) {
    const ts = Date.parse(p.last_activity ?? "") || 0;
    if (ts > bestTs) { best = p; bestTs = ts; }
  }
  return best.path;
}

export function AppShell() {
  const projectsFetcher = useCallback(() => api.listProjects(), []);
  const statsFetcher = useCallback(() => api.getStats(), []);
  const prefsFetcher = useCallback(() => api.getPrefs(), []);
  const {
    data: projects,
    refresh: refreshProjects,
    lastRefresh,
    error: projectsError,
  } = usePoll<Project[]>(projectsFetcher);
  const { refresh: refreshStats, error: statsError } = usePoll(statsFetcher);
  const { data: prefs, refresh: refreshPrefs } = usePoll<Prefs>(prefsFetcher);

  const [selectedProjectPath, setSelectedProjectPath] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [focusedSessionId, setFocusedSessionId] = useState<string | null>(null);
  const [hiddenOpen, setHiddenOpen] = useState(false);
  const [pendingUndo, setPendingUndo] = useState<{ path: string; name: string } | null>(null);

  const all = projects ?? [];
  const visible = all.filter((p) => !p.hidden);
  const hiddenCount = all.length - visible.length;
  const allSessions = useMemo(() => flattenAll(visible), [visible]);
  const sessionToProject = useMemo(() => buildSessionProjectMap(visible), [visible]);

  useEffect(() => {
    if (!projects) return;
    const stillValid = selectedProjectPath
      && visible.some((p) => p.path === selectedProjectPath);
    if (!stillValid) {
      setSelectedProjectPath(pickMostRecent(visible));
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [projects]);

  useEffect(() => {
    if (!focusedSessionId) return;
    const t = setTimeout(() => setFocusedSessionId(null), 1500);
    return () => clearTimeout(t);
  }, [focusedSessionId]);

  useEffect(() => {
    if (!focusedSessionId) return;
    const el = document.querySelector(
      `[data-testid="session-card"][data-session-id="${focusedSessionId}"]`,
    );
    if (el && "scrollIntoView" in el) {
      (el as HTMLElement).scrollIntoView({ block: "center", behavior: "smooth" });
    }
  }, [focusedSessionId]);

  const pinnedIdsArr = prefs?.pinned_session_ids ?? [];
  const pinnedIdsSet = useMemo(() => new Set(pinnedIdsArr), [pinnedIdsArr]);
  const pinnedSessions = useMemo(() => {
    const byId = new Map<string, Session>();
    for (const s of allSessions) byId.set(s.id, s);
    return pinnedIdsArr.map((id) => byId.get(id)).filter((s): s is Session => !!s);
  }, [allSessions, pinnedIdsArr]);

  const selectedProject = useMemo(
    () => visible.find((p) => p.path === selectedProjectPath) ?? null,
    [visible, selectedProjectPath],
  );

  const refreshAll = useCallback(() => {
    refreshProjects();
    refreshStats();
    refreshPrefs();
  }, [refreshProjects, refreshStats, refreshPrefs]);

  const handleHide = useCallback(async (project: Project) => {
    try {
      await api.hideProject(project.path);
      setPendingUndo({ path: project.path, name: project.display_name });
      refreshAll();
    } catch (err) {
      console.error("hide_project failed", err);
    }
  }, [refreshAll]);

  const handleUndo = useCallback(async () => {
    if (!pendingUndo) return;
    try {
      await api.unhideProject(pendingUndo.path);
    } catch (err) {
      console.error("unhide_project failed", err);
    } finally {
      setPendingUndo(null);
      refreshAll();
    }
  }, [pendingUndo, refreshAll]);

  const handleDismissUndo = useCallback(() => setPendingUndo(null), []);

  const handleSelectSession = useCallback((projectPath: string, sessionId: string) => {
    setSelectedProjectPath(projectPath);
    setFocusedSessionId(sessionId);
    setSearchQuery("");
  }, []);

  const errorMessage =
    (projectsError as Error | null)?.toString() ??
    (statsError as Error | null)?.toString() ??
    null;

  return (
    <div className="h-screen flex flex-col bg-bg">
      <TitleBar
        allSessions={allSessions}
        onRefresh={refreshAll}
        lastRefresh={lastRefresh}
      />

      {errorMessage && (
        <div className="px-4 py-2 border-b border-danger bg-surface text-danger text-sm">
          {errorMessage}
        </div>
      )}

      <div className="flex-1 flex min-h-0">
        <LeftRail
          projects={all}
          pinnedSessions={pinnedSessions}
          pinnedProjectPaths={sessionToProject}
          selectedProjectPath={selectedProjectPath}
          searchQuery={searchQuery}
          hiddenCount={hiddenCount}
          onSearchChange={setSearchQuery}
          onSelectProject={setSelectedProjectPath}
          onSelectSession={handleSelectSession}
          onHideProject={handleHide}
          onOpenHidden={() => setHiddenOpen(true)}
          onMutate={refreshAll}
        />

        <main className="flex-1 overflow-y-auto px-4 pb-6">
          {visible.length === 0 ? (
            <div className="mt-12 text-text-3 text-sm py-12 text-center border border-dashed border-border rounded-md">
              No projects to show.
            </div>
          ) : selectedProject ? (
            <>
              <ProjectHeader project={selectedProject} />
              <SessionCardGrid
                project={selectedProject}
                pinnedIds={pinnedIdsSet}
                searchQuery={searchQuery}
                focusedSessionId={focusedSessionId}
                onMutate={refreshAll}
              />
            </>
          ) : null}
        </main>
      </div>

      <HiddenProjectsManager
        count={hiddenCount}
        open={hiddenOpen}
        onClose={() => setHiddenOpen(false)}
        onChange={refreshAll}
      />

      {pendingUndo && (
        <UndoToast
          project={pendingUndo}
          onUndo={handleUndo}
          onDismiss={handleDismissUndo}
        />
      )}
    </div>
  );
}
