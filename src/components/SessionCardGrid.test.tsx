import { describe, expect, it, vi, afterEach } from "vitest";
import { render, screen, cleanup } from "@testing-library/react";
import type { Project, Session } from "../lib/types";

vi.mock("../lib/api", () => ({
  api: {
    openSession: vi.fn(),
    attachAgent: vi.fn(),
    closeSession: vi.fn(),
    pinSession: vi.fn(),
    unpinSession: vi.fn(),
  },
}));

import { SessionCardGrid } from "./SessionCardGrid";

function makeSession(over: Partial<Session> = {}): Session {
  return {
    id: "x",
    jsonl_path: "/tmp/x.jsonl",
    cwd: null,
    title: "default title",
    model: null,
    message_count: 0,
    tokens: 0,
    context_tokens: 0,
    max_prompt_tokens: 0,
    last_activity: null,
    live_context_window: null,
    live_model_id: null,
    is_bg_agent: false,
    live_status: null,
    bg_state: null,
    bg_detail: null,
    bg_tempo: null,
    bg_intent: null,
    bg_name: null,
    recent_excerpt: null,
    ...over,
  };
}

function makeProject(over: Partial<Project> = {}): Project {
  return {
    path: "/repo",
    display_name: "repo",
    session_count: 0,
    total_tokens: 0,
    last_activity: null,
    sessions: [],
    worktrees: [],
    hidden: false,
    used_1m_recently: false,
    ...over,
  };
}

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("SessionCardGrid", () => {
  it("renders cards for every session in the project + worktrees", () => {
    const project = makeProject({
      sessions: [makeSession({ id: "a", title: "alpha" })],
      worktrees: [
        { path: "/repo/.wt/feature", sessions: [makeSession({ id: "b", title: "beta" })] },
      ],
    });
    render(
      <SessionCardGrid
        project={project}
        pinnedIds={new Set()}
        searchQuery=""
        focusedSessionId={null}
        onMutate={() => {}}
      />,
    );
    expect(screen.getByText("alpha")).toBeInTheDocument();
    expect(screen.getByText("beta")).toBeInTheDocument();
    expect(screen.getByText(/worktree: feature/)).toBeInTheDocument();
  });

  it("orders live > bg_agent > inactive (by last_activity desc)", () => {
    const project = makeProject({
      sessions: [
        makeSession({ id: "old", title: "old", last_activity: "2026-05-01T00:00:00Z" }),
        makeSession({ id: "live", title: "live", live_status: "busy" }),
        makeSession({ id: "bg", title: "bg", is_bg_agent: true, bg_state: "running" }),
        makeSession({ id: "recent", title: "recent", last_activity: "2026-05-19T00:00:00Z" }),
      ],
    });
    render(
      <SessionCardGrid
        project={project}
        pinnedIds={new Set()}
        searchQuery=""
        focusedSessionId={null}
        onMutate={() => {}}
      />,
    );
    const cards = screen.getAllByTestId("session-card");
    expect(cards.map((c) => c.getAttribute("data-session-id"))).toEqual([
      "live", "bg", "recent", "old",
    ]);
  });

  it("filters by case-insensitive substring on title + recent_excerpt", () => {
    const project = makeProject({
      sessions: [
        makeSession({ id: "a", title: "Refactor session loader" }),
        makeSession({ id: "b", title: "Wire dark-mode tokens", recent_excerpt: "two places where the panel" }),
        makeSession({ id: "c", title: "Audit kebab z-index" }),
      ],
    });
    render(
      <SessionCardGrid
        project={project}
        pinnedIds={new Set()}
        searchQuery="DARK"
        focusedSessionId={null}
        onMutate={() => {}}
      />,
    );
    expect(screen.getByText("Wire dark-mode tokens")).toBeInTheDocument();
    expect(screen.queryByText("Refactor session loader")).toBeNull();
    expect(screen.queryByText("Audit kebab z-index")).toBeNull();
  });

  it("matches recent_excerpt content for search", () => {
    const project = makeProject({
      sessions: [
        makeSession({ id: "a", title: "alpha", recent_excerpt: "uniquephrase here" }),
        makeSession({ id: "b", title: "beta", recent_excerpt: "nothing related" }),
      ],
    });
    render(
      <SessionCardGrid
        project={project}
        pinnedIds={new Set()}
        searchQuery="uniquephrase"
        focusedSessionId={null}
        onMutate={() => {}}
      />,
    );
    expect(screen.getByText("alpha")).toBeInTheDocument();
    expect(screen.queryByText("beta")).toBeNull();
  });

  it("shows the search-empty state when nothing matches", () => {
    const project = makeProject({
      sessions: [makeSession({ id: "a", title: "alpha" })],
    });
    render(
      <SessionCardGrid
        project={project}
        pinnedIds={new Set()}
        searchQuery="zzz-no-match"
        focusedSessionId={null}
        onMutate={() => {}}
      />,
    );
    expect(screen.getByText(/no sessions match/i)).toBeInTheDocument();
  });

  it("shows the empty-project state when project has zero sessions", () => {
    const project = makeProject({ sessions: [], worktrees: [] });
    render(
      <SessionCardGrid
        project={project}
        pinnedIds={new Set()}
        searchQuery=""
        focusedSessionId={null}
        onMutate={() => {}}
      />,
    );
    expect(screen.getByText(/no sessions yet/i)).toBeInTheDocument();
  });
});
