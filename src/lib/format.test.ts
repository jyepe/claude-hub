import { describe, expect, it, vi, afterEach } from "vitest";
import type { Session } from "./types";
import { formatTokens, formatTimeAgo, formatProjectPath, activeToday } from "./format";

function makeSession(over: Partial<Session> = {}): Session {
  return {
    id: "x",
    jsonl_path: "/tmp/x.jsonl",
    cwd: null,
    title: null,
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

describe("formatTokens", () => {
  it("formats <1k as raw", () => {
    expect(formatTokens(0)).toBe("0");
    expect(formatTokens(999)).toBe("999");
  });
  it("formats <1M as k", () => {
    expect(formatTokens(1_000)).toBe("1.0k");
    expect(formatTokens(47_321)).toBe("47.3k");
  });
  it("formats >=1M as M", () => {
    expect(formatTokens(1_500_000)).toBe("1.5M");
  });
});

describe("formatTimeAgo", () => {
  it("returns 'just now' for <60s", () => {
    const now = new Date();
    expect(formatTimeAgo(new Date(now.getTime() - 5_000).toISOString())).toBe(
      "just now",
    );
  });
  it("returns minutes for <1h", () => {
    const now = new Date();
    expect(formatTimeAgo(new Date(now.getTime() - 5 * 60_000).toISOString())).toBe(
      "5m ago",
    );
  });
  it("handles null", () => {
    expect(formatTimeAgo(null)).toBe("never");
  });
});

describe("formatProjectPath", () => {
  it("returns last two segments for windows path", () => {
    expect(formatProjectPath("C:\\Users\\me\\Desktop\\x")).toBe("Desktop\\x");
  });
  it("returns last two segments for unix path", () => {
    expect(formatProjectPath("/home/me/projects/x")).toBe("projects/x");
  });
});

describe("activeToday", () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it("counts sessions whose last_activity is within the last 24h", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-05-20T12:00:00Z"));
    const sessions = [
      makeSession({ last_activity: "2026-05-20T11:00:00Z" }), //  1h ago — in
      makeSession({ last_activity: "2026-05-19T13:00:00Z" }), // 23h ago — in
      makeSession({ last_activity: "2026-05-19T11:00:00Z" }), // 25h ago — out
      makeSession({ last_activity: null }),                    // null    — out
    ];
    expect(activeToday(sessions)).toBe(2);
  });

  it("returns 0 for an empty list", () => {
    expect(activeToday([])).toBe(0);
  });
});
