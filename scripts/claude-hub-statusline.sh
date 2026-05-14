#!/usr/bin/env bash
# claude-hub statusline wrapper.
#
# Install: point ~/.claude/settings.json's statusLine.command at this script's
# absolute path. Optionally export CLAUDE_HUB_STATUSLINE_NEXT to your previous
# statusline command — its stdout is forwarded as the rendered status line. If
# unset, a minimal fallback is printed so you don't lose the bar entirely.
#
# Side effect: writes ~/.claude-hub/ctx-cache/{session_id}.json on each tick so
# the hub UI can report the same context% Claude Code displays in its own bar.

set -u

input="$(cat)"
cache_dir="${CLAUDE_HUB_DIR:-$HOME/.claude-hub}/ctx-cache"
mkdir -p "$cache_dir" 2>/dev/null || true

# Cache the entire payload + a timestamp keyed by session_id. Best-effort: if
# Python is missing or the payload is malformed, skip the cache and continue.
CACHE_SCRIPT='
import json, os, sys, time, tempfile
cache_dir = sys.argv[1]
try:
    d = json.loads(sys.stdin.read())
except Exception:
    sys.exit(0)
sid = d.get("session_id") or d.get("sessionId")
if not sid:
    sys.exit(0)
entry = {
    "session_id": sid,
    "cwd": d.get("cwd") or (d.get("workspace") or {}).get("current_dir"),
    "model": d.get("model"),
    "context_window": d.get("context_window"),
    "updated_at_ms": int(time.time() * 1000),
}
fd, tmp = tempfile.mkstemp(prefix=".claude-hub-", dir=cache_dir)
try:
    with os.fdopen(fd, "w", encoding="utf-8") as f:
        json.dump(entry, f, separators=(",", ":"))
    os.replace(tmp, os.path.join(cache_dir, sid + ".json"))
except Exception:
    try: os.unlink(tmp)
    except Exception: pass
'
printf '%s' "$input" | python3 -c "$CACHE_SCRIPT" "$cache_dir" 2>/dev/null || true

# Forward to the user's previous statusline if configured. Resolution order:
#   1. $CLAUDE_HUB_STATUSLINE_NEXT (explicit override)
#   2. $HOME/.claude/statusline-command.sh (auto-detected default)
#   3. Built-in minimal fallback below
next_script="${CLAUDE_HUB_STATUSLINE_NEXT:-}"
if [ -z "$next_script" ] && [ -f "$HOME/.claude/statusline-command.sh" ]; then
    next_script="$HOME/.claude/statusline-command.sh"
fi

if [ -n "$next_script" ] && [ -x "$next_script" ]; then
    printf '%s' "$input" | "$next_script"
elif [ -n "$next_script" ]; then
    printf '%s' "$input" | bash "$next_script"
else
    FALLBACK_SCRIPT='
import json, sys
try:
    d = json.load(sys.stdin)
except Exception:
    sys.exit(0)
model = (d.get("model") or {}).get("display_name", "")
cwd = (d.get("workspace") or {}).get("current_dir") or d.get("cwd", "")
pct = (d.get("context_window") or {}).get("used_percentage")
parts_path = [p for p in cwd.replace("\\", "/").split("/") if p]
short = "/".join(parts_path[-2:]) if parts_path else "?"
out = [model, short]
if pct is not None:
    out.append(f"ctx:{round(float(pct))}%")
print("  ".join(p for p in out if p))
'
    printf '%s' "$input" | python3 -c "$FALLBACK_SCRIPT"
fi
