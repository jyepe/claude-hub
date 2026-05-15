# claude-hub statusline wrapper (Windows / PowerShell edition).
#
# Install: in ~/.claude/settings.json set
#   "statusLine": {
#     "type": "command",
#     "command": "powershell -NoProfile -ExecutionPolicy Bypass -File \"<absolute path to this file>\""
#   }
# Optionally set $env:CLAUDE_HUB_STATUSLINE_NEXT to your previous statusline command;
# its stdout is forwarded as the rendered status line. If unset, a minimal fallback
# is printed so the bar doesn't go blank.
#
# Side effect: writes ~/.claude-hub/ctx-cache/{session_id}.json on each tick so the
# hub UI can report the same context% Claude Code displays.
#
# Why this exists: see GitHub issue #8 — the bash wrapper orphans bash.exe children
# on Windows. PowerShell does the same work with zero subprocesses.

$ErrorActionPreference = 'SilentlyContinue'
trap { exit 0 }


# 1. Read stdin
$rawInput = [Console]::In.ReadToEnd()
if ([string]::IsNullOrEmpty($rawInput)) { exit 0 }

# 2. Resolve paths
$hubDir = if ($env:CLAUDE_HUB_DIR) { $env:CLAUDE_HUB_DIR } else { Join-Path $HOME '.claude-hub' }
$cacheDir = Join-Path $hubDir 'ctx-cache'
try { New-Item -ItemType Directory -Path $cacheDir -Force | Out-Null } catch {}

# 3. Parse payload and write cache file (best-effort)
$payload = $null
try { $payload = $rawInput | ConvertFrom-Json } catch { $payload = $null }

if ($payload) {
    $sid = $payload.session_id
    if (-not $sid) { $sid = $payload.sessionId }
    if ($sid) {
        $cwd = $payload.cwd
        if (-not $cwd -and $payload.workspace) { $cwd = $payload.workspace.current_dir }

        $entry = [ordered]@{
            session_id     = $sid
            cwd            = $cwd
            model          = $payload.model
            context_window = $payload.context_window
            updated_at_ms  = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
        }

        $tmpName = ".claude-hub-$([System.Guid]::NewGuid().ToString('N'))"
        $tmpPath = Join-Path $cacheDir $tmpName
        $finalPath = Join-Path $cacheDir ($sid + '.json')

        try {
            $json = $entry | ConvertTo-Json -Compress -Depth 8
            [System.IO.File]::WriteAllText($tmpPath, $json, [System.Text.UTF8Encoding]::new($false))
            Move-Item -LiteralPath $tmpPath -Destination $finalPath -Force
        } catch {
            try { Remove-Item -LiteralPath $tmpPath -Force } catch {}
        }
    }
}

# 4. Forward to next statusline if configured, else built-in fallback
$nextScript = $env:CLAUDE_HUB_STATUSLINE_NEXT
if (-not $nextScript) {
    $candidate = Join-Path $HOME '.claude\statusline-command.ps1'
    if (Test-Path -LiteralPath $candidate) { $nextScript = $candidate }
}

if ($nextScript) {
    try {
        $rawInput | & $nextScript
    } catch {
        # Forwarding failed — fall through to built-in fallback so the bar isn't blank
        $nextScript = $null
    }
}

if (-not $nextScript) {
    if (-not $payload) { exit 0 }
    $model = $null
    if ($payload.model) { $model = $payload.model.display_name }
    $cwdFallback = $payload.cwd
    if (-not $cwdFallback -and $payload.workspace) { $cwdFallback = $payload.workspace.current_dir }
    if (-not $cwdFallback) { $cwdFallback = '' }

    $parts = $cwdFallback -split '[\\/]' | Where-Object { $_ -ne '' }
    $short = if ($parts.Count -ge 2) { ($parts[-2], $parts[-1]) -join '/' } elseif ($parts.Count -eq 1) { $parts[0] } else { '?' }

    $pct = $null
    if ($payload.context_window) { $pct = $payload.context_window.used_percentage }

    $out = @($model, $short)
    if ($null -ne $pct) {
        $out += "ctx:$([math]::Round([double]$pct))%"
    }
    ($out | Where-Object { $_ -and $_ -ne '' }) -join '  '
}
