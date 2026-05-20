import { describe, expect, it, vi, afterEach, beforeEach } from "vitest";
import {
  render,
  screen,
  fireEvent,
  cleanup,
  waitFor,
  within,
} from "@testing-library/react";
import type { Project, Stats, Prefs } from "../lib/types";

// Mock the api module BEFORE importing AppShell so the polling hook picks it up.
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
  },
}));

import { AppShell } from "./AppShell";
import { api } from "../lib/api";

const STATS: Stats = {
  project_count: 1,
  session_count: 0,
  tokens_7d: 0,
  tokens_all_time: 0,
};
const PREFS: Prefs = { hidden_projects: [], noise_threshold: 0 };

function makeProject(over: Partial<Project> = {}): Project {
  return {
    path: "/Users/x/code/claude-hub",
    display_name: "claude-hub",
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

beforeEach(() => {
  vi.mocked(api.listProjects).mockResolvedValue([makeProject()]);
  vi.mocked(api.getStats).mockResolvedValue(STATS);
  vi.mocked(api.getPrefs).mockResolvedValue(PREFS);
  vi.mocked(api.hideProject).mockResolvedValue(undefined);
  vi.mocked(api.unhideProject).mockResolvedValue(undefined);
});

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("AppShell — hide/undo wiring", () => {
  it("does NOT render the Hidden manager when no projects are hidden", async () => {
    render(<AppShell />);
    await waitFor(() =>
      expect(screen.getByText("claude-hub")).toBeInTheDocument(),
    );
    expect(screen.queryByRole("button", { name: /^Hidden \(/ })).toBeNull();
  });

  it("DOES render the Hidden manager when at least one project is hidden", async () => {
    vi.mocked(api.listProjects).mockResolvedValue([
      makeProject({ path: "/a", display_name: "a", hidden: true }),
      makeProject({ path: "/b", display_name: "b", hidden: false }),
    ]);
    render(<AppShell />);
    await waitFor(() =>
      expect(screen.getByRole("button", { name: /^Hidden \(1\)$/ })).toBeInTheDocument(),
    );
  });

  it("shows the Undo toast after hiding a project", async () => {
    render(<AppShell />);
    await waitFor(() =>
      expect(screen.getByText("claude-hub")).toBeInTheDocument(),
    );
    fireEvent.click(screen.getByRole("button", { name: /more actions/i }));
    fireEvent.click(screen.getByRole("menuitem", { name: /hide project/i }));
    await waitFor(() => expect(api.hideProject).toHaveBeenCalled());
    expect(await screen.findByRole("status")).toHaveTextContent("claude-hub");
  });

  it("calls api.unhideProject when the toast's Undo is clicked", async () => {
    render(<AppShell />);
    await waitFor(() =>
      expect(screen.getByText("claude-hub")).toBeInTheDocument(),
    );
    fireEvent.click(screen.getByRole("button", { name: /more actions/i }));
    fireEvent.click(screen.getByRole("menuitem", { name: /hide project/i }));
    const toast = await screen.findByRole("status");
    fireEvent.click(within(toast).getByRole("button", { name: /undo/i }));
    await waitFor(() =>
      expect(api.unhideProject).toHaveBeenCalledWith("/Users/x/code/claude-hub"),
    );
  });
});
