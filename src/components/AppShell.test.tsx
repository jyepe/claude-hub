import { describe, expect, it, vi, afterEach, beforeEach } from "vitest";
import {
  render,
  screen,
  fireEvent,
  cleanup,
  waitFor,
  within,
} from "@testing-library/react";
import type { Project, Prefs, Session, Stats } from "../lib/types";

vi.mock("../lib/api", () => ({
  api: {
    listProjects: vi.fn(),
    getStats: vi.fn(),
    getPrefs: vi.fn(),
    setPrefs: vi.fn(),
    hideProject: vi.fn(),
    unhideProject: vi.fn(),
    openSession: vi.fn(),
    attachAgent: vi.fn(),
    closeSession: vi.fn(),
    pinSession: vi.fn(),
    unpinSession: vi.fn(),
  },
}));

import { AppShell } from "./AppShell";
import { api } from "../lib/api";

const STATS: Stats = {
  project_count: 0, session_count: 0, tokens_7d: 0, tokens_all_time: 0,
};
const PREFS: Prefs = { hidden_projects: [], noise_threshold: 0, pinned_session_ids: [] };

function makeSession(over: Partial<Session> = {}): Session {
  return {
    id: "s1", jsonl_path: "/tmp/s1.jsonl", cwd: "/Users/x/code/claude-hub",
    title: "default", model: null,
    message_count: 0, tokens: 0, context_tokens: 0, max_prompt_tokens: 0,
    last_activity: "2026-05-20T13:00:00Z",
    live_context_window: null, live_model_id: null,
    is_bg_agent: false, live_status: null,
    bg_state: null, bg_detail: null, bg_tempo: null, bg_intent: null, bg_name: null,
    recent_excerpt: null,
    ...over,
  };
}

function makeProject(over: Partial<Project> = {}): Project {
  return {
    path: "/Users/x/code/claude-hub",
    display_name: "claude-hub",
    session_count: 1,
    total_tokens: 0,
    last_activity: "2026-05-20T13:00:00Z",
    sessions: [makeSession()],
    worktrees: [],
    hidden: false,
    used_1m_recently: false,
    ...over,
  };
}

beforeEach(() => {
  vi.mocked(api.listProjects).mockResolvedValue([makeProject()]);
  vi.mocked(api.getStats).mockResolvedValue(STATS);
  vi.mocked(api.getPrefs).mockResolvedValue(PREFS);
  vi.mocked(api.hideProject).mockResolvedValue(undefined);
  vi.mocked(api.unhideProject).mockResolvedValue(undefined);
  vi.mocked(api.pinSession).mockResolvedValue(undefined);
  vi.mocked(api.unpinSession).mockResolvedValue(undefined);
});

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("AppShell — workbench layout", () => {
  it("auto-selects the most-recently-active project and shows its sessions", async () => {
    vi.mocked(api.listProjects).mockResolvedValue([
      makeProject({
        path: "/p1", display_name: "p1",
        last_activity: "2026-05-01T00:00:00Z",
        sessions: [makeSession({ id: "s-p1", title: "p1 session", last_activity: "2026-05-01T00:00:00Z" })],
      }),
      makeProject({
        path: "/p2", display_name: "p2",
        last_activity: "2026-05-20T13:00:00Z",
        sessions: [makeSession({ id: "s-p2", title: "p2 session", last_activity: "2026-05-20T13:00:00Z" })],
      }),
    ]);
    render(<AppShell />);
    await waitFor(() => expect(screen.getByText("p2 session")).toBeInTheDocument());
    expect(screen.queryByText("p1 session")).toBeNull();
  });

  it("clicking a different project in the rail switches the main pane", async () => {
    vi.mocked(api.listProjects).mockResolvedValue([
      makeProject({ path: "/p1", display_name: "p1", last_activity: "2026-05-20T13:00:00Z",
        sessions: [makeSession({ id: "s-p1", title: "p1 session" })] }),
      makeProject({ path: "/p2", display_name: "p2", last_activity: "2026-05-01T00:00:00Z",
        sessions: [makeSession({ id: "s-p2", title: "p2 session", last_activity: "2026-05-01T00:00:00Z" })] }),
    ]);
    render(<AppShell />);
    await waitFor(() => expect(screen.getByText("p1 session")).toBeInTheDocument());
    fireEvent.click(screen.getByRole("button", { name: /^p2$/i }));
    expect(screen.getByText("p2 session")).toBeInTheDocument();
    expect(screen.queryByText("p1 session")).toBeNull();
  });

  it("hiding the currently-selected project falls back to most-recent", async () => {
    vi.mocked(api.listProjects)
      .mockResolvedValueOnce([
        makeProject({ path: "/p1", display_name: "p1", last_activity: "2026-05-20T13:00:00Z",
          sessions: [makeSession({ id: "s-p1", title: "p1 session" })] }),
        makeProject({ path: "/p2", display_name: "p2", last_activity: "2026-05-01T00:00:00Z",
          sessions: [makeSession({ id: "s-p2", title: "p2 session", last_activity: "2026-05-01T00:00:00Z" })] }),
      ])
      .mockResolvedValue([
        makeProject({ path: "/p2", display_name: "p2", last_activity: "2026-05-01T00:00:00Z",
          sessions: [makeSession({ id: "s-p2", title: "p2 session", last_activity: "2026-05-01T00:00:00Z" })] }),
      ]);
    render(<AppShell />);
    await waitFor(() => expect(screen.getByText("p1 session")).toBeInTheDocument());
    fireEvent.click(within(screen.getByRole("button", { name: /^p1$/i }).closest("[data-selected]") as HTMLElement)
      .getByRole("button", { name: /more actions/i }));
    fireEvent.click(screen.getByRole("menuitem", { name: /hide project/i }));
    await waitFor(() => expect(api.hideProject).toHaveBeenCalledWith("/p1"));
    await waitFor(() => expect(screen.getByText("p2 session")).toBeInTheDocument());
  });

  it("the undo toast still appears and calls unhideProject", async () => {
    vi.mocked(api.listProjects).mockResolvedValue([makeProject()]);
    render(<AppShell />);
    await waitFor(() =>
      expect(screen.getByRole("button", { name: /^claude-hub$/i })).toBeInTheDocument(),
    );
    const railRow = screen
      .getByRole("button", { name: /^claude-hub$/i })
      .closest("[data-selected]") as HTMLElement;
    fireEvent.click(within(railRow).getByRole("button", { name: /more actions/i }));
    fireEvent.click(screen.getByRole("menuitem", { name: /hide project/i }));
    const toast = await screen.findByRole("status");
    fireEvent.click(within(toast).getByRole("button", { name: /undo/i }));
    await waitFor(() => expect(api.unhideProject).toHaveBeenCalledWith("/Users/x/code/claude-hub"));
  });

  it("typing in the rail search filters cards in the main pane", async () => {
    vi.mocked(api.listProjects).mockResolvedValue([
      makeProject({
        path: "/repo", display_name: "repo",
        sessions: [
          makeSession({ id: "a", title: "alpha" }),
          makeSession({ id: "b", title: "beta" }),
        ],
      }),
    ]);
    render(<AppShell />);
    await waitFor(() => expect(screen.getByText("alpha")).toBeInTheDocument());
    fireEvent.change(screen.getByPlaceholderText(/search sessions/i), {
      target: { value: "beta" },
    });
    await waitFor(() => {
      expect(screen.queryByText("alpha")).toBeNull();
      expect(screen.getByText("beta")).toBeInTheDocument();
    });
  });

  it("pinned sessions appear in the PINNED rail section", async () => {
    vi.mocked(api.listProjects).mockResolvedValue([
      makeProject({
        path: "/repo", display_name: "repo",
        sessions: [makeSession({ id: "pinned-1", title: "alpha pinned" })],
      }),
    ]);
    vi.mocked(api.getPrefs).mockResolvedValue({ ...PREFS, pinned_session_ids: ["pinned-1"] });
    render(<AppShell />);
    await waitFor(() => {
      const pinnedSection = screen.getByRole("region", { name: /PINNED/i });
      expect(within(pinnedSection).getByText("alpha pinned")).toBeInTheDocument();
    });
  });
});
