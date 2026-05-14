import { describe, expect, it } from "vitest";
import { formatTokens, formatTimeAgo, formatProjectPath } from "./format";

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
