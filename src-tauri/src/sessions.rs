use crate::active_sessions::LiveStatus;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub jsonl_path: String,
    pub cwd: Option<String>,
    pub title: Option<String>,
    pub model: Option<String>,
    pub message_count: u32,
    pub tokens: u64,
    pub context_tokens: u64,
    pub max_prompt_tokens: u64,
    pub last_activity: Option<DateTime<Utc>>,
    /// Window resolved from the statusline-cache wrapper. When present, the UI
    /// uses this verbatim instead of inferring a window from the model name.
    pub live_context_window: Option<u64>,
    /// Model ID as reported by CC at the statusline tick (may include the
    /// `[1m]` suffix the JSONL drops). Display-only.
    pub live_model_id: Option<String>,
    /// True when this session ID appears in ~/.claude/daemon/roster.json —
    /// i.e., a bg agent is currently running and can be attached to.
    /// Always `false` when deserialized from cache; overridden by `scanner::scan_all` using the live roster.
    #[serde(default)]
    pub is_bg_agent: bool,
    /// Status from `~/.claude/sessions/{pid}.json` when a live CC process is
    /// attached to this session. `None` means closed (no live process).
    /// Always `None` when deserialized from cache; overlayed by `scanner::scan_all`.
    #[serde(default)]
    pub live_status: Option<LiveStatus>,
    /// State of the bg agent from `~/.claude/jobs/<id>/state.json`.
    /// `None` for non-bg sessions. Always `None` when deserialized from cache;
    /// overlayed by `scanner::scan_all`.
    #[serde(default)]
    pub bg_state: Option<String>,
    /// Free-text detail line from the bg agent's `state.json`. See `bg_state` for the overlay contract.
    #[serde(default)]
    pub bg_detail: Option<String>,
    /// Tempo (e.g. "idle") from the bg agent's `state.json`. See `bg_state` for the overlay contract.
    #[serde(default)]
    pub bg_tempo: Option<String>,
    /// User intent / slash command that spawned this bg agent. See `bg_state` for the overlay contract.
    #[serde(default)]
    pub bg_intent: Option<String>,
    /// Display name from the bg agent's `state.json` (auto or user-set). See `bg_state` for the overlay contract.
    #[serde(default)]
    pub bg_name: Option<String>,
}

#[derive(Debug, Default)]
struct Acc {
    seen_uuids: HashSet<String>,
    cwd: Option<String>,
    title: Option<String>,
    model: Option<String>,
    message_count: u32,
    tokens: u64,
    context_tokens: u64,
    max_prompt_tokens: u64,
    latest_assistant_ts: Option<DateTime<Utc>>,
    last_activity: Option<DateTime<Utc>>,
    session_id: Option<String>,
}

pub fn parse_session(jsonl_path: &Path, contents: &str) -> Session {
    let mut acc = Acc::default();
    for line in contents.lines() {
        let v: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        absorb(&mut acc, &v);
    }

    let id = acc
        .session_id
        .or_else(|| {
            jsonl_path
                .file_stem()
                .and_then(|s| s.to_str())
                .map(String::from)
        })
        .unwrap_or_else(|| "unknown".into());

    Session {
        id,
        jsonl_path: jsonl_path.to_string_lossy().into_owned(),
        cwd: acc.cwd,
        title: acc.title,
        model: acc.model,
        message_count: acc.message_count,
        tokens: acc.tokens,
        context_tokens: acc.context_tokens,
        max_prompt_tokens: acc.max_prompt_tokens,
        last_activity: acc.last_activity,
        live_context_window: None,
        live_model_id: None,
        is_bg_agent: false,
        live_status: None,
        bg_state: None,
        bg_detail: None,
        bg_tempo: None,
        bg_intent: None,
        bg_name: None,
    }
}

fn absorb(acc: &mut Acc, v: &serde_json::Value) {
    if let Some(uuid) = v.get("uuid").and_then(|u| u.as_str()) {
        if !acc.seen_uuids.insert(uuid.to_string()) {
            return;
        }
    }
    if acc.session_id.is_none() {
        if let Some(s) = v.get("sessionId").and_then(|s| s.as_str()) {
            acc.session_id = Some(s.to_string());
        }
    }
    if acc.cwd.is_none() {
        if let Some(c) = v.get("cwd").and_then(|c| c.as_str()) {
            acc.cwd = Some(c.to_string());
        }
    }
    let event_ts = v
        .get("timestamp")
        .and_then(|t| t.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|p| p.with_timezone(&Utc));
    if let Some(utc) = event_ts {
        acc.last_activity = Some(match acc.last_activity {
            Some(prev) if prev > utc => prev,
            _ => utc,
        });
    }
    let typ = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
    match typ {
        "ai-title" => {
            if let Some(t) = v.get("aiTitle").and_then(|t| t.as_str()) {
                acc.title = Some(t.to_string());
            }
        }
        "user" => {
            acc.message_count += 1;
            if acc.title.is_none() {
                acc.title = extract_user_text(v).map(|s| truncate(&s, 80));
            }
        }
        "assistant" => {
            acc.message_count += 1;
            if acc.model.is_none() {
                if let Some(m) = v
                    .get("message")
                    .and_then(|m| m.get("model"))
                    .and_then(|m| m.as_str())
                {
                    acc.model = Some(m.to_string());
                }
            }
            if let Some(usage) = v.get("message").and_then(|m| m.get("usage")) {
                for k in &[
                    "input_tokens",
                    "cache_creation_input_tokens",
                    "cache_read_input_tokens",
                    "output_tokens",
                ] {
                    if let Some(n) = usage.get(*k).and_then(|n| n.as_u64()) {
                        acc.tokens += n;
                    }
                }
                let prompt: u64 = ["input_tokens", "cache_creation_input_tokens", "cache_read_input_tokens"]
                    .iter()
                    .filter_map(|k| usage.get(*k).and_then(|n| n.as_u64()))
                    .sum();
                if prompt > acc.max_prompt_tokens {
                    acc.max_prompt_tokens = prompt;
                }
                // Context-window fill = prompt size of the latest assistant turn.
                // Excludes output_tokens (those are produced by the call, not part of its prompt).
                // For files without timestamps, last-seen wins.
                let is_latest = match (event_ts, acc.latest_assistant_ts) {
                    (Some(ts), Some(prev)) => ts >= prev,
                    (None, None) => true,
                    (Some(_), None) => true,
                    (None, Some(_)) => false,
                };
                if is_latest {
                    acc.context_tokens = prompt;
                    acc.latest_assistant_ts = event_ts.or(acc.latest_assistant_ts);
                }
            }
        }
        _ => {}
    }
}

fn extract_user_text(v: &serde_json::Value) -> Option<String> {
    let content = v.get("message").and_then(|m| m.get("content"))?;
    let raw = if let Some(s) = content.as_str() {
        s.to_string()
    } else if let Some(arr) = content.as_array() {
        let mut found = None;
        for block in arr {
            if let Some(t) = block.get("text").and_then(|t| t.as_str()) {
                found = Some(t.to_string());
                break;
            }
        }
        found?
    } else {
        return None;
    };
    let stripped = strip_xml_tags(&raw);
    if stripped.is_empty() { None } else { Some(stripped) }
}

fn strip_xml_tags(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '<' {
            let next = chars.get(i + 1).copied();
            let looks_like_tag = matches!(next, Some(ch) if ch.is_ascii_alphabetic() || ch == '/');
            if looks_like_tag {
                if let Some(off) = chars[i + 1..].iter().position(|&c| c == '>') {
                    i += off + 2;
                    continue;
                }
            }
            out.push(c);
            i += 1;
        } else {
            out.push(c);
            i += 1;
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(n).collect();
        out.push('…');
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn pp() -> PathBuf {
        PathBuf::from("/tmp/abc.jsonl")
    }

    #[test]
    fn dedupes_assistant_tokens_by_uuid() {
        let line = r#"{"uuid":"u1","type":"assistant","timestamp":"2026-05-09T03:55:16.638Z","message":{"model":"claude-opus-4-7","usage":{"input_tokens":10,"cache_creation_input_tokens":5,"cache_read_input_tokens":100,"output_tokens":7}}}"#;
        let contents = format!("{}\n{}\n", line, line);
        let s = parse_session(&pp(), &contents);
        assert_eq!(s.tokens, 122);
        assert_eq!(s.context_tokens, 115);
        assert_eq!(s.max_prompt_tokens, 115);
        assert_eq!(s.message_count, 1);
        assert_eq!(s.model.as_deref(), Some("claude-opus-4-7"));
    }

    #[test]
    fn max_prompt_tokens_tracks_peak_across_turns() {
        // Turn 1: small prompt.
        let t1 = r#"{"uuid":"a1","type":"assistant","timestamp":"2026-05-09T03:55:00Z","message":{"usage":{"input_tokens":10,"cache_creation_input_tokens":0,"cache_read_input_tokens":0}}}"#;
        // Turn 2: huge prompt — bigger than 200k.
        let t2 = r#"{"uuid":"a2","type":"assistant","timestamp":"2026-05-09T03:56:00Z","message":{"usage":{"input_tokens":1,"cache_creation_input_tokens":0,"cache_read_input_tokens":250000}}}"#;
        // Turn 3 (latest by ts): small again — context_tokens drops but max persists.
        let t3 = r#"{"uuid":"a3","type":"assistant","timestamp":"2026-05-09T03:57:00Z","message":{"usage":{"input_tokens":2,"cache_creation_input_tokens":0,"cache_read_input_tokens":500}}}"#;
        let s = parse_session(&pp(), &format!("{}\n{}\n{}\n", t1, t2, t3));
        assert_eq!(s.context_tokens, 502);
        assert_eq!(s.max_prompt_tokens, 250001);
    }

    #[test]
    fn context_tokens_reflect_only_latest_assistant_turn() {
        // Turn 1: small prompt, big cache creation.
        let turn1 = r#"{"uuid":"a1","type":"assistant","timestamp":"2026-05-09T03:55:00Z","message":{"model":"claude-opus-4-7","usage":{"input_tokens":10,"cache_creation_input_tokens":50000,"cache_read_input_tokens":0,"output_tokens":200}}}"#;
        // Turn 2: cache read of the whole context.
        let turn2 = r#"{"uuid":"a2","type":"assistant","timestamp":"2026-05-09T03:56:00Z","message":{"model":"claude-opus-4-7","usage":{"input_tokens":300,"cache_creation_input_tokens":0,"cache_read_input_tokens":50000,"output_tokens":400}}}"#;
        // Turn 3 (latest): context has grown.
        let turn3 = r#"{"uuid":"a3","type":"assistant","timestamp":"2026-05-09T03:57:00Z","message":{"model":"claude-opus-4-7","usage":{"input_tokens":500,"cache_creation_input_tokens":1000,"cache_read_input_tokens":52000,"output_tokens":700}}}"#;
        let s = parse_session(&pp(), &format!("{}\n{}\n{}\n", turn1, turn2, turn3));
        // Lifetime sum: 50210 + 50700 + 54200 = 155110
        assert_eq!(s.tokens, 50210 + 50700 + 54200);
        // Context = latest turn's prompt size (input + cache_creation + cache_read), no output.
        assert_eq!(s.context_tokens, 500 + 1000 + 52000);
    }

    #[test]
    fn context_tokens_use_latest_timestamp_not_file_order() {
        // File order out-of-order; latest by timestamp should win.
        let later = r#"{"uuid":"a2","type":"assistant","timestamp":"2026-05-09T04:00:00Z","message":{"usage":{"input_tokens":1,"cache_creation_input_tokens":2,"cache_read_input_tokens":3,"output_tokens":99}}}"#;
        let earlier = r#"{"uuid":"a1","type":"assistant","timestamp":"2026-05-09T03:00:00Z","message":{"usage":{"input_tokens":100,"cache_creation_input_tokens":200,"cache_read_input_tokens":300,"output_tokens":99}}}"#;
        let s = parse_session(&pp(), &format!("{}\n{}\n", later, earlier));
        assert_eq!(s.context_tokens, 1 + 2 + 3);
    }

    #[test]
    fn picks_first_user_text_as_title() {
        let user1 = r#"{"uuid":"u1","type":"user","message":{"content":"hello world"}}"#;
        let user2 = r#"{"uuid":"u2","type":"user","message":{"content":"second"}}"#;
        let s = parse_session(&pp(), &format!("{}\n{}\n", user1, user2));
        assert_eq!(s.title.as_deref(), Some("hello world"));
        assert_eq!(s.message_count, 2);
    }

    #[test]
    fn captures_cwd_and_session_id() {
        let line = r#"{"uuid":"u1","type":"attachment","sessionId":"sess-123","cwd":"C:\\Users\\me\\proj","timestamp":"2026-05-09T03:55:16.638Z"}"#;
        let s = parse_session(&pp(), &format!("{}\n", line));
        assert_eq!(s.id, "sess-123");
        assert_eq!(s.cwd.as_deref(), Some("C:\\Users\\me\\proj"));
        assert!(s.last_activity.is_some());
    }

    #[test]
    fn falls_back_to_filename_for_id() {
        let line = r#"{"uuid":"u1","type":"user","message":{"content":"x"}}"#;
        let s = parse_session(
            &PathBuf::from("/tmp/0b36e159-8022-444a-a9f7-164faaa78e49.jsonl"),
            &format!("{}\n", line),
        );
        assert_eq!(s.id, "0b36e159-8022-444a-a9f7-164faaa78e49");
    }

    #[test]
    fn ai_title_overrides_first_user_message() {
        let user = r#"{"uuid":"u1","type":"user","message":{"content":"raw first message"}}"#;
        let title = r#"{"type":"ai-title","aiTitle":"Claude-generated title"}"#;
        let s = parse_session(&pp(), &format!("{}\n{}\n", user, title));
        assert_eq!(s.title.as_deref(), Some("Claude-generated title"));
    }

    #[test]
    fn strips_xml_tags_from_title() {
        let user = r#"{"uuid":"u1","type":"user","message":{"content":"<command-message>init</command-message> <command-name>/init</command-name>"}}"#;
        let s = parse_session(&pp(), &format!("{}\n", user));
        assert_eq!(s.title.as_deref(), Some("init /init"));
    }

    #[test]
    fn preserves_literal_less_than_when_not_a_tag() {
        let user = r#"{"uuid":"u1","type":"user","message":{"content":"if x < 10 then continue"}}"#;
        let s = parse_session(&pp(), &format!("{}\n", user));
        assert_eq!(s.title.as_deref(), Some("if x < 10 then continue"));
    }

    #[test]
    fn preserves_unclosed_bracket() {
        let user = r#"{"uuid":"u1","type":"user","message":{"content":"User said < hello"}}"#;
        let s = parse_session(&pp(), &format!("{}\n", user));
        assert_eq!(s.title.as_deref(), Some("User said < hello"));
    }

    #[test]
    fn skips_malformed_lines_without_panicking() {
        let contents = "not json\n{\"broken\":\n";
        let s = parse_session(&pp(), contents);
        assert_eq!(s.tokens, 0);
    }
}
