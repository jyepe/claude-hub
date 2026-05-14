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
    pub last_activity: Option<DateTime<Utc>>,
}

#[derive(Debug, Default)]
struct Acc {
    seen_uuids: HashSet<String>,
    cwd: Option<String>,
    title: Option<String>,
    model: Option<String>,
    message_count: u32,
    tokens: u64,
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
        last_activity: acc.last_activity,
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
    if let Some(ts) = v.get("timestamp").and_then(|t| t.as_str()) {
        if let Ok(parsed) = DateTime::parse_from_rfc3339(ts) {
            let utc = parsed.with_timezone(&Utc);
            acc.last_activity = Some(match acc.last_activity {
                Some(prev) if prev > utc => prev,
                _ => utc,
            });
        }
    }
    let typ = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
    match typ {
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
            }
        }
        _ => {}
    }
}

fn extract_user_text(v: &serde_json::Value) -> Option<String> {
    let content = v.get("message").and_then(|m| m.get("content"))?;
    if let Some(s) = content.as_str() {
        return Some(s.to_string());
    }
    if let Some(arr) = content.as_array() {
        for block in arr {
            if let Some(t) = block.get("text").and_then(|t| t.as_str()) {
                return Some(t.to_string());
            }
        }
    }
    None
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
        assert_eq!(s.message_count, 1);
        assert_eq!(s.model.as_deref(), Some("claude-opus-4-7"));
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
    fn skips_malformed_lines_without_panicking() {
        let contents = "not json\n{\"broken\":\n";
        let s = parse_session(&pp(), contents);
        assert_eq!(s.tokens, 0);
    }
}
