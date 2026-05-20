import { describe, expect, it, vi, afterEach } from "vitest";
import { render, screen, fireEvent, cleanup } from "@testing-library/react";
import type { Project, Session } from "../lib/types";

vi.mock("../lib/api", () => ({ api: {} }));

import { ProjectRow } from "./ProjectRow";

function makeSession(over: Partial<Session> = {}): Session {
  return {
    id: "x", jsonl_path: "/tmp/x.jsonl", cwd: null,
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
    path: "/repo",
    display_name: "repo",
    session_count: 1,
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

describe("ProjectRow", () => {
  it("renders display_name", () => {
    render(
      <ProjectRow
        project={makeProject({ display_name: "groxsen" })}
        selected={false}
        onSelect={() => {}}
        onHide={() => {}}
      />,
    );
    expect(screen.getByText("groxsen")).toBeInTheDocument();
  });

  it("shows the live-dot when any session is live", () => {
    render(
      <ProjectRow
        project={makeProject({
          sessions: [makeSession({ live_status: "busy" })],
        })}
        selected={false}
        onSelect={() => {}}
        onHide={() => {}}
      />,
    );
    expect(screen.getByTestId("project-row-live-dot")).toBeInTheDocument();
  });

  it("does NOT show the live-dot when no session is live", () => {
    render(
      <ProjectRow
        project={makeProject()}
        selected={false}
        onSelect={() => {}}
        onHide={() => {}}
      />,
    );
    expect(screen.queryByTestId("project-row-live-dot")).toBeNull();
  });

  it("applies the selected indicator when selected=true", () => {
    const { container } = render(
      <ProjectRow
        project={makeProject()}
        selected={true}
        onSelect={() => {}}
        onHide={() => {}}
      />,
    );
    expect(container.querySelector("[data-selected='true']")).not.toBeNull();
  });

  it("clicking the row calls onSelect with project.path", () => {
    const onSelect = vi.fn();
    render(
      <ProjectRow
        project={makeProject({ path: "/x" })}
        selected={false}
        onSelect={onSelect}
        onHide={() => {}}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: /^repo$/i }));
    expect(onSelect).toHaveBeenCalledWith("/x");
  });

  it("kebab → Hide calls onHide with the project", () => {
    const onHide = vi.fn();
    const project = makeProject({ path: "/y" });
    render(
      <ProjectRow
        project={project}
        selected={false}
        onSelect={() => {}}
        onHide={onHide}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: /more actions/i }));
    fireEvent.click(screen.getByRole("menuitem", { name: /hide project/i }));
    expect(onHide).toHaveBeenCalledWith(project);
  });
});
