# Windows Terminal: tab stays open after "Close session"

## Symptom

You click the **×** button on a live session row. The Claude process dies (good), but the Windows Terminal tab stays open showing:

```
[process exited with code 1]
You can now close this terminal with Ctrl+D, or press Enter to restart.
```

…often with a yellow banner: *"Termination behavior can be configured in advanced profile settings."*

## Cause

This is **not a bug in claude-hub**. The hub correctly kills the entire process tree (claude → shims → `cmd.exe`); you can verify this in the dev-server log, which shows `taskkill /F /T` succeeding on every PID in the chain.

The tab stays visible because Windows Terminal's per-profile **`closeOnExit`** setting is `"graceful"` (the default for the legacy *Command Prompt* profile). With that value, WT keeps the tab open whenever the hosted process exits with a non-zero code — and a force-killed `cmd.exe` always exits non-zero.

The `[process exited…]` line and the *"close this terminal with Ctrl+D"* prompt are Windows Terminal's own exit-handler UI, not a lingering shell.

## Fix

Change the Windows Terminal profile to close tabs unconditionally on exit.

### Via UI

**Settings → Profiles → Defaults → Advanced → "When a command exits" → "Always close window"**

Click **Don't show again** on the yellow banner while you're there.

### Via `settings.json`

```json
{
  "profiles": {
    "defaults": {
      "closeOnExit": "always"
    }
  }
}
```

Acceptable values: `"always"`, `"graceful"`, `"never"`, `"automatic"`. Use `"always"` for the cleanest close-button behavior.

## Why we don't work around it in code

Each alternative we considered was worse than the one-line WT setting:

| Approach | Why we didn't do it |
|---|---|
| Send `WM_CLOSE` to the WT tab's window | WT tabs aren't separate `HWND`s; would need the `windows` crate and per-tab HWND-hunting, which is fragile across WT versions. |
| Switch spawn to `cmd /c` | The killed process still exits non-zero, so WT still keeps the tab open under `"graceful"`. Doesn't help. |
| Drop WT in favor of legacy `cmd /c start cmd` console windows | Legacy windows *do* close on `taskkill /F /T`, but using them everywhere is a UX regression. |

## How the close path actually works (for reference)

1. User clicks **×** → frontend calls `closeSession(session_id)` via `src/lib/api.ts`.
2. `close_session` in `src-tauri/src/lib.rs` looks up the live process from `~/.claude/sessions/{pid}.json` (via `active_sessions::read_all`).
3. `killer::find_shell_ancestor(claude_pid)` walks the process table looking for the first ancestor whose name matches an interactive shell (`cmd.exe`, `pwsh.exe`, `bash`, `zsh`, …).
4. If a shell ancestor is found, `killer::kill_tree(shell_pid)` is called first. On Windows this uses `taskkill /T`, which cascades downward and kills claude as part of the same tree.
5. `killer::kill_tree(claude_pid)` is called as an authoritative second step. On Windows this is a no-op (claude is already dead from the cascade). On Unix it's required because `kill -KILL` does not cascade.

After step 4, `cmd.exe` is gone — what you see is purely Windows Terminal's exit overlay behavior.
