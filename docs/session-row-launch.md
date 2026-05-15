# Session Row Launch Behavior

When a user clicks the action button on a session row, the hub spawns a terminal running a `claude` command. The button label and the command depend on whether the session is a live background agent.

## Open vs Attach

| Session type | Button | Command spawned |
|---|---|---|
| Regular session | **Open** | `claude --resume <session-id>` |
| Live background agent | **Attach** | `claude agents attach <session-id>` |

The `session-id` is the UUID derived from the JSONL filename (e.g. `f5ec5e6f-84b2-4375-8ed8-a47fcab8ae24`).

## How `is_bg_agent` is set

`scanner::scan_all()` (Rust) reads `~/.claude/jobs/*/state.json` after parsing all sessions. Each subfolder under `~/.claude/jobs/` corresponds to exactly one background agent job. Two fields are extracted from each `state.json`:

- **`sessionId`** — the full UUID of the agent's session JSONL.
- **`linkScanPath`** — the JSONL file the agent is actively writing to. For *resumed* background agents this path points to the original session's JSONL, whose UUID differs from `sessionId`. Extracting the UUID from this path handles that case.

Any session whose `id` appears in either set gets `is_bg_agent = true`. Regular interactive `claude` sessions never create a jobs entry, so they are never marked.

## Frontend wiring

`SessionRow.tsx` reads `session.is_bg_agent` and conditionally renders:

```tsx
session.is_bg_agent
  ? <button onClick={() => api.attachAgent(cwd, session.id)}>Attach</button>
  : <button onClick={() => api.openSession(cwd, session.id)}>Open</button>
```

Both call through `src/lib/api.ts` → Tauri `invoke` → Rust:

- `openSession` → `terminal::open_in_terminal(cwd, Some(session_id))` → `claude --resume <id>`
- `attachAgent` → `terminal::attach_in_terminal(cwd, session_id)` → `claude agents attach <id>`

## Security

`attach_in_terminal` validates the session ID before building the shell command — only ASCII alphanumeric, `-`, and `_` are accepted. Any other character returns `SpawnError::InvalidSessionId` without spawning a process.

## Platform behavior

Both functions delegate to `spawn_platform`, which tries terminal emulators in this order:

| OS | Primary | Fallback |
|---|---|---|
| Windows | `wt.exe -d <cwd> cmd /k <cmd>` | `cmd /c start cmd /k cd /d "<cwd>" && <cmd>` |
| macOS | AppleScript → `Terminal.app` | — |
| Linux | `gnome-terminal`, `konsole`, `xfce4-terminal`, `alacritty`, `kitty`, `wezterm` (first found) | `SpawnError::NoTerminal` |
