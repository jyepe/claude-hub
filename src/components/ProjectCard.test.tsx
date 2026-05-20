import { describe, expect, it, vi, afterEach } from "vitest";
import { render, screen, fireEvent, cleanup } from "@testing-library/react";
import { ProjectCard } from "./ProjectCard";
import type { Project } from "../lib/types";

afterEach(() => {
  cleanup();
});

const PROJECT: Project = {
  path: "/Users/x/code/claude-hub",
  display_name: "claude-hub",
  session_count: 3,
  total_tokens: 0,
  last_activity: null,
  sessions: [],
  worktrees: [],
  hidden: false,
  used_1m_recently: false,
};

describe("ProjectCard", () => {
  it("does NOT hide on right-click (regression guard for #4)", () => {
    const onHide = vi.fn();
    render(<ProjectCard project={PROJECT} onMutate={() => {}} onHide={onHide} />);
    const card = screen.getByTestId("project-card");
    fireEvent.contextMenu(card);
    expect(onHide).not.toHaveBeenCalled();
  });

  it("renders a kebab button with aria-label", () => {
    render(<ProjectCard project={PROJECT} onMutate={() => {}} onHide={() => {}} />);
    expect(screen.getByRole("button", { name: /more actions/i })).toBeInTheDocument();
  });

  it("opens the popover with 'Hide project' when the kebab is clicked", () => {
    render(<ProjectCard project={PROJECT} onMutate={() => {}} onHide={() => {}} />);
    expect(screen.queryByRole("menuitem", { name: /hide project/i })).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: /more actions/i }));
    expect(screen.getByRole("menuitem", { name: /hide project/i })).toBeInTheDocument();
  });

  it("calls onHide with the project when 'Hide project' is clicked", () => {
    const onHide = vi.fn();
    render(<ProjectCard project={PROJECT} onMutate={() => {}} onHide={onHide} />);
    fireEvent.click(screen.getByRole("button", { name: /more actions/i }));
    fireEvent.click(screen.getByRole("menuitem", { name: /hide project/i }));
    expect(onHide).toHaveBeenCalledTimes(1);
    expect(onHide).toHaveBeenCalledWith(PROJECT);
    expect(screen.queryByRole("menuitem", { name: /hide project/i })).toBeNull();
  });

  it("closes the popover when Escape is pressed", () => {
    render(<ProjectCard project={PROJECT} onMutate={() => {}} onHide={() => {}} />);
    fireEvent.click(screen.getByRole("button", { name: /more actions/i }));
    expect(screen.getByRole("menuitem", { name: /hide project/i })).toBeInTheDocument();
    fireEvent.keyDown(window, { key: "Escape" });
    expect(screen.queryByRole("menuitem", { name: /hide project/i })).toBeNull();
  });

  it("closes the popover on outside click", () => {
    render(
      <div>
        <span data-testid="outside">outside</span>
        <ProjectCard project={PROJECT} onMutate={() => {}} onHide={() => {}} />
      </div>,
    );
    fireEvent.click(screen.getByRole("button", { name: /more actions/i }));
    expect(screen.getByRole("menuitem", { name: /hide project/i })).toBeInTheDocument();
    fireEvent.mouseDown(screen.getByTestId("outside"));
    expect(screen.queryByRole("menuitem", { name: /hide project/i })).toBeNull();
  });
});
