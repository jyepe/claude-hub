import { describe, expect, it, vi, afterEach } from "vitest";
import { render, screen, cleanup } from "@testing-library/react";
import type { Session } from "../lib/types";
import { TitleBar } from "./TitleBar";

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

afterEach(() => {
  cleanup();
  vi.useRealTimers();
});

describe("TitleBar", () => {
  it("renders 'N active today' from given sessions", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-05-20T14:00:00Z"));
    render(
      <TitleBar
        allSessions={[
          makeSession({ last_activity: "2026-05-20T13:00:00Z" }),
          makeSession({ last_activity: "2026-05-19T15:00:00Z" }),
          makeSession({ last_activity: null }),
        ]}
        onRefresh={() => {}}
        lastRefresh={null}
      />,
    );
    // Count is wrapped in a <span> so getByText against the full string can't
    // match across element boundaries — assert on the container's textContent.
    expect(screen.getByText(/active today/i)).toHaveTextContent(/2\s+active today/i);
  });

  it("renders the wordmark and refresh button", () => {
    render(
      <TitleBar
        allSessions={[]}
        onRefresh={() => {}}
        lastRefresh={null}
      />,
    );
    expect(screen.getByRole("button", { name: /refresh/i })).toBeInTheDocument();
    expect(screen.getByAltText(/claude hub/i)).toBeInTheDocument();
  });

  it("renders a version pill", () => {
    render(
      <TitleBar
        allSessions={[]}
        onRefresh={() => {}}
        lastRefresh={null}
      />,
    );
    // Matches "v0.1.0" or whatever package.json currently reports.
    expect(screen.getByText(/^v\d+\.\d+\.\d+/)).toBeInTheDocument();
  });
});
