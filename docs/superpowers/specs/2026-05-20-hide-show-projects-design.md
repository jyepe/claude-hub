# Hide/Show Projects — Design

**Issue:** [#4 — Improve hide/show logic for projects](https://github.com/jyepe/claude-hub/issues/4)
**Date:** 2026-05-20
**Scope:** Frontend only. No Rust changes — `hide_project` / `unhide_project` Tauri commands already exist.

## Problem

Right-clicking a `ProjectCard` immediately hides the project with no confirmation, no toast, and no visible affordance to restore it. Easy to trigger by accident (especially when reaching for the OS context menu), and the project simply disappears — which reads as a bug.

Today, getting back to a hidden project requires knowing that the "Hidden (N)" button in the header opens a manager — which is itself not discoverable. The "Hidden (0)" affordance is always visible, even when there is nothing to manage.

## Solution at a glance

1. Remove the bare `onContextMenu` hide. Replace with a hover-revealed **kebab (`⋯`) button** on each project card. Click the kebab to open a small popover containing a single "Hide project" item.
2. After a hide, show a **bottom-right toast** ("Hidden *project-name* · Undo · ×") that auto-dismisses after 5s. Undo calls `unhide_project`.
3. **Hide the "Hidden (N)" button entirely when no projects are hidden.** It re-appears the moment the first project is hidden — paired with the toast's Undo, the recovery path becomes discoverable on demand.

## Component breakdown

### `src/components/ProjectCard.tsx` (modified)

- Remove `onContextMenu={handleHide}` on the root `<div>`.
- Remove the existing `handleHide` function.
- Add a kebab button (28×28, border, `⋯` glyph) immediately to the right of the "New session" button.
  - Visibility: opacity `0` by default, opacity `100` via Tailwind's `group-hover:opacity-100` and `group-focus-within:opacity-100` (so keyboard users can reach it via Tab).
  - Apply Tailwind `group` to the root `<div>` so these selectors work.
- Clicking the kebab opens a small popover; outside-click and `Escape` close it.
- Popover contains one item: "Hide project" (rendered in danger tint, `text-danger`).
- Clicking the item calls the new `onHide(project)` prop. **`ProjectCard` no longer calls `api.hideProject` directly** — the parent (`AppShell`) owns the hide-and-undo lifecycle.
- Add `onHide: (project: Project) => void` to `Props`.

### `src/components/UndoToast.tsx` (new file)

```tsx
interface Props {
  project: { path: string; name: string };
  onUndo: () => void;
  onDismiss: () => void;
}
```

- Fixed at `bottom-4 right-4`, `z-20`.
- Markup: `Hidden <strong>{name}</strong>` · `Undo` (coral, `text-accent hover:text-accent-hover`) · `×` (tertiary text).
- `role="status" aria-live="polite"` for screen-reader announcement.
- Self-dismisses via a `useEffect`:
  ```ts
  useEffect(() => {
    const t = setTimeout(onDismiss, 5000);
    return () => clearTimeout(t);
  }, [project.path, onDismiss]);
  ```
  Keying on `project.path` resets the timer when a second hide replaces the toast.
- Visual tokens: `bg-surface-hi`, `border-border`, `rounded-md` (8px), menu-elevation shadow per `DESIGN.md`.

### `src/components/HiddenProjectsManager.tsx` (no internal change needed)

The component itself stays the same. The visibility gate moves to the parent (`AppShell`) — see below.

### `src/components/AppShell.tsx` (modified)

Add state and handlers:

```tsx
const [pendingUndo, setPendingUndo] = useState<{ path: string; name: string } | null>(null);

const handleHide = useCallback(async (project: Project) => {
  try {
    await api.hideProject(project.path);
    setPendingUndo({ path: project.path, name: project.display_name });
    refreshAll();
  } catch (err) {
    console.error("hide_project failed", err);
  }
}, [refreshAll]);

const handleUndo = useCallback(async () => {
  if (!pendingUndo) return;
  try {
    await api.unhideProject(pendingUndo.path);
  } catch (err) {
    console.error("unhide_project failed", err);
  } finally {
    setPendingUndo(null);
    refreshAll();
  }
}, [pendingUndo, refreshAll]);
```

- Pass `onHide={handleHide}` to every `<ProjectCard>`.
- Derive `hiddenCount` from `projects` (no extra fetch — `Project.hidden` is already populated by `projects::build_project_list()`):
  ```ts
  const hiddenCount = (projects ?? []).filter((p) => p.hidden).length;
  ```
- Conditionally render the manager:
  ```tsx
  {hiddenCount > 0 && <HiddenProjectsManager onChange={refreshAll} />}
  ```
- Render the toast when `pendingUndo` is set:
  ```tsx
  {pendingUndo && (
    <UndoToast
      project={pendingUndo}
      onUndo={handleUndo}
      onDismiss={() => setPendingUndo(null)}
    />
  )}
  ```

## Data flow

**Hide:**

```
ProjectCard (kebab → Hide project)
  → onHide(project)
  → AppShell.handleHide
    → api.hideProject(path)        [Tauri IPC, writes prefs.json]
    → setPendingUndo({ path, name })
    → refreshAll()                 [polls list_projects, get_stats]
  → ProjectCard disappears (Project.hidden becomes true → filtered out of `visible`)
  → UndoToast renders bottom-right
```

**Undo (within 5s):**

```
UndoToast (click "Undo")
  → onUndo()
  → AppShell.handleUndo
    → api.unhideProject(path)
    → setPendingUndo(null)
    → refreshAll()
  → ProjectCard reappears in the list
```

**Auto-dismiss (no click within 5s):**

```
UndoToast (setTimeout 5000ms fires)
  → onDismiss()
  → AppShell setPendingUndo(null)
  → UndoToast unmounts
  → Project remains hidden (the toast disappearing does NOT unhide)
```

**Replacement during the 5s window:** if a second project is hidden while the toast is up, `pendingUndo` is replaced with the new `{ path, name }`. The toast's `useEffect` re-keys on `project.path`, clearing the prior timer and starting a fresh 5s window. The first hide is *not* automatically undoable any more — that's an intentional simplification (one toast at a time).

## Visual specification

All values from `DESIGN.md`:

| Element | Tokens |
|---|---|
| Kebab button | `28×28`, `border border-border`, `rounded-md` (8px), `text-text-2`, hover → `bg-surface-hi`, transition `120ms ease-out` |
| Popover container | `bg-surface-hi`, `border border-border`, `rounded-md`, shadow `0 1px 2px rgba(0,0,0,0.12), 0 8px 24px rgba(0,0,0,0.28)`, `min-w-[160px]`, `p-1` |
| Popover item | `<button>`, `px-3 py-2`, `text-sm`, `text-danger` (#c1554a) for "Hide project", hover → `bg-border`, `rounded-sm` |
| Toast | Same surface/border/shadow as popover; `px-3 py-2`, `gap-3` |
| Toast "Undo" | `text-accent` (#d97757), `hover:text-accent-hover` (#e88a6c), `font-semibold`, `text-sm` |
| Toast dismiss × | `text-text-3` (#6e6864), `hover:text-text-2` |

## Accessibility

- Kebab button: `aria-label="More actions"`, `aria-haspopup="menu"`, `aria-expanded={open}`.
- Popover: `role="menu"`; the "Hide project" item is `role="menuitem"`.
- Outside-click and `Escape` close the popover (single `useEffect` with `mousedown` + `keydown` listeners on `window`).
- `UndoToast`: `role="status" aria-live="polite"`. Undo and dismiss are real `<button>` elements.
- Kebab stays reachable for keyboard users via `:focus-within` on the card root.

## Error handling

- Both `api.hideProject` and `api.unhideProject` are wrapped in `try/catch`. Failures log to `console.error` and *do not* surface a UI banner — `prefs.json` writes don't fail in practice, and the existing top-level `errorMessage` banner is reserved for poll errors.
- If `hideProject` rejects, `setPendingUndo` is not called → no misleading toast.
- If `unhideProject` rejects, the toast still clears (`finally` block) — the user can re-open the project from the `HiddenProjectsManager`.

## Testing

New tests alongside their components in `src/components/` (existing convention — see `ContextMeter.test.tsx`, `SessionRow.test.tsx`). Vitest + React Testing Library.

- **`ProjectCard.test.tsx`**
  - Kebab is hidden by default and visible on `hover` (use `userEvent.hover`).
  - Clicking the kebab opens the popover.
  - Clicking "Hide project" calls `onHide` with the project.
  - **Regression guard:** firing a `contextmenu` event on the card does **not** call `onHide` and does **not** call `api.hideProject`.
- **`UndoToast.test.tsx`**
  - Renders the project name.
  - Clicking "Undo" fires `onUndo`.
  - Auto-dismisses (calls `onDismiss`) after 5000ms — uses `vi.useFakeTimers()` and `vi.advanceTimersByTime(5000)`.
  - Changing the `project.path` prop resets the timer.
- **`AppShell.test.tsx`** (mocking `../lib/api` via `vi.mock`)
  - When a `ProjectCard.onHide` fires, `api.hideProject` is called and the toast appears.
  - Clicking the toast's "Undo" calls `api.unhideProject` with the right path.
  - `HiddenProjectsManager` is **not** rendered when no projects are hidden.
  - `HiddenProjectsManager` **is** rendered when at least one project is hidden.

No Rust tests — no Rust changes.

## Out of scope

- A reusable toast/notification system for other parts of the app. The `UndoToast` is single-purpose. If future features need toasts, extract then.
- "Open in file manager", "Copy path", or other kebab-menu items beyond "Hide project". Easy to add later; YAGNI today.
- Animation/transition on toast appear/disappear beyond default. The motion budget in `DESIGN.md` allows a 180ms ease-out slide-in if it feels right during implementation, but it's optional polish.
- Tooltip on the kebab button explaining the gesture (the issue suggests this; an `aria-label` covers screen readers, and the popover content is self-explanatory).

## Files touched

| File | Change |
|---|---|
| `src/components/ProjectCard.tsx` | Remove `onContextMenu` hide. Add kebab + popover. Add `onHide` prop. |
| `src/components/UndoToast.tsx` | New. |
| `src/components/AppShell.tsx` | Add `pendingUndo` state, `handleHide`/`handleUndo`. Gate `HiddenProjectsManager` on `hiddenCount > 0`. Render toast. |
| `src/components/ProjectCard.test.tsx` | New / extended. |
| `src/components/UndoToast.test.tsx` | New. |
| `src/components/AppShell.test.tsx` | New / extended. |
