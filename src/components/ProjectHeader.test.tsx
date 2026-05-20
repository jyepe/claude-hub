import { describe, expect, it, vi, afterEach, beforeEach } from "vitest";
import { render, screen, fireEvent, cleanup } from "@testing-library/react";
import type { Project, Session } from "../lib/types";

vi.mock("../lib/api", () => ({
  api: {
    openSession: vi.fn(),
  },
}));

import { ProjectHeader } from "./ProjectHeader";
import { api } from "../lib/api";

function makeSession(over: Partial<Session> = {}): Session {
  return {
    id: "x", jsonl_path: "/tmp/x.jsonl", cwd: null,
    title: null, model: null, message_count: 0,
    tokens: 0, context_tokens: 0, max_prompt_tokens: 0,
    last_activity: null,
    live_context_window: null, live_model_id: null,
    is_bg_agent: false, live_status: null,
    bg_state: null, bg_detail: null, bg_tempo: null, bg_intent: null, bg_name: null,
    recent_excerpt: null,
    ...over,
  };
}

function makeProject(over: Partial<Project> = {}): Project {
  return {
    path: "/Users/x/wrk/claude-hub",
    display_name: "claude-hub",
    session_count: 6,
    total_tokens: 0,
    last_activity: "2026-05-20T13:00:00Z",
    sessions: [],
    worktrees: [],
    hidden: false,
    used_1m_recently: false,
    ...over,
  };
}

beforeEach(() => {
  vi.mocked(api.openSession).mockResolvedValue(undefined);
});

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("ProjectHeader", () => {
  it("renders name, path, session count, active-today and time-ago", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-05-20T14:00:00Z"));
    const project = makeProject({
      sessions: [
        makeSession({ id: "a", last_activity: "2026-05-20T13:00:00Z" }), // in
        makeSession({ id: "b", last_activity: "2026-05-19T12:00:00Z" }), // out (>24h)
      ],
    });
    render(<ProjectHeader project={project} />);
    expect(screen.getByText("claude-hub")).toBeInTheDocument();
    expect(screen.getByText(/6 sessions/)).toBeInTheDocument();
    expect(screen.getByText(/1 active today/)).toBeInTheDocument();
    vi.useRealTimers();
  });

  it("New session button calls api.openSession with project.path and no session id", () => {
    const project = makeProject();
    render(<ProjectHeader project={project} />);
    fireEvent.click(screen.getByRole("button", { name: /new session/i }));
    expect(api.openSession).toHaveBeenCalledWith("/Users/x/wrk/claude-hub");
  });
});
