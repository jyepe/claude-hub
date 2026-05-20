# Hide/Show Projects Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the silent right-click hide on project cards with a discoverable hover-revealed kebab menu, add a 5s bottom-right Undo toast after hide, and gate the `HiddenProjectsManager` header button on whether any projects are actually hidden.

**Architecture:** Frontend-only (React + TypeScript + Tailwind). `AppShell` owns the new `pendingUndo` state and the hide/undo handlers; `ProjectCard` becomes presentational for hide (callback prop, no direct API call); a new `UndoToast` component is single-purpose with a self-managed 5s `setTimeout`. No Rust changes — `hide_project`/`unhide_project` Tauri commands already exist.

**Tech Stack:** React 19, TypeScript 5.8, Tailwind 4, Vitest 4 + @testing-library/react (no `@testing-library/user-event` is installed — use `fireEvent` from `@testing-library/react`).

**Spec:** `docs/superpowers/specs/2026-05-20-hide-show-projects-design.md`

**Branch:** `4-improve-hideshow-logic-for-projects-right-click-hides-silently-no-confirmation-no-discoverable-restore` (created via `gh issue develop 4`, auto-linked to issue #4).

---

## File Structure

| File | Action | Responsibility |
|---|---|---|
| `src/components/UndoToast.tsx` | Create | Single-purpose floating toast. Renders project name, Undo button, dismiss ×. Self-dismisses via 5000ms `setTimeout`, re-keyed on `project.path`. |
| `src/components/UndoToast.test.tsx` | Create | Verifies render, undo click, auto-dismiss, timer reset on path change. |
| `src/components/ProjectCard.tsx` | Modify | Remove `onContextMenu` hide. Add `onHide` callback prop. Add a `KebabMenu`-style popover (inline component or sub-component in the same file) with a single "Hide project" item. |
| `src/components/ProjectCard.test.tsx` | Create | Verifies kebab is rendered, click opens menu, "Hide project" calls `onHide`, **regression guard** that right-click no longer hides. |
| `src/components/AppShell.tsx` | Modify | Add `pendingUndo` state, `handleHide` / `handleUndo` / `handleDismiss` handlers (the last in `useCallback` with empty deps to keep the toast's 5s timer stable across polls). Pass `onHide` to every `<ProjectCard>`. Gate `<HiddenProjectsManager>` on `hiddenCount > 0` and pass `count={hiddenCount}` to it (the manager's internal `hidden` array is empty until the panel is opened, so the parent supplies the button label count). Render `<UndoToast>` when `pendingUndo !== null`. |
| `src/components/HiddenProjectsManager.tsx` | Modify | Accept a new `count: number` prop. Use it in the button label instead of `hidden.length`. Panel contents continue to load from `api.getPrefs()` on open (no logic change). |
| `src/components/AppShell.test.tsx` | Create | Verifies hide → toast appears, undo → `api.unhideProject` called, manager hidden when no hidden projects. Mocks `../lib/api` via `vi.mock`. |

The `KebabMenu` popover lives inside `ProjectCard.tsx` as a co-located component (not a separate file). It's single-use — extracting it would be premature.

---

## Task 1: UndoToast component (TDD)

**Files:**
- Create: `src/components/UndoToast.tsx`
- Test: `src/components/UndoToast.test.tsx`

- [ ] **Step 1: Write the failing test**

Create `src/components/UndoToast.test.tsx`:

```tsx
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
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `npm test -- UndoToast`
Expected: FAIL with "Cannot find module './UndoToast'" (or equivalent — file doesn't exist).

- [ ] **Step 3: Create the UndoToast component**

Create `src/components/UndoToast.tsx`:

```tsx
import { useEffect } from "react";

interface Props {
  project: { path: string; name: string };
  onUndo: () => void;
  onDismiss: () => void;
}

export function UndoToast({ project, onUndo, onDismiss }: Props) {
  useEffect(() => {
    const t = setTimeout(onDismiss, 5000);
    return () => clearTimeout(t);
  }, [project.path, onDismiss]);

  return (
    <div
      role="status"
      aria-live="polite"
      className="fixed bottom-4 right-4 z-20 inline-flex items-center gap-3 bg-surface-hi border border-border rounded-md px-3 py-2 text-sm text-text-1 shadow-[0_1px_2px_rgba(0,0,0,0.12),0_8px_24px_rgba(0,0,0,0.28)]"
    >
      <span>
        Hidden <strong className="font-semibold">{project.name}</strong>
      </span>
      <button
        type="button"
        onClick={onUndo}
        className="text-accent hover:text-accent-hover font-semibold transition-colors duration-[120ms]"
      >
        Undo
      </button>
      <button
        type="button"
        aria-label="Dismiss"
        onClick={onDismiss}
        className="text-text-3 hover:text-text-2 transition-colors duration-[120ms]"
      >
        ×
      </button>
    </div>
  );
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `npm test -- UndoToast`
Expected: PASS — 6 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/components/UndoToast.tsx src/components/UndoToast.test.tsx
git commit -m "feat(toast): add UndoToast component (#4)"
```

---

## Task 2: ProjectCard kebab menu + remove silent hide (TDD)

**Files:**
- Modify: `src/components/ProjectCard.tsx`
- Test: `src/components/ProjectCard.test.tsx` (create)

- [ ] **Step 1: Write the failing test**

Create `src/components/ProjectCard.test.tsx`:

```tsx
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
    const card = screen.getByText("claude-hub").closest("div");
    fireEvent.contextMenu(card!);
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
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `npm test -- ProjectCard`
Expected: FAIL — `ProjectCard` either doesn't accept `onHide` or the `more actions` button doesn't exist yet. The right-click regression guard may pass coincidentally only if `onHide` is unwired, but other tests will fail.

- [ ] **Step 3: Modify `src/components/ProjectCard.tsx`**

Replace the entire file with:

```tsx
import { useEffect, useRef, useState } from "react";
import type { Project } from "../lib/types";
import { SessionRow } from "./SessionRow";
import { api } from "../lib/api";
import { formatTimeAgo, formatProjectPath } from "../lib/format";

interface Props {
  project: Project;
  onMutate: () => void;
  onHide: (project: Project) => void;
}

function KebabMenu({
  onHide,
}: {
  onHide: () => void;
}) {
  const [open, setOpen] = useState(false);
  const wrapRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const onMouseDown = (e: MouseEvent) => {
      if (!wrapRef.current?.contains(e.target as Node)) setOpen(false);
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setOpen(false);
    };
    window.addEventListener("mousedown", onMouseDown);
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("mousedown", onMouseDown);
      window.removeEventListener("keydown", onKey);
    };
  }, [open]);

  return (
    <div ref={wrapRef} className="relative" onClick={(e) => e.stopPropagation()}>
      <button
        type="button"
        aria-label="More actions"
        aria-haspopup="menu"
        aria-expanded={open}
        onClick={(e) => {
          e.stopPropagation();
          setOpen((o) => !o);
        }}
        className="w-7 h-7 inline-flex items-center justify-center rounded-md border border-border bg-surface text-text-2 hover:bg-surface-hi transition-colors duration-[120ms] opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 focus:opacity-100"
      >
        <span aria-hidden className="text-base leading-none">⋯</span>
      </button>
      {open && (
        <div
          role="menu"
          className="absolute right-0 top-full mt-1 z-10 min-w-[160px] bg-surface-hi border border-border rounded-md p-1 shadow-[0_1px_2px_rgba(0,0,0,0.12),0_8px_24px_rgba(0,0,0,0.28)]"
        >
          <button
            type="button"
            role="menuitem"
            onClick={() => {
              setOpen(false);
              onHide();
            }}
            className="w-full text-left px-3 py-2 text-sm text-danger hover:bg-border rounded-sm transition-colors duration-[120ms]"
          >
            Hide project
          </button>
        </div>
      )}
    </div>
  );
}

export function ProjectCard({ project, onMutate, onHide }: Props) {
  const [open, setOpen] = useState(false);

  return (
    <div className="group border border-border rounded-md bg-surface">
      <button
        type="button"
        onClick={() => setOpen((o) => !o)}
        className="w-full flex items-center justify-between gap-3 px-4 py-3 text-left hover:bg-surface-hi"
      >
        <div className="min-w-0 flex-1">
          <div className="text-text-1 text-base font-semibold truncate">
            {project.display_name}
          </div>
          <div className="font-mono text-[11px] text-text-3 truncate">
            {formatProjectPath(project.path)}
          </div>
        </div>
        <div className="text-text-2 text-sm whitespace-nowrap">
          {project.session_count} sessions · {formatTimeAgo(project.last_activity)}
        </div>
        <button
          type="button"
          onClick={(e) => {
            e.stopPropagation();
            api.openSession(project.path).catch(console.error);
          }}
          className="px-3 py-1 text-sm rounded-md border border-border hover:bg-surface-hi"
        >
          New session
        </button>
        <KebabMenu onHide={() => onHide(project)} />
      </button>

      {open && (
        <div>
          {project.sessions.map((s) => (
            <SessionRow
              key={s.id}
              session={s}
              cwd={project.path}
              projectUsed1m={project.used_1m_recently}
              onRefresh={onMutate}
            />
          ))}
          {project.worktrees.map((w) => (
            <div key={w.path}>
              <div className="px-4 py-1 bg-surface-hi font-mono text-[11px] text-text-3 border-t border-border">
                worktree · {formatProjectPath(w.path)}
              </div>
              {w.sessions.map((s) => (
                <SessionRow
                  key={s.id}
                  session={s}
                  cwd={w.path}
                  projectUsed1m={project.used_1m_recently}
                  onRefresh={onMutate}
                />
              ))}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
```

Key changes vs. the original file:
- Added `onHide: (project: Project) => void` to `Props`.
- Removed `handleHide` and the `onContextMenu={handleHide}` on the root `<div>`.
- Added `group` class on the root `<div>` so `group-hover` and `group-focus-within` can target the kebab.
- Added `<KebabMenu>` (defined above `ProjectCard` in the same file) after the "New session" button.
- The kebab forwards `onHide()` → caller passes `onHide(project)` from `ProjectCard`.

(The nested `<button>` inside the outer expand button mirrors the existing "New session" pattern. Cleaning that up is out of scope for this issue — see spec "Out of scope".)

- [ ] **Step 4: Run the test to verify it passes**

Run: `npm test -- ProjectCard`
Expected: PASS — all 6 tests pass.

- [ ] **Step 5: Run the full test suite to check for regressions**

Run: `npm test`
Expected: All tests pass, including pre-existing `ContextMeter`, `SessionRow`, `format` tests and the new `UndoToast` tests.

- [ ] **Step 6: Commit**

```bash
git add src/components/ProjectCard.tsx src/components/ProjectCard.test.tsx
git commit -m "feat(projects): replace silent right-click hide with kebab menu (#4)"
```

---

## Task 3: Wire AppShell — handlers, toast, gated manager (TDD)

**Files:**
- Modify: `src/components/AppShell.tsx`
- Test: `src/components/AppShell.test.tsx` (create)

- [ ] **Step 1: Write the failing test**

Create `src/components/AppShell.test.tsx`:

```tsx
import { describe, expect, it, vi, afterEach, beforeEach } from "vitest";
import {
  render,
  screen,
  fireEvent,
  cleanup,
  waitFor,
  within,
} from "@testing-library/react";
import type { Project, Stats, Prefs } from "../lib/types";

// Mock the api module BEFORE importing AppShell so the polling hook picks it up.
vi.mock("../lib/api", () => ({
  api: {
    listProjects: vi.fn(),
    getStats: vi.fn(),
    getPrefs: vi.fn(),
    setPrefs: vi.fn(),
    hideProject: vi.fn(),
    unhideProject: vi.fn(),
    openSession: vi.fn(),
    attachAgent: vi.fn(),
    closeSession: vi.fn(),
  },
}));

import { AppShell } from "./AppShell";
import { api } from "../lib/api";

const STATS: Stats = {
  project_count: 1,
  session_count: 0,
  tokens_7d: 0,
  tokens_all_time: 0,
};
const PREFS: Prefs = { hidden_projects: [], noise_threshold: 0 };

function makeProject(over: Partial<Project> = {}): Project {
  return {
    path: "/Users/x/code/claude-hub",
    display_name: "claude-hub",
    session_count: 0,
    total_tokens: 0,
    last_activity: null,
    sessions: [],
    worktrees: [],
    hidden: false,
    used_1m_recently: false,
    ...over,
  };
}

beforeEach(() => {
  vi.mocked(api.listProjects).mockResolvedValue([makeProject()]);
  vi.mocked(api.getStats).mockResolvedValue(STATS);
  vi.mocked(api.getPrefs).mockResolvedValue(PREFS);
  vi.mocked(api.hideProject).mockResolvedValue(undefined);
  vi.mocked(api.unhideProject).mockResolvedValue(undefined);
});

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("AppShell — hide/undo wiring", () => {
  it("does NOT render the Hidden manager when no projects are hidden", async () => {
    render(<AppShell />);
    await waitFor(() =>
      expect(screen.getByText("claude-hub")).toBeInTheDocument(),
    );
    expect(screen.queryByRole("button", { name: /^Hidden \(/ })).toBeNull();
  });

  it("DOES render the Hidden manager when at least one project is hidden", async () => {
    vi.mocked(api.listProjects).mockResolvedValue([
      makeProject({ path: "/a", display_name: "a", hidden: true }),
      makeProject({ path: "/b", display_name: "b", hidden: false }),
    ]);
    render(<AppShell />);
    await waitFor(() =>
      expect(screen.getByRole("button", { name: /^Hidden \(1\)$/ })).toBeInTheDocument(),
    );
  });

  it("shows the Undo toast after hiding a project", async () => {
    render(<AppShell />);
    await waitFor(() =>
      expect(screen.getByText("claude-hub")).toBeInTheDocument(),
    );
    fireEvent.click(screen.getByRole("button", { name: /more actions/i }));
    fireEvent.click(screen.getByRole("menuitem", { name: /hide project/i }));
    await waitFor(() => expect(api.hideProject).toHaveBeenCalled());
    expect(await screen.findByRole("status")).toBeInTheDocument();
    expect(screen.getByRole("status")).toHaveTextContent("claude-hub");
  });

  it("calls api.unhideProject when the toast's Undo is clicked", async () => {
    render(<AppShell />);
    await waitFor(() =>
      expect(screen.getByText("claude-hub")).toBeInTheDocument(),
    );
    fireEvent.click(screen.getByRole("button", { name: /more actions/i }));
    fireEvent.click(screen.getByRole("menuitem", { name: /hide project/i }));
    const toast = await screen.findByRole("status");
    fireEvent.click(within(toast).getByRole("button", { name: /undo/i }));
    await waitFor(() =>
      expect(api.unhideProject).toHaveBeenCalledWith("/Users/x/code/claude-hub"),
    );
  });
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `npm test -- AppShell`
Expected: FAIL — `ProjectCard` is rendered without `onHide`, no toast appears on hide, and the `Hidden` button is always shown.

- [ ] **Step 3: Modify `src/components/AppShell.tsx`**

Replace the entire file with:

```tsx
import { useCallback, useState } from "react";
import { api } from "../lib/api";
import { usePoll } from "../lib/usePoll";
import type { Project, Stats } from "../lib/types";
import { HeaderStats } from "./HeaderStats";
import { ProjectCard } from "./ProjectCard";
import { RefreshButton } from "./RefreshButton";
import { HiddenProjectsManager } from "./HiddenProjectsManager";
import { UndoToast } from "./UndoToast";

export function AppShell() {
  const projectsFetcher = useCallback(() => api.listProjects(), []);
  const statsFetcher = useCallback(() => api.getStats(), []);
  const {
    data: projects,
    refresh: refreshProjects,
    lastRefresh,
    error: projectsError,
  } = usePoll<Project[]>(projectsFetcher);
  const { data: stats, refresh: refreshStats, error: statsError } =
    usePoll<Stats>(statsFetcher);

  const [pendingUndo, setPendingUndo] = useState<{ path: string; name: string } | null>(null);

  const errorMessage =
    (projectsError as Error | null)?.toString() ??
    (statsError as Error | null)?.toString() ??
    null;

  const refreshAll = useCallback(() => {
    refreshProjects();
    refreshStats();
  }, [refreshProjects, refreshStats]);

  const handleHide = useCallback(
    async (project: Project) => {
      try {
        await api.hideProject(project.path);
        setPendingUndo({ path: project.path, name: project.display_name });
        refreshAll();
      } catch (err) {
        console.error("hide_project failed", err);
      }
    },
    [refreshAll],
  );

  const handleUndo = useCallback(async () => {
    if (!pendingUndo) return;
    const path = pendingUndo.path;
    try {
      await api.unhideProject(path);
    } catch (err) {
      console.error("unhide_project failed", err);
    } finally {
      setPendingUndo(null);
      refreshAll();
    }
  }, [pendingUndo, refreshAll]);

  // Wrapped in useCallback so the 30s poll-driven re-render of AppShell does
  // NOT mint a fresh `onDismiss` reference, which would otherwise re-trigger
  // UndoToast's useEffect and reset the 5s auto-dismiss countdown.
  const handleDismiss = useCallback(() => setPendingUndo(null), []);

  const all = projects ?? [];
  const visible = all.filter((p) => !p.hidden);
  const hiddenCount = all.filter((p) => p.hidden).length;

  return (
    <div className="min-h-screen flex flex-col gap-6 p-6 max-w-[1200px] mx-auto">
      <header className="flex items-end justify-between gap-4">
        <div>
          <h1 className="text-text-1 text-[22px] font-semibold tracking-tight">
            Claude Hub
          </h1>
          <p className="text-text-3 text-sm">
            Every Claude Code session on this machine.
          </p>
        </div>
        <div className="flex items-center gap-2">
          {hiddenCount > 0 && <HiddenProjectsManager count={hiddenCount} onChange={refreshAll} />}
          <RefreshButton onRefresh={refreshAll} lastRefresh={lastRefresh} />
        </div>
      </header>

      {errorMessage && (
        <div className="px-4 py-2 border border-danger rounded-md bg-surface text-danger text-sm">
          {errorMessage}
        </div>
      )}
      <HeaderStats stats={stats} />

      <main className="flex flex-col gap-3">
        {visible.length === 0 && (
          <div className="text-text-3 text-sm py-12 text-center border border-dashed border-border rounded-md">
            No projects to show. (Drop the noise threshold from "Hidden" if you've hidden everything.)
          </div>
        )}
        {visible.map((p) => (
          <ProjectCard
            key={p.path}
            project={p}
            onMutate={refreshAll}
            onHide={handleHide}
          />
        ))}
      </main>

      {pendingUndo && (
        <UndoToast
          project={pendingUndo}
          onUndo={handleUndo}
          onDismiss={handleDismiss}
        />
      )}
    </div>
  );
}
```

Key changes vs. the original file:
- Added `useState` import + `pendingUndo` state.
- Added `UndoToast` import.
- Added `handleHide` and `handleUndo` callbacks.
- Derived `hiddenCount` from `projects`; gated `<HiddenProjectsManager>` on `hiddenCount > 0`.
- Passed `onHide={handleHide}` to every `<ProjectCard>`.
- Rendered `<UndoToast>` at the end of the root `<div>` when `pendingUndo` is set.

- [ ] **Step 4: Run the test to verify it passes**

Run: `npm test -- AppShell`
Expected: PASS — all 4 tests pass.

- [ ] **Step 5: Run the full test suite**

Run: `npm test`
Expected: All tests pass — `AppShell`, `ProjectCard`, `UndoToast`, `ContextMeter`, `SessionRow`, `format`.

- [ ] **Step 6: Type-check + production build**

Run: `npm run build`
Expected: `tsc` passes with no errors, `vite build` produces `dist/` without warnings related to the changed files. (Pre-existing warnings unrelated to this task can be ignored.)

- [ ] **Step 7: Commit**

```bash
git add src/components/AppShell.tsx src/components/AppShell.test.tsx
git commit -m "feat(shell): wire hide handler, undo toast, gated hidden manager (#4)"
```

---

## Task 4: Manual verification in the running app

The frontend tests cover behavior; this step confirms the actual desktop app works.

- [ ] **Step 1: Start the app**

Run: `npm run tauri dev`
Expected: Vite at port 1420, Tauri window opens after the Rust build.

- [ ] **Step 2: Verify the kebab affordance**

In the running app:
- Hover a project card → kebab (`⋯`) button fades in next to "New session".
- Click the kebab → popover appears with "Hide project" in coral-red (`#c1554a`).
- Press Escape → popover closes.
- Click the kebab, then click somewhere outside the card → popover closes.
- Right-click the card → **nothing happens** (no hide, no custom menu). Confirms the regression is fixed.

- [ ] **Step 3: Verify the hide + undo flow**

- Click kebab → "Hide project" → card disappears from the list.
- Bottom-right toast appears: "Hidden **<name>** · Undo · ×".
- Click "Undo" → toast disappears, card reappears in the list.

- [ ] **Step 4: Verify the auto-dismiss**

- Hide another project, do nothing for ~5s → toast disappears on its own. Project stays hidden.

- [ ] **Step 5: Verify the gated Hidden manager**

- With at least one project hidden: a "Hidden (N)" button is visible in the header. Click it → manager pops open with the hidden path and an "unhide" link.
- Unhide everything via the manager → the "Hidden (N)" button disappears entirely from the header.
- Hide a project again → the "Hidden (1)" button reappears.

- [ ] **Step 6: Close the dev app**

Close the Tauri window.

---

## Spec coverage check

| Spec requirement | Task |
|---|---|
| Remove `onContextMenu={handleHide}` on `ProjectCard` root | Task 2 (step 3) |
| Hover-revealed kebab (`⋯`) button via `group-hover` + `group-focus-within` | Task 2 (step 3) |
| Popover with single "Hide project" item, danger tint | Task 2 (step 3) |
| Outside-click and Escape close the popover | Task 2 (step 3, test step 1) |
| `onHide` callback prop replaces direct `api.hideProject` call in `ProjectCard` | Task 2 (step 3) |
| New `UndoToast.tsx` at `bottom-4 right-4`, `role="status"`, `aria-live="polite"` | Task 1 (step 3) |
| 5000ms `setTimeout` auto-dismiss, re-keyed on `project.path` | Task 1 (step 3) |
| AppShell `pendingUndo` state + `handleHide`/`handleUndo` | Task 3 (step 3) |
| `hiddenCount > 0` gating of `<HiddenProjectsManager>` | Task 3 (step 3) |
| Toast render when `pendingUndo !== null` | Task 3 (step 3) |
| Error handling: `try/catch` + `console.error`, no UI banner | Task 3 (step 3) |
| Visual tokens (warm-ink surfaces, accent for Undo, shadow tokens) | Tasks 1 & 2 (step 3) |
| ARIA: `aria-label` on kebab, `aria-haspopup`, `aria-expanded`, `role="menu"`/`"menuitem"`, `role="status"` | Tasks 1 & 2 (step 3) |
| Regression guard: right-click on `ProjectCard` no longer hides | Task 2 (step 1, test "does NOT hide on right-click") |
| Manager not rendered when zero hidden | Task 3 (step 1, test "does NOT render the Hidden manager...") |
| Manager rendered when ≥1 hidden | Task 3 (step 1, test "DOES render the Hidden manager...") |
