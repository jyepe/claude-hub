import { describe, expect, it, vi, afterEach } from "vitest";
import { render, screen, fireEvent, cleanup } from "@testing-library/react";
import { UndoToast } from "./UndoToast";

afterEach(() => {
  cleanup();
  vi.useRealTimers();
});

describe("UndoToast", () => {
  it("renders the project name", () => {
    render(
      <UndoToast
        project={{ path: "/x", name: "claude-hub" }}
        onUndo={() => {}}
        onDismiss={() => {}}
      />,
    );
    expect(screen.getByText("claude-hub")).toBeInTheDocument();
  });

  it("calls onUndo when Undo is clicked", () => {
    const onUndo = vi.fn();
    render(
      <UndoToast
        project={{ path: "/x", name: "claude-hub" }}
        onUndo={onUndo}
        onDismiss={() => {}}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: /undo/i }));
    expect(onUndo).toHaveBeenCalledTimes(1);
  });

  it("calls onDismiss when × is clicked", () => {
    const onDismiss = vi.fn();
    render(
      <UndoToast
        project={{ path: "/x", name: "claude-hub" }}
        onUndo={() => {}}
        onDismiss={onDismiss}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: /dismiss/i }));
    expect(onDismiss).toHaveBeenCalledTimes(1);
  });

  it("auto-dismisses after 5000ms", () => {
    vi.useFakeTimers();
    const onDismiss = vi.fn();
    render(
      <UndoToast
        project={{ path: "/x", name: "claude-hub" }}
        onUndo={() => {}}
        onDismiss={onDismiss}
      />,
    );
    expect(onDismiss).not.toHaveBeenCalled();
    vi.advanceTimersByTime(4999);
    expect(onDismiss).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(onDismiss).toHaveBeenCalledTimes(1);
  });

  it("resets the dismiss timer when project.path changes", () => {
    vi.useFakeTimers();
    const onDismiss = vi.fn();
    const { rerender } = render(
      <UndoToast
        project={{ path: "/a", name: "first" }}
        onUndo={() => {}}
        onDismiss={onDismiss}
      />,
    );
    vi.advanceTimersByTime(4000);
    rerender(
      <UndoToast
        project={{ path: "/b", name: "second" }}
        onUndo={() => {}}
        onDismiss={onDismiss}
      />,
    );
    // After re-key, the original 4000ms doesn't count.
    vi.advanceTimersByTime(4000);
    expect(onDismiss).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1000);
    expect(onDismiss).toHaveBeenCalledTimes(1);
  });

  it("has role=status for screen readers", () => {
    render(
      <UndoToast
        project={{ path: "/x", name: "claude-hub" }}
        onUndo={() => {}}
        onDismiss={() => {}}
      />,
    );
    expect(screen.getByRole("status")).toBeInTheDocument();
  });
});
