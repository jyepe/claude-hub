# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project state

This is **claude-hub** — a Tauri 2 + React + TypeScript desktop app. A local-first "Mission Control" dashboard for every Claude Code session on the user's machine: project list, session transcripts, MCP servers, installed skills, and a tray icon with live context-window usage on the active session.

**Phase 1 (Sessions & Launcher) is complete and on `master`.** The scaffold phase is over — all Rust modules and React components listed below exist and are functional.

## Source of truth documents

Read these before non-trivial work:

- **`PROJECT.md`** — vision, phased scope (Phase 1 → 7+), data-source table for `~/.claude/`, JSONL parsing gotchas, cross-platform terminal-spawn matrix, and v0.1 ship checklist.
- **`DESIGN.md`** — full design system (Warm Ink palette, Geist typography, 4px spacing scale, shape language, motion timings, component specs). Frontmatter is machine-readable tokens.

If a request conflicts with these docs, surface the conflict before coding.

## Commands

```powershell
npm install              # first-time setup
npm run tauri dev        # start the desktop app (Vite :1420 + Tauri shell)
npm run dev              # Vite only — frontend iteration in a browser
npm run build            # tsc type-check + vite production build (frontend only)
npm test                 # run frontend tests (vitest)
npm run test:watch       # vitest in watch mode
```

```powershell
# Rust tests (run from src-tauri/)
cargo test                          # all tests
cargo test scanner                  # tests in a specific module
cargo test looks_like_uuid          # single test by name substring
```

Notes:
- Vite port **1420 is fixed** (`vite.config.ts` `strictPort: true`) — Tauri's `devUrl` hardcodes it.
- Rust changes recompile on save during `tauri dev`; first build is slow.

## Architecture

Two processes communicate over Tauri's `invoke` bridge. The frontend never touches the filesystem directly.

### Rust backend (`src-tauri/src/`)

| Module | Responsibility |
|---|---|
| `lib.rs` | Tauri command registration, `AppState` (holds prefs + statusline cache) |
| `sessions.rs` | `Session` struct + `parse_session()` — fail-soft per-line JSONL parser |
| `scanner.rs` | Scans `~/.claude/projects/**/*.jsonl`, calls `parse_session`, marks `is_bg_agent` |
| `cache.rs` | mtime-validated sidecar cache at `~/.claude-hub/cache/` (keeps relaunches fast) |
| `projects.rs` | Groups sessions into `Project` structs, applies prefs filters + worktree grouping |
| `stats.rs` | Aggregates token totals (7-day / all-time) and session/project counts |
| `paths.rs` | Cross-platform path helpers — `claude_dir()`, `claude_jobs_dir()`, `hub_cache_dir()`, etc. |
| `prefs.rs` | `~/.claude-hub/prefs.json` (hidden projects set, noise threshold) |
| `terminal.rs` | Cross-platform terminal spawn: `open_in_terminal` (`claude --resume`) and `attach_in_terminal` (`claude agents attach`) |
| `worktree.rs` | Detects `.git`-file worktrees and resolves them to their parent repo |
| `claude_config.rs` | Reads `~/.claude.json` for recently-used projects and per-project model history |
| `statusline_cache.rs` | Reads `~/.claude-hub/ctx-cache/` for live context % written by the statusline wrapper |

**Data flow for `list_projects`:**
`invoke("list_projects")` → `lib.rs` → `projects::build_project_list()` → `scanner::scan_all()` → per-file `parse_session()` (hydrated from `cache.rs` if mtime matches)

**Background agent detection** (`scanner.rs`):
`scan_all()` reads `~/.claude/jobs/*/state.json` after scanning sessions. Each subfolder is one background agent job. It extracts `sessionId` and `linkScanPath` (for resumed sessions where the JSONL UUID differs from the agent's session ID) and sets `Session.is_bg_agent = true` on matching sessions. Regular interactive sessions never have a jobs entry.

### React frontend (`src/`)

All `invoke` calls go through `src/lib/api.ts` — components never call `invoke` directly.

- `lib/types.ts` — shared TS interfaces (`Session`, `Project`, `Stats`, `Prefs`)
- `lib/format.ts` — token formatting (K/M), time-ago, path display
- `lib/usePoll.ts` — 30s polling hook used by `AppShell`
- `components/AppShell.tsx` — root: polls `list_projects` + `get_stats`, passes data down
- `components/ProjectCard.tsx` — collapsible project row with hide button
- `components/SessionRow.tsx` — session row with `ContextMeter` and Open/Attach button; `windowFor()` infers the model's context window from multiple signals
- `components/ContextMeter.tsx` — token-fill bar (green/amber/red thresholds)
- `components/HeaderStats.tsx` — four stat tiles at the top
- `components/HiddenProjectsManager.tsx` — modal to unhide projects
- `components/RefreshButton.tsx` — manual refresh with last-refreshed timestamp

**Open vs Attach:** `SessionRow` renders "Open" (→ `openSession` → `claude --resume <id>`) for normal sessions and "Attach" (→ `attachAgent` → `claude agents attach <id>`) for sessions where `session.is_bg_agent === true`. See [`docs/session-row-launch.md`](docs/session-row-launch.md) for the full launch flow, bg-agent detection logic, security validation, and per-platform terminal behavior.

## Load-bearing constraints

- **Dedupe JSONL events by UUID before summing tokens.** Claude Code writes the same event to multiple JSONLs during branching/resumption — naive sums inflate by 2–4×.
- **Two distinct token quantities — don't confuse them:**
  - **Lifetime tokens (`Session.tokens`)** — sum of all four `message.usage` fields across every assistant event. Used for header stats.
  - **Context-window fill (`Session.context_tokens`)** — prompt size of the **latest** assistant turn only: `input + cache_creation + cache_read` (no output). Used for `ContextMeter` and the tray tooltip.
  - Never feed the lifetime sum to the context meter — it will peg at 100% on any moderately long session.
- **Never display MCP env values** in the UI. Show env *keys* only. `~/.claude.json` contains secrets.
- **No pills, no gradients in chrome.** The only gradient is the context meter fill. See `DESIGN.md` "Shapes".
- **Stay on the 4px spacing grid** (`4, 8, 12, 16, 20, 24, 32, 40, 48, 64`). Never `6, 10, 14, 18`.
- **Pure white (`#fff`) is never used.** `#f5f1ec` is the warm ceiling for primary text in dark mode.
- **Borders, not shadows, separate planes.** Shadows only for floating layers (menus, modals).
- **Use `PathBuf` everywhere in Rust**; only stringify at the I/O boundary.
- **Debounce file-watcher events to ~250ms per file**, throttle tray tooltip to ~1s (Phase 3/4).

## Phased scope discipline

`PROJECT.md` §5 sequences work as Phase 1 → 7+. Don't pull features forward across phase boundaries without an explicit ask. The v0.1 bar is Phase 1 + 2 (sessions/launcher + MCP/skills panel); everything else is post-MVP.
