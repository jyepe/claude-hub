import { describe, expect, it, vi, afterEach, beforeEach } from "vitest";
import { render, screen, fireEvent, cleanup } from "@testing-library/react";
import type { Session } from "../lib/types";

vi.mock("../lib/api", () => ({
  api: {
    openSession: vi.fn(),
    attachAgent: vi.fn(),
    closeSession: vi.fn(),
    pinSession: vi.fn(),
    unpinSession: vi.fn(),
  },
}));

import { SessionCard } from "./SessionCard";
import { api } from "../lib/api";

function makeSession(over: Partial<Session> = {}): Session {
  return {
    id: "sess-1",
    jsonl_path: "/tmp/sess-1.jsonl",
    cwd: "/repo",
    title: "Refactor session loader",
    model: "claude-sonnet-4-6",
    message_count: 4,
    tokens: 184_000,
    context_tokens: 120_000,
    max_prompt_tokens: 120_000,
    last_activity: "2026-05-15T12:00:00Z",
    live_context_window: null,
    live_model_id: null,
    is_bg_agent: false,
    live_status: null,
    bg_state: null,
    bg_detail: null,
    bg_tempo: null,
    bg_intent: null,
    bg_name: null,
    recent_excerpt: "pulled the chunk handler into its own module",
    ...over,
  };
}

beforeEach(() => {
  vi.mocked(api.openSession).mockResolvedValue(undefined);
  vi.mocked(api.attachAgent).mockResolvedValue(undefined);
  vi.mocked(api.pinSession).mockResolvedValue(undefined);
  vi.mocked(api.unpinSession).mockResolvedValue(undefined);
});

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("SessionCard", () => {
  it("renders title, recent_excerpt and Open session for a regular session", () => {
    render(
      <SessionCard
        session={makeSession()}
        cwd="/repo"
        projectUsed1m={false}
        isPinned={false}
        focused={false}
        onMutate={() => {}}
      />,
    );
    expect(screen.getByText("Refactor session loader")).toBeInTheDocument();
    expect(screen.getByText(/pulled the chunk handler/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /open session/i })).toBeInTheDocument();
  });

  it("shows a 'running' chip when live_status === 'busy'", () => {
    render(
      <SessionCard
        session={makeSession({ live_status: "busy" })}
        cwd="/repo"
        projectUsed1m={false}
        isPinned={false}
        focused={false}
        onMutate={() => {}}
      />,
    );
    expect(screen.getByText("running")).toBeInTheDocument();
  });

  it("shows an 'idle' chip when live_status === 'idle'", () => {
    render(
      <SessionCard
        session={makeSession({ live_status: "idle" })}
        cwd="/repo"
        projectUsed1m={false}
        isPinned={false}
        focused={false}
        onMutate={() => {}}
      />,
    );
    expect(screen.getByText("idle")).toBeInTheDocument();
  });

  it("uses bg_state precedence for bg agents (done overrides live_status)", () => {
    render(
      <SessionCard
        session={makeSession({
          is_bg_agent: true,
          bg_state: "done",
          live_status: "busy", // should be ignored for bg agents
          bg_name: "Audit kebab z-index",
        })}
        cwd="/repo"
        projectUsed1m={false}
        isPinned={false}
        focused={false}
        onMutate={() => {}}
      />,
    );
    expect(screen.getByText("done")).toBeInTheDocument();
    expect(screen.getByText("Audit kebab z-index")).toBeInTheDocument();
  });

  it("renders Attach button for bg agents", () => {
    render(
      <SessionCard
        session={makeSession({ is_bg_agent: true, bg_state: "running" })}
        cwd="/repo"
        projectUsed1m={false}
        isPinned={false}
        focused={false}
        onMutate={() => {}}
      />,
    );
    expect(screen.getByRole("button", { name: /attach/i })).toBeInTheDocument();
  });

  it("calls api.openSession with cwd + session id when Open is clicked on an inactive session", () => {
    render(
      <SessionCard
        session={makeSession({ id: "sess-99", live_status: null })}
        cwd="/repo"
        projectUsed1m={false}
        isPinned={false}
        focused={false}
        onMutate={() => {}}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: /open session/i }));
    expect(api.openSession).toHaveBeenCalledWith("/repo", "sess-99");
  });

  it("shows 'Already open' text (no Open button) when the session is live", () => {
    render(
      <SessionCard
        session={makeSession({ live_status: "idle" })}
        cwd="/repo"
        projectUsed1m={false}
        isPinned={false}
        focused={false}
        onMutate={() => {}}
      />,
    );
    expect(screen.getByText(/already open/i)).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /open session/i })).toBeNull();
  });

  it("kebab → Pin calls api.pinSession", () => {
    render(
      <SessionCard
        session={makeSession({ id: "sess-99" })}
        cwd="/repo"
        projectUsed1m={false}
        isPinned={false}
        focused={false}
        onMutate={() => {}}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: /more actions/i }));
    fireEvent.click(screen.getByRole("menuitem", { name: /^pin$/i }));
    expect(api.pinSession).toHaveBeenCalledWith("sess-99");
  });

  it("kebab → Unpin appears (and calls api.unpinSession) when isPinned", () => {
    render(
      <SessionCard
        session={makeSession({ id: "sess-99" })}
        cwd="/repo"
        projectUsed1m={false}
        isPinned={true}
        focused={false}
        onMutate={() => {}}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: /more actions/i }));
    fireEvent.click(screen.getByRole("menuitem", { name: /^unpin$/i }));
    expect(api.unpinSession).toHaveBeenCalledWith("sess-99");
  });

  it("Close menu item is gated to live sessions", () => {
    render(
      <SessionCard
        session={makeSession({ live_status: null })}
        cwd="/repo"
        projectUsed1m={false}
        isPinned={false}
        focused={false}
        onMutate={() => {}}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: /more actions/i }));
    expect(screen.queryByRole("menuitem", { name: /^close$/i })).toBeNull();
  });
});
