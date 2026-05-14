import { describe, expect, it } from "vitest";
import { windowFor } from "./SessionRow";

describe("windowFor", () => {
  it("liveWindow short-circuits all other signals", () => {
    // Even when every other signal says 200k, the live cache wins.
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
