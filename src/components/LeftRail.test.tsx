import { describe, expect, it, vi, afterEach } from "vitest";
import { render, screen, fireEvent, cleanup } from "@testing-library/react";
import type { Project, Session } from "../lib/types";

vi.mock("../lib/api", () => ({
  api: { unpinSession: vi.fn() },
}));

import { LeftRail } from "./LeftRail";

function makeSession(over: Partial<Session> = {}): Session {
  return {
    id: "x", jsonl_path: "/tmp/x.jsonl", cwd: "/repo",
    title: null, model: null,
    message_count: 0, tokens: 0, context_tokens: 0, max_prompt_tokens: 0,
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
    path: "/repo", display_name: "repo",
    session_count: 0, total_tokens: 0, last_activity: null,
    sessions: [], worktrees: [], hidden: false, used_1m_recently: false,
    ...over,
  };
}

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("LeftRail", () => {
  it("renders PROJECTS section with visible projects", () => {
    render(
      <LeftRail
        projects={[makeProject({ path: "/a", display_name: "alpha" }), makeProject({ path: "/b", display_name: "beta" })]}
        pinnedSessions={[]}
        pinnedProjectPaths={new Map()}
        selectedProjectPath={null}
        searchQuery=""
        hiddenCount={0}
        onSearchChange={() => {}}
        onSelectProject={() => {}}
        onSelectSession={() => {}}
        onHideProject={() => {}}
        onOpenHidden={() => {}}
        onMutate={() => {}}
      />,
    );
    expect(screen.getByText("alpha")).toBeInTheDocument();
    expect(screen.getByText("beta")).toBeInTheDocument();
  });

  it("hides PINNED section when no pinned sessions", () => {
    render(
      <LeftRail
        projects={[]}
        pinnedSessions={[]}
        pinnedProjectPaths={new Map()}
        selectedProjectPath={null}
        searchQuery=""
        hiddenCount={0}
        onSearchChange={() => {}}
        onSelectProject={() => {}}
        onSelectSession={() => {}}
        onHideProject={() => {}}
        onOpenHidden={() => {}}
        onMutate={() => {}}
      />,
    );
    expect(screen.queryByText(/PINNED/i)).toBeNull();
  });

  it("renders PINNED section with given sessions", () => {
    render(
      <LeftRail
        projects={[]}
        pinnedSessions={[makeSession({ id: "s1", title: "alpha" })]}
        pinnedProjectPaths={new Map([["s1", "/repo"]])}
        selectedProjectPath={null}
        searchQuery=""
        hiddenCount={0}
        onSearchChange={() => {}}
        onSelectProject={() => {}}
        onSelectSession={() => {}}
        onHideProject={() => {}}
        onOpenHidden={() => {}}
        onMutate={() => {}}
      />,
    );
    expect(screen.getByText(/PINNED/i)).toBeInTheDocument();
    expect(screen.getByText("alpha")).toBeInTheDocument();
  });

  it("hides 'Manage hidden' link when hiddenCount is 0", () => {
    render(
      <LeftRail
        projects={[]}
        pinnedSessions={[]}
        pinnedProjectPaths={new Map()}
        selectedProjectPath={null}
        searchQuery=""
        hiddenCount={0}
        onSearchChange={() => {}}
        onSelectProject={() => {}}
        onSelectSession={() => {}}
        onHideProject={() => {}}
        onOpenHidden={() => {}}
        onMutate={() => {}}
      />,
    );
    expect(screen.queryByText(/Manage hidden/i)).toBeNull();
  });

  it("shows 'Manage hidden (N)' when hiddenCount > 0", () => {
    const onOpen = vi.fn();
    render(
      <LeftRail
        projects={[]}
        pinnedSessions={[]}
        pinnedProjectPaths={new Map()}
        selectedProjectPath={null}
        searchQuery=""
        hiddenCount={3}
        onSearchChange={() => {}}
        onSelectProject={() => {}}
        onSelectSession={() => {}}
        onHideProject={() => {}}
        onOpenHidden={onOpen}
        onMutate={() => {}}
      />,
    );
    const btn = screen.getByRole("button", { name: /manage hidden \(3\)/i });
    expect(btn).toBeInTheDocument();
    fireEvent.click(btn);
    expect(onOpen).toHaveBeenCalled();
  });

  it("typing in the search input fires onSearchChange", () => {
    const onChange = vi.fn();
    render(
      <LeftRail
        projects={[]}
        pinnedSessions={[]}
        pinnedProjectPaths={new Map()}
        selectedProjectPath={null}
        searchQuery=""
        hiddenCount={0}
        onSearchChange={onChange}
        onSelectProject={() => {}}
        onSelectSession={() => {}}
        onHideProject={() => {}}
        onOpenHidden={() => {}}
        onMutate={() => {}}
      />,
    );
    const input = screen.getByPlaceholderText(/search sessions/i);
    fireEvent.change(input, { target: { value: "dark" } });
    expect(onChange).toHaveBeenCalledWith("dark");
  });
});
