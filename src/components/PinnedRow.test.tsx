import { describe, expect, it, vi, afterEach, beforeEach } from "vitest";
import { render, screen, fireEvent, cleanup } from "@testing-library/react";
import type { Session } from "../lib/types";

vi.mock("../lib/api", () => ({
  api: { unpinSession: vi.fn() },
}));

import { PinnedRow } from "./PinnedRow";
import { api } from "../lib/api";

function makeSession(over: Partial<Session> = {}): Session {
  return {
    id: "s1", jsonl_path: "/tmp/s1.jsonl", cwd: "/repo",
    title: "Refactor session loader", model: "claude-sonnet-4-6",
    message_count: 0, tokens: 0, context_tokens: 0, max_prompt_tokens: 0,
    last_activity: null,
    live_context_window: null, live_model_id: null,
    is_bg_agent: false, live_status: null,
    bg_state: null, bg_detail: null, bg_tempo: null, bg_intent: null, bg_name: null,
    recent_excerpt: null,
    ...over,
  };
}

beforeEach(() => {
  vi.mocked(api.unpinSession).mockResolvedValue(undefined);
});

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("PinnedRow", () => {
  it("renders title and short model tag", () => {
    render(
      <PinnedRow
        session={makeSession()}
        projectPath="/repo"
        dimmed={false}
        onClick={() => {}}
        onMutate={() => {}}
      />,
    );
    expect(screen.getByText("Refactor session loader")).toBeInTheDocument();
    expect(screen.getByText("sonnet")).toBeInTheDocument();
  });

  it("dimmed style is applied when dimmed=true", () => {
    render(
      <PinnedRow
        session={makeSession()}
        projectPath="/repo"
        dimmed={true}
        onClick={() => {}}
        onMutate={() => {}}
      />,
    );
    expect(screen.getByRole("button", { name: /refactor session loader/i }))
      .toHaveClass("opacity-40");
  });

  it("clicking the row calls onClick with the project path and session id", () => {
    const onClick = vi.fn();
    render(
      <PinnedRow
        session={makeSession({ id: "s9" })}
        projectPath="/repo"
        dimmed={false}
        onClick={onClick}
        onMutate={() => {}}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: /refactor session loader/i }));
    expect(onClick).toHaveBeenCalledWith("/repo", "s9");
  });

  it("unpin button calls api.unpinSession with session id", async () => {
    render(
      <PinnedRow
        session={makeSession({ id: "s9" })}
        projectPath="/repo"
        dimmed={false}
        onClick={() => {}}
        onMutate={() => {}}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: /unpin/i }));
    expect(api.unpinSession).toHaveBeenCalledWith("s9");
  });
});
