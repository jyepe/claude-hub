# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project state

This is **claude-hub** — a Tauri 2 + React + TypeScript desktop app. A local-first "Mission Control" dashboard for every Claude Code session on the user's machine: project list, session transcripts, MCP servers, installed skills, and a tray icon with live context-window usage on the active session.

The repo is currently the **default Tauri scaffold** (`App.tsx` is the boilerplate greeter, `lib.rs` exposes a `greet` command). Almost nothing in `PROJECT.md` §9's planned structure exists yet. New work is greenfield against that plan, not refactor.

## Source of truth documents

These two files override anything you might infer from the current scaffold. Read them before non-trivial work:

- **`PROJECT.md`** — vision, phased scope (Phase 1 → 7+), data-source table for `~/.claude/`, JSONL parsing gotchas, cross-platform terminal-spawn matrix, planned `src-tauri/src/` module layout, and v0.1 ship checklist.
- **`DESIGN.md`** — full design system (Warm Ink palette, Geist typography, 4px spacing scale, restrained-rounded shape language, motion timings, component specs). Frontmatter is machine-readable tokens.

If a request conflicts with these docs, surface the conflict before coding.

## Commands

```powershell
npm install              # first-time setup (also runs cargo fetch via tauri build.rs on dev)
npm run tauri dev        # start the desktop app (spawns Vite on :1420 + Tauri shell)
npm run dev              # Vite only — useful for pure-frontend iteration in a browser
npm run build            # tsc type-check + vite production build (frontend only)
npm run tauri build      # produce signed/unsigned native bundle (Phase-12 territory)
```

Notes:
- Vite port **1420 is fixed** (`vite.config.ts` uses `strictPort: true`) because Tauri's `devUrl` hardcodes it. Don't change one without the other.
- There is no test runner, linter, or formatter configured yet. If you add one, prefer the ecosystem default (vitest, eslint, prettier) and update this file.
- Rust changes recompile on save during `tauri dev`; they're slower than frontend HMR — be patient on the first build.

## Architecture (intended)

Two processes communicate via Tauri's `invoke` bridge:

- **Rust backend (`src-tauri/src/`)** owns all filesystem I/O against `~/.claude/`, terminal spawning, the `notify` file watcher, and the system tray. Planned modules per `PROJECT.md` §9: `sessions.rs`, `terminal.rs`, `mcp.rs`, `skills.rs`, `stats.rs`, `watcher.rs`, `tray.rs`. Register commands in `lib.rs`.
- **React frontend (`src/`)** is a pure dashboard view. All data flows through `src/lib/api.ts` (Tauri invoke wrappers) — components must not call `invoke` directly. Types live in `src/lib/types.ts`, formatters in `src/lib/format.ts`.

The frontend is **read-only over Claude Code's on-disk state**. Hub does not write into `~/.claude/` (no transcript editing, no settings mutation in v0.1).

## Load-bearing constraints

These are the easy-to-miss decisions that will hurt if violated:

- **Dedupe JSONL events by UUID before summing tokens.** Claude Code writes the same event to multiple JSONLs during branching/resumption — naive sums inflate by 2–4×.
- **Token usage** lives on `assistant` events under `message.usage`. Sum all four: `input_tokens`, `cache_creation_input_tokens`, `cache_read_input_tokens`, `output_tokens`.
- **Never display MCP env values** in the UI. Show env *keys* only. Treat `~/.claude.json` as containing secrets.
- **No pills, no gradients in chrome.** The only gradient in the entire UI is the context meter's fill. The only `9999px` radius use is — none. See `DESIGN.md` "Shapes".
- **Stay on the 4px spacing grid** (`4, 8, 12, 16, 20, 24, 32, 40, 48, 64`). Never `6, 10, 14, 18`.
- **Pure white (`#fff`) is never used.** `#f5f1ec` is the warm ceiling for primary text in dark mode.
- **Borders, not shadows, separate planes.** Shadows exist only for floating layers (menus, modals, command palette).
- **Use `PathBuf` everywhere in Rust**; only stringify at the I/O boundary. Cross-platform path handling matters — this app ships macOS / Linux / Windows from day one.
- **Debounce file-watcher events to ~250ms per file**, and throttle tray-tooltip updates to ~1s, or live-update churn will dominate CPU.

## Phased scope discipline

`PROJECT.md` §5 sequences the work as Phase 1 → 7+. Don't pull features forward across phase boundaries without an explicit ask — the phasing exists so each phase is independently shippable. The v0.1 bar is Phase 1 + 2 (sessions/launcher + MCP/skills panel); everything else is post-MVP.
