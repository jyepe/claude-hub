import { describe, expect, it } from "vitest";
import { render } from "@testing-library/react";
import { SessionRow, windowFor } from "./SessionRow";
import type { Session } from "../lib/types";

function baseSession(overrides: Partial<Session> = {}): Session {
  return {
    id: "sess-1",
    jsonl_path: "/tmp/sess-1.jsonl",
    cwd: "/tmp/proj",
    title: "first user message",
    model: "claude-opus-4-7",
    message_count: 4,
    tokens: 1234,
    context_tokens: 1000,
    max_prompt_tokens: 1000,
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
    ...overrides,
  };
}

describe("windowFor", () => {
  it("liveWindow short-circuits all other signals", () => {
    expect(windowFor("claude-opus-4-7", 0, false, 1_000_000)).toBe(1_000_000);
    expect(windowFor("claude-opus-4-7[1m]", 0, false, 200_000)).toBe(200_000);
  });

  it("returns 1M when model has [1m] suffix", () => {
    expect(windowFor("claude-opus-4-7[1m]", 0, false)).toBe(1_000_000);
  });

  it("returns 1M when any prompt exceeded 200k (observable upgrade)", () => {
    expect(windowFor("claude-opus-4-7", 250_000, false)).toBe(1_000_000);
  });

  it("returns 1M when project recently used 1m on opus", () => {
    expect(windowFor("claude-opus-4-7", 50_000, true)).toBe(1_000_000);
  });

  it("returns 1M when project recently used 1m on sonnet", () => {
    expect(windowFor("claude-sonnet-4-6", 50_000, true)).toBe(1_000_000);
  });

  it("does not extend haiku to 1M from project hint (no 1M variant)", () => {
    expect(windowFor("claude-haiku-4-5-20251001", 50_000, true)).toBe(200_000);
  });

  it("falls back to 200k for base opus without signals", () => {
    expect(windowFor("claude-opus-4-7", 50_000, false)).toBe(200_000);
  });

  it("returns 200k when model is null", () => {
    expect(windowFor(null, 0, false)).toBe(200_000);
  });

  it("returns 200k for unknown models", () => {
    expect(windowFor("some-future-model", 0, false)).toBe(200_000);
  });
});

describe("SessionRow bg-agent variant", () => {
  it("renders bg_name as the title and the state subtitle for a bg agent", () => {
    const session = baseSession({
      is_bg_agent: true,
      bg_state: "running",
      bg_detail: "task in progress; reading files",
      bg_name: "job application history",
      title: "should not appear",
    });
    const { container, getByText } = render(
      <SessionRow session={session} cwd="/tmp/proj" projectUsed1m={false} onRefresh={() => {}} />,
    );
    expect(getByText("job application history")).toBeTruthy();
    expect(getByText("RUNNING")).toBeTruthy();
    expect(getByText("task in progress; reading files")).toBeTruthy();
    // model/msgs subtitle is dropped for bg rows
    expect(container.textContent ?? "").not.toContain("4 msgs");
    // running → amber dot
    expect(container.querySelector(".bg-warn")).not.toBeNull();
  });

  it("falls back to bg_intent when bg_name is null", () => {
    const session = baseSession({
      is_bg_agent: true,
      bg_state: "done",
      bg_intent: "/wiki search question",
      bg_name: null,
    });
    const { container, getByText } = render(
      <SessionRow session={session} cwd="/tmp/proj" projectUsed1m={false} onRefresh={() => {}} />,
    );
    expect(getByText("/wiki search question")).toBeTruthy();
    expect(getByText("DONE")).toBeTruthy();
    expect(container.querySelector(".bg-ok")).not.toBeNull();
  });

  it("falls back to session.title when both bg_name and bg_intent are null", () => {
    const session = baseSession({
      is_bg_agent: true,
      bg_state: "error",
      bg_name: null,
      bg_intent: null,
      title: "fallback title",
    });
    const { container, getByText } = render(
      <SessionRow session={session} cwd="/tmp/proj" projectUsed1m={false} onRefresh={() => {}} />,
    );
    expect(getByText("fallback title")).toBeTruthy();
    expect(getByText("ERROR")).toBeTruthy();
    // error → red dot
    expect(container.querySelector(".bg-danger")).not.toBeNull();
  });

  it("renders UNKNOWN with neutral dot when bg_state is null", () => {
    const session = baseSession({
      is_bg_agent: true,
      bg_state: null,
      bg_name: "agent x",
    });
    const { container, getByText } = render(
      <SessionRow session={session} cwd="/tmp/proj" projectUsed1m={false} onRefresh={() => {}} />,
    );
    expect(getByText("UNKNOWN")).toBeTruthy();
    expect(container.querySelector(".bg-text-3")).not.toBeNull();
  });

  it("non-bg-agent rows render existing title and model subtitle unchanged", () => {
    const session = baseSession({
      is_bg_agent: false,
      title: "user prompt",
      model: "claude-opus-4-7",
    });
    const { getByText } = render(
      <SessionRow session={session} cwd="/tmp/proj" projectUsed1m={false} onRefresh={() => {}} />,
    );
    expect(getByText("user prompt")).toBeTruthy();
    // model/msgs/tokens subtitle present
    expect(getByText(/claude-opus-4-7 · 4 msgs/)).toBeTruthy();
  });
});
