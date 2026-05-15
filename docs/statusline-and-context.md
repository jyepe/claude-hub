# Statusline & context tracking

How claude-hub stays in sync with Claude Code's own `ctx:NN%` display, and what you need to set up to make it work.

## What this is

Claude Code lets you configure a custom statusline command in `~/.claude/settings.json`. Every few seconds, CC pipes a JSON snapshot of the current session (session id, cwd, model, token usage, etc.) into that command's stdin and prints the command's stdout as the bar at the bottom of the terminal.

claude-hub installs a thin wrapper as that command. The wrapper does two things:

1. **Caches the snapshot** to `~/.claude-hub/ctx-cache/{session_id}.json` so the hub UI can read the same ctx% CC is showing.
2. **Forwards** to your existing statusline command (if any) and prints its output, so your bar keeps working. If you don't have one, a minimal fallback is printed (`<model>  <short cwd>  ctx:NN%`).

## Data flow

```
Claude Code (per tick, ~every few seconds)
  │
  │ pipes JSON to stdin
  ▼
claude-hub-statusline wrapper  (.sh on macOS/Linux/WSL, .ps1 on Windows)
  │
  ├─► writes ~/.claude-hub/ctx-cache/{session_id}.json   (cache file)
  │
  └─► forwards stdin → your next statusline → stdout     (the bar you see)

Hub app
  │
  ▼
Rust: statusline_cache::read_all()  →  Session.context_tokens / window inference
  │
  ▼
React: SessionRow → ContextMeter
```

The cache file is the **only** bridge between CC's view of context fill and the hub UI. Without the wrapper installed, the hub falls back to inferring context tokens from the JSONL transcript, which is close but not what CC's own bar shows.

## Two distinct token quantities — don't confuse them

claude-hub tracks two different numbers; the wrapper feeds the second one:

- **Lifetime tokens** — the sum of all four `message.usage` fields (`input_tokens`, `cache_creation_input_tokens`, `cache_read_input_tokens`, `output_tokens`) across every assistant event in the JSONL. Used for header stats and rollups. Long sessions naturally reach millions because cache reads are re-counted each turn.
- **Context-window fill** — prompt size of the **latest** assistant turn only (`input_tokens + cache_creation_input_tokens + cache_read_input_tokens`, no output). Used for the per-session `ContextMeter` and the tray tooltip. This is what the wrapper's cache file makes authoritative.

Never feed the lifetime sum to the context meter — it would peg at 100% on any moderately long session.

## Setup — macOS / Linux / WSL

Use the bash wrapper. In `~/.claude/settings.json`:

```json
{
  "statusLine": {
    "type": "command",
    "command": "bash \"/absolute/path/to/claude-hub/scripts/claude-hub-statusline.sh\""
  }
}
```

Replace `/absolute/path/to/claude-hub` with your local checkout path. Restart Claude Code.

## Setup — Windows

Use the PowerShell wrapper. The bash wrapper works on Git Bash, but on Windows it orphans `bash.exe` child processes that accumulate over time (see [issue #8](https://github.com/jyepe/claude-hub/issues/8)). The PowerShell wrapper does the same work with zero subprocesses.

In `~/.claude/settings.json`:

```json
{
  "statusLine": {
    "type": "command",
    "command": "powershell -NoProfile -ExecutionPolicy Bypass -File \"C:\\absolute\\path\\to\\claude-hub\\scripts\\claude-hub-statusline.ps1\""
  }
}
```

- `-NoProfile` skips loading your PowerShell profile (faster tick).
- `-ExecutionPolicy Bypass` allows the unsigned script to run without prompting.

Replace the path with your local checkout. Restart Claude Code.

**Migrating from an existing `statusLine.command`?** Before replacing the value, copy your current command path somewhere — you'll pass it as `CLAUDE_HUB_STATUSLINE_NEXT` (see [Forwarding](#forwarding-to-an-existing-statusline) below) so your bar keeps working alongside the wrapper.

## Forwarding to an existing statusline

If you already have a custom statusline command, set the environment variable `CLAUDE_HUB_STATUSLINE_NEXT` to its absolute path. The wrapper will pipe the payload to that script and forward its stdout as the bar.

If the env var isn't set, the wrapper auto-detects:

- `~/.claude/statusline-command.sh` (`.sh` wrapper only)
- `~/.claude/statusline-command.ps1` (`.ps1` wrapper only)

If neither is set or found, the built-in fallback (`<model>  <short cwd>  ctx:NN%`) is printed.

> **Windows migration note:** The `.ps1` wrapper does **not** auto-detect `~/.claude/statusline-command.sh`. If you're moving from a bash statusline on Windows, either port your script to PowerShell and save it as `~/.claude/statusline-command.ps1` (it will be picked up automatically), or set `CLAUDE_HUB_STATUSLINE_NEXT` to the full path of a `.ps1` equivalent.

### PowerShell next-scripts: how to read the payload

The `.sh` wrapper forwards stdin via an OS-level pipe — your next script reads it however it would normally read stdin.

The `.ps1` wrapper forwards via PowerShell's object pipeline. **Your next `.ps1` script must read the payload via the `$input` automatic variable, not `[Console]::In.ReadToEnd()`** (the OS-stdin handle has already been consumed by the wrapper). Minimal example:

```powershell
# my-statusline.ps1
$payload = ($input | Out-String).TrimEnd()
$obj = $payload | ConvertFrom-Json
"My bar: $($obj.model.display_name)"
```

## Verification

After installing the wrapper and restarting Claude Code:

**1. Cache file is updating.** Run a `claude` session for ~30 seconds, then:

```bash
# macOS/Linux/WSL
ls -la ~/.claude-hub/ctx-cache/
```

```powershell
# Windows
Get-ChildItem $HOME\.claude-hub\ctx-cache\ | Sort-Object LastWriteTime -Descending | Select-Object -First 5
```

There should be one `<session-uuid>.json` file per active session, and its modification time should be within the last few seconds.

**2. Hub UI matches CC's bar.** Open the hub app and look at the active session's `ContextMeter`. It should read the same percentage CC's own bar shows.

**3. No orphan processes (Windows).** After 5+ minutes of normal interaction:

```powershell
Get-CimInstance Win32_Process -Filter "Name='powershell.exe'" | Where-Object { $_.CommandLine -like '*claude-hub-statusline*' }
Get-CimInstance Win32_Process -Filter "Name='bash.exe'"      | Where-Object { $_.CommandLine -like '*claude-hub-statusline*' }
```

Both should return zero rows. If the `bash.exe` query returns rows, you're still on the `.sh` wrapper on Windows — switch your `settings.json` to the `.ps1` command.
