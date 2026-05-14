# Claude Hub — Planning Doc

A local-first "Mission Control" for every Claude Code session on your machine. One window that shows every project, every session, every MCP, every skill, and live context usage — with one-click launch into a fresh terminal at any of them.

---

## 1. Vision & differentiator

Existing tools in this space (lm-assist, claude-code-transcripts, claude-JSONL-browser) are **read-only viewers**. They let you inspect past sessions but don't let you _do anything_.

Claude Hub is a **launcher fused with a dashboard**. The mental model is more like Docker Desktop or GitHub Desktop than a log viewer — you live here between coding bursts.

The signature feature no existing tool has: a **system tray icon with live context-usage on your active session** (e.g. tooltip reads `Green Seasons · 47k / 200k (24%)` and updates in real time). You always know how close you are to auto-compaction without `/status`-ing.

---

## 2. Goals & non-goals

### Goals

- See every Claude Code session on this machine in one view, grouped by project.
- One-click launch into a terminal at any project (new session or resumed).
- Visibility into configured MCPs (global + per-project) and installed skills (user + project).
- Real-time updates as sessions accumulate tokens.
- Live tray indicator for the currently-active session.
- Cross-platform from day 1 (macOS, Linux, Windows).

### Non-goals (for v0.1)

- Editing transcripts.
- Cloud sync / multi-machine session sharing.
- Replacing the `claude` CLI — we _launch_ it, we don't reimplement it.
- Acting as an MCP server itself.
- Mobile.

---

## 3. Stack decision

**Tauri 2 + React + TypeScript + Tailwind.**

|Choice|Why|
|---|---|
|Tauri 2|Native OS webview (≈10 MB ship size vs Electron's ≈150 MB); Rust backend gives fast filesystem scans + native terminal spawning; built-in system tray, autostart, global hotkeys.|
|React + TS|Familiar territory; component model fits a dashboard with many panels.|
|Tailwind|Speed of iteration on a dense dashboard layout.|
|Rust crates|`serde_json` (JSONL parsing), `chrono` (timestamps), `notify` (file watcher), `dirs` (cross-platform home dir), `reqwest` (MCP health pings).|

**Alternatives considered & rejected:**

- _Electron_ — same architecture, much heavier. Worth it only if Rust comfort is a blocker.
- _Next.js in a browser tab_ — can't spawn terminals cleanly, can't sit in the menu bar.
- _Ink / TUI_ — fun, but loses dashboard density and visual context bars.

---

## 4. Data sources — where everything lives

The entire surface area of Claude Code's on-disk state. **No private APIs needed**, no auth, no network — this is the whole list.

|Path|Content|Notes|
|---|---|---|
|`~/.claude/projects/<encoded-cwd>/<session-uuid>.jsonl`|Per-session transcripts|Encoded folder name = absolute path with `/` replaced by `-`. Each line is a typed event: `user`, `assistant`, `tool_use`, `tool_result`, `thinking`, `summary`.|
|`~/.claude.json`|Global config + MCP servers|Both global `mcpServers` and per-project under `projects.<path>.mcpServers`.|
|`~/.claude/skills/<skill>/SKILL.md`|User-installed skills|YAML frontmatter with `name` + `description`.|
|`<project>/.claude/skills/<skill>/SKILL.md`|Project-scoped skills|Same format.|
|`<project>/.mcp.json`|Project MCP servers (in repo)|Checked into version control.|
|`~/.claude/CLAUDE.md`|Global memory|Plain markdown.|
|`<project>/CLAUDE.md`|Project memory|Plain markdown.|
|`~/.claude/history.jsonl`|Slash-command history|Across all sessions.|
|`~/.claude/sessions.idx`|Pre-built session index cache|May or may not be present; tab-separated metadata. We can build our own if missing.|
|`~/.claude/settings.json`|User settings|If we ever want to display "your defaults".|

### Critical JSONL parsing notes

- **Dedupe by UUID.** Claude Code writes the same event to multiple JSONLs during branching/resumption. Naive token summing inflates totals by 2–4×. Always `HashSet` the UUIDs.
- **Token usage** lives on `assistant`-type events under `message.usage`. The four keys to sum: `input_tokens`, `cache_creation_input_tokens`, `cache_read_input_tokens`, `output_tokens`.
- **Model** lives on `message.model` of assistant events. First one wins (sessions occasionally switch models mid-conversation).
- **Title** = first `user`-type event's `message.content` (string or first text block), truncated to ~80 chars.
- **Project path decoding** — the dash-encoded folder is _mostly_ reversible by replacing `-` with `/`, but breaks for paths with literal dashes in directory names. Probably fine for v0.1; revisit if it bites.

---

## 5. Feature scope — phased

Each phase is independently shippable. Don't move to the next phase until the previous one feels good.

### Phase 1 — Sessions & launcher (the core)

The minimum that justifies the app existing.

- Scan `~/.claude/projects/` recursively.
- Parse JSONL → Session model (id, project, title, model, tokens, msg count, last activity).
- Group sessions by decoded project path.
- Project card UI: name, count, last-touched timestamp, "Resume" + "New session" buttons.
- Expand a project card → list of its sessions, each with a context-usage bar and an "open" button.
- "Open session" → spawn terminal with `cd <project> && claude --resume <id>` (or just `claude` for new).

**Definition of done:** Can replace `cd ~/projects/foo && claude --resume <hunt-for-id>` with a click.

### Phase 2 — MCPs & skills panel

Read-only visibility.

- Parse `~/.claude.json` → list of MCPs with name, scope (global / project), transport (stdio/http/sse), command or URL, env _keys_ (never values).
- Scan `~/.claude/skills/` and (optionally) project `.claude/skills/` → list of skills with name + description from frontmatter.
- Render both in a right-side panel beside the project list.

**Definition of done:** No more digging through `~/.claude.json` to remember what MCPs are wired up.

### Phase 3 — Live updates

The dashboard becomes ambient.

- `notify` crate watches `~/.claude/projects/` recursively.
- New line appended to a JSONL → re-parse just that session → emit Tauri event → frontend updates the relevant card.
- New session file created → add card.
- Background re-aggregation of header stats.

**Definition of done:** Open the hub in one monitor, work in claude in another, watch the bars grow.

### Phase 4 — The signature feature: tray with live context

This is the differentiator. Without it we're another viewer.

- Detect "active" session = the JSONL most recently appended to (within last N minutes).
- Tray tooltip: `<project name> · <tokens>k / <window>k (<%>)`.
- Tray icon color shifts at thresholds (green < 50%, amber 50–80%, red > 80%).
- Click tray → focus the hub window.

**Definition of done:** I never have to type `/status` again.

### Phase 5 — Cost & insights

Quantitative layer.

- Per-session cost estimate using published Anthropic rates (per-model, per-token-type).
- Header tiles: "This week's tokens / cost" rollup.
- Per-project cost over time (simple sparkline).

### Phase 6 — Search

Discovery.

- Search across **session titles + metadata only** for v0.1 (decision §8.8). Cheap, no index needed.
- Results show: project, session title, matched snippet, jump-to-session button.
- Full-transcript search (ripgrep or `tantivy`) deferred — revisit only if title-search proves insufficient.

### Phase 7+ — Stretch (decide later)

See §10.

---

## 6. Cross-platform terminal spawning

The single trickiest cross-platform piece.

|OS|Approach|Fallback|
|---|---|---|
|macOS|AppleScript via `osascript` → `tell application "Terminal" to do script "..."`|iTerm variant if user prefers.|
|Linux|Try `gnome-terminal`, `konsole`, `xfce4-terminal`, `alacritty`, `kitty`, `wezterm` in order; first one in PATH wins.|Configurable override in settings.|
|Windows|`wt.exe -d <path> cmd /k claude ...`|`cmd /c start cmd /k ...` for pre-Win11.|

### Open question: terminal preference

Should the user be able to pick their preferred terminal in settings? (e.g. iTerm vs Terminal.app on Mac; `kitty` vs `alacritty` on Linux.) Probably yes by v0.2 — for v0.1, hardcode the default per platform.

---

## 7. Technical gotchas — known landmines

|Risk|Mitigation|
|---|---|
|Duplicate JSONL events inflating token counts|Dedupe by UUID before summing.|
|Project path decoding breaks on dirs with literal `-`|Accept the imperfection in v0.1; explore real reversible encoding (Claude Code may use base64 or hash in future).|
|Large JSONL files (long sessions) → slow parsing|Parse lazily; cache parsed metadata to a JSON sidecar keyed by JSONL mtime (see §8.10).|
|File watcher floods on rapid writes|Debounce events per-file to ~250ms.|
|MCP env values exposed in UI|**Never** display values. Show keys only. Treat `~/.claude.json` as containing secrets.|
|Tray-tooltip update on every JSONL line write is wasteful|Throttle to once per second.|
|Cross-platform path differences|Use Rust's `PathBuf` everywhere; only stringify at the I/O boundary.|
|Tauri 2 webview differences (WebView2 vs WebKit)|Stick to widely-supported CSS; test in both early.|
|Claude Code might change session format|Version-check the JSONL schema field if present; warn user if unknown.|

---

## 8. Resolved decisions

These were the live questions before v0.1 build kicked off. Each is now a committed call with a one-line rationale. Pure visual/design decisions still live in `design.md`.

### Behavior

1. **Window close → hide to tray; ⌘Q (Cmd-Q / Alt-F4) is the real quit.** The hub is meant to live in the background; close-as-quit defeats the tray indicator.
2. **Autostart on login → default on, with a settings toggle.** The tray feature only works if the app is running; default-on is the natural shape for an ambient tool.
3. **Active session detection → filesystem signal.** Most recently appended JSONL within the last ~5 minutes wins. Simpler and more reliable than process-walking; one source of truth.

### Scope

1. **Noise projects → filter by default + manual override.** Hide projects below a configurable session-count threshold (default e.g. 2). Right-click any project to explicitly hide it regardless. Both knobs live in user prefs.
2. **Refresh strategy (pre-Phase 3) → 30s interval poll + manual refresh button.** Live enough to be useful, cheap enough not to need the file watcher yet. Both are removed/replaced when Phase 3 ships.
3. **Project skills scan → lazy on expand.** Don't open every project's `.claude/skills/` at startup. Scan only when the user expands a project card. Cache the result per-project once scanned.
4. **`history.jsonl` → yes, surface as a tile.** A "most-used slash commands" tile fed by `~/.claude/history.jsonl`. Small, easy to compute, genuinely useful for noticing your own habits.
5. **Search scope → light (metadata + titles only) for v0.1.** No transcript index. Defer the ripgrep/tantivy question to Phase 6 — by then we'll know whether title-search is enough.

### Storage / state

1. **User prefs → flat JSON in `~/.claude-hub/`.** No dependency, no migration story, trivial to inspect with `cat`. Layout: `~/.claude-hub/prefs.json` for user prefs, `~/.claude-hub/cache/` for parsed-session sidecars. If we ever need a real search index or session-tags table, SQLite shows up _next to_ this folder as a separate concern.
2. **Cache parsed session metadata → yes, from v0.1.** Sidecar JSON files under `~/.claude-hub/cache/`, keyed by JSONL absolute path, validated against the source JSONL's mtime. If mtime matches, hydrate from cache; otherwise re-parse and rewrite the sidecar. This is the only reason re-launches will stay fast as session count grows.

### Architecture / data

1. **Worktrees → sub-group under parent project.** Detect a worktree by checking the directory's `.git` (file pointing to a gitdir, vs. directory). Group its sessions under the parent repo's card, labeled with the worktree's branch or directory name.

### Distribution

1. **Ship intent → undecided; build clean enough that public release is possible.** Don't sink v0.1 time into code-signing, auto-updates, icon polish, or marketplace listings. Defer the actual release work until the app is being used daily and the shape is stable.

---

## 9. Project structure

```
claude-hub/
├── src-tauri/                          # Rust backend
│   ├── src/
│   │   ├── main.rs                     # Tauri bootstrap
│   │   ├── lib.rs                      # Command + tray registration
│   │   ├── sessions.rs                 # JSONL scanner + parser
│   │   ├── cache.rs                    # ~/.claude-hub/cache sidecar I/O
│   │   ├── prefs.rs                    # ~/.claude-hub/prefs.json reader/writer
│   │   ├── terminal.rs                 # Cross-platform spawn
│   │   ├── mcp.rs                      # ~/.claude.json reader
│   │   ├── skills.rs                   # SKILL.md scanner
│   │   ├── stats.rs                    # Aggregates for header tiles
│   │   ├── worktree.rs                 # Detect .git-file worktrees, group under parent
│   │   ├── watcher.rs                  # notify → Tauri events  (Phase 3)
│   │   └── tray.rs                     # Live tooltip updater    (Phase 4)
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── icons/
├── src/                                # React frontend
│   ├── components/                     # See design.md
│   ├── lib/
│   │   ├── api.ts                      # Tauri invoke wrappers
│   │   ├── types.ts
│   │   └── format.ts                   # tokens, time-ago, paths
│   ├── App.tsx
│   ├── main.tsx
│   └── index.css
├── design.md                           # UI/UX spec (separate doc)
├── package.json
├── vite.config.ts
├── tsconfig.json
└── README.md
```

External state (created at first launch, lives in `$HOME`):

```
~/.claude-hub/
├── prefs.json                          # terminal choice, hidden projects, threshold, etc.
└── cache/
    └── <hash-of-jsonl-path>.json       # parsed session metadata, mtime-validated
```

---

## 10. Future ideas (post-v0.1, not committed)

Quick capture so we don't lose them.

- **Session tags + notes.** Annotate sessions with labels ("debugging RLS", "spike: dynamic pricing"). Searchable.
- **AI session summaries.** Use the Claude API to generate a 1-line summary of each session beyond the first user prompt. Maybe daily batch.
- **Cost dashboard.** Per-week/-month spend rollups, per-project, per-model breakdowns.
- **MCP health pings.** Fire `tools/list` at HTTP/SSE servers, green/red dot per MCP. Show last-error text on hover.
- **Skill enable/disable toggle.** Treat skills like a plugin manager — rename folder with `.disabled` suffix to opt out.
- **Quick prompt launcher.** Pre-fill a starter prompt and open a session ready to go. Could integrate with snippets.
- **Heatmap.** When-am-I-most-active-with-Claude calendar view.
- **Cross-machine sync.** Push session metadata (not transcripts — privacy) to Supabase, see your sessions from your laptop on your desktop and vice versa. Natural fit given you're already on Supabase.
- **Diff viewer.** What files did this session modify? Correlate `file-history-snapshot` records with edit tool calls.
- **Multi-agent visibility.** Some sessions spawn subagents. Show the parent-child tree.
- **CLAUDE.md editor.** Edit project + global memory in-app without leaving the hub.
- **MCP marketplace integration.** Connect to Anthropic's connector registry, one-click install MCPs.
- **Token budget alerts.** "You've used 80% of context in `greenseasons` — consider /compact."
- **Session resume by topic.** "Show me sessions where we worked on the truck-load summary RPC."
- **Full-transcript search.** Promote Phase 6 from light to heavy if title-search proves insufficient.
- **SQLite migration.** When tags/notes or a real search index lands, move from flat-JSON prefs to SQLite alongside.

---

## 11. Open-source angle (optional)

If you ever want to publish this:

- BSD-2 or MIT license — match the ecosystem.
- README with screenshots + a clear "this is not affiliated with Anthropic" disclaimer.
- Auto-update via Tauri's updater plugin against GitHub releases.
- Icon set — needs a real designer touch (or generate via gradient + iconography).

Per §8.12, none of this is v0.1 work — but keep the codebase in a state where it could be.

---

## 12. Definition of "v0.1 ships"

The MVP bar to call it done and start using it daily:

- [ ] Dashboard window opens, shows all projects from `~/.claude/projects/`.
- [ ] Each project shows session count + last-touched timestamp.
- [ ] Expand a project → see all its sessions with context-usage bars.
- [ ] "Resume" button spawns terminal at correct path with `claude --resume <id>` — works on the user's primary OS.
- [ ] "New session" button spawns terminal at correct path with `claude`.
- [ ] MCPs and skills panel renders on the right, populated from disk.
- [ ] Header stat tiles populated (projects / sessions / tokens 7d / tokens all-time).
- [ ] 30s interval auto-refresh + manual refresh button (true live updates come in Phase 3).
- [ ] Noise-project filter active by default; right-click to hide an individual project.
- [ ] Sidecar metadata cache at `~/.claude-hub/cache/` working (relaunches stay fast).
- [ ] Worktree sessions sub-grouped under their parent project.
- [ ] Close-to-tray + ⌘Q-to-quit behavior wired up.
- [ ] No crashes on missing/malformed JSONL.
- [ ] No secrets ever displayed in the MCP panel.

When all of those are checked, ship to self, dogfood for a week, then sequence Phase 2 onward.

---

## 13. Sequencing summary

```
Phase 1: Sessions & launcher          ← v0.1 ships here
Phase 2: MCPs & skills panel          ← polish + completeness
Phase 3: File watcher → live updates  ← ambient
Phase 4: Tray with live context       ← THE differentiator
Phase 5: Cost & insights
Phase 6: Search (titles + metadata, per §8.8)
Phase 7+: tags, AI summaries, sync, etc.
```
