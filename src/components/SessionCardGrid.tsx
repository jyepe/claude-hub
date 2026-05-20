import { useMemo } from "react";
import type { Project, Session } from "../lib/types";
import { SessionCard } from "./SessionCard";

interface Props {
  project: Project;
  pinnedIds: Set<string>;
  searchQuery: string;
  focusedSessionId: string | null;
  onMutate: () => void;
}

interface FlatItem {
  session: Session;
  cwd: string;
  worktreeLeaf: string | null;
}

function leafName(path: string): string {
  const cleaned = path.replace(/[\\/]+$/, "");
  const idx = Math.max(cleaned.lastIndexOf("/"), cleaned.lastIndexOf("\\"));
  return idx >= 0 ? cleaned.slice(idx + 1) : cleaned;
}

function sortKey(s: Session): number {
  if (s.live_status !== null) return 0;
  if (s.is_bg_agent) return 1;
  return 2;
}

function activityTs(s: Session): number {
  if (!s.last_activity) return 0;
  const t = Date.parse(s.last_activity);
  return Number.isFinite(t) ? t : 0;
}

function flatten(project: Project): FlatItem[] {
  const items: FlatItem[] = [];
  for (const s of project.sessions) {
    items.push({ session: s, cwd: project.path, worktreeLeaf: null });
  }
  for (const w of project.worktrees) {
    const leaf = leafName(w.path);
    for (const s of w.sessions) {
      items.push({ session: s, cwd: w.path, worktreeLeaf: leaf });
    }
  }
  items.sort((a, b) => {
    const k = sortKey(a.session) - sortKey(b.session);
    if (k !== 0) return k;
    return activityTs(b.session) - activityTs(a.session);
  });
  return items;
}

function matches(s: Session, q: string): boolean {
  if (!q) return true;
  const needle = q.toLowerCase();
  const haystack = [
    s.title ?? "",
    s.recent_excerpt ?? "",
    s.bg_name ?? "",
    s.bg_intent ?? "",
    s.bg_detail ?? "",
  ].join(" ").toLowerCase();
  return haystack.includes(needle);
}

export function SessionCardGrid({
  project,
  pinnedIds,
  searchQuery,
  focusedSessionId,
  onMutate,
}: Props) {
  const items = useMemo(() => flatten(project), [project]);
  const visible = useMemo(
    () => items.filter((it) => matches(it.session, searchQuery)),
    [items, searchQuery],
  );

  if (items.length === 0) {
    return (
      <div className="text-text-3 text-sm py-12 text-center border border-dashed border-border rounded-md">
        No sessions yet. Hit + New session to start one.
      </div>
    );
  }

  if (visible.length === 0) {
    return (
      <div className="text-text-3 text-sm py-12 text-center border border-dashed border-border rounded-md">
        No sessions match "{searchQuery}".
      </div>
    );
  }

  return (
    <div
      className="grid gap-4"
      style={{ gridTemplateColumns: "repeat(auto-fill, minmax(280px, 1fr))" }}
    >
      {visible.map(({ session, cwd, worktreeLeaf }) => (
        <SessionCard
          key={session.id}
          session={session}
          cwd={cwd}
          projectUsed1m={project.used_1m_recently}
          isPinned={pinnedIds.has(session.id)}
          focused={focusedSessionId === session.id}
          worktreeLeaf={worktreeLeaf}
          onMutate={onMutate}
        />
      ))}
    </div>
  );
}
