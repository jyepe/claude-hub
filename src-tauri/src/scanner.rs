use crate::cache;
use crate::paths;
use crate::sessions::{parse_session, Session};
use std::collections::HashSet;
use std::path::Path;
use walkdir::WalkDir;

pub fn scan_all() -> Vec<Session> {
    let projects_dir = match paths::claude_projects_dir() {
        Some(p) if p.exists() => p,
        _ => return Vec::new(),
    };
    let cache_dir = paths::hub_cache_dir().unwrap_or_else(|| projects_dir.join(".cache"));
    let _ = std::fs::create_dir_all(&cache_dir);

    let mut out = Vec::new();
    for entry in WalkDir::new(&projects_dir)
        .min_depth(2)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
            continue;
        }
        match parse_one(path, &cache_dir) {
            Some(s) => out.push(s),
            None => continue,
        }
    }
    let bg_ids = read_bg_agent_ids();
    for s in &mut out {
        s.is_bg_agent = bg_ids.contains(&s.id);
    }
    out
}

fn parse_one(jsonl: &Path, cache_dir: &Path) -> Option<Session> {
    if let Some(s) = cache::read(cache_dir, jsonl) {
        return Some(s);
    }
    let contents = std::fs::read_to_string(jsonl).ok()?;
    let session = parse_session(jsonl, &contents);
    let _ = cache::write(cache_dir, jsonl, &session);
    Some(session)
}

fn read_bg_agent_ids() -> HashSet<String> {
    let path = match paths::claude_daemon_roster_path() {
        Some(p) => p,
        None => return HashSet::new(),
    };
    parse_bg_agent_ids_from_path(&path)
}

fn parse_bg_agent_ids_from_path(path: &std::path::Path) -> HashSet<String> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(_) => return HashSet::new(),
    };
    parse_bg_agent_ids_from_str(&text)
}

fn parse_bg_agent_ids_from_str(text: &str) -> HashSet<String> {
    let v: serde_json::Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(_) => return HashSet::new(),
    };
    let workers = match v.get("workers").and_then(|w| w.as_object()) {
        Some(w) => w,
        None => return HashSet::new(),
    };
    let mut ids = HashSet::new();
    for (_, worker) in workers {
        // Regular interactive sessions use mode "prompt" — skip them.
        // Background agents use a different mode (e.g. "agent", "task") or have no mode.
        let mode = worker
            .get("dispatch")
            .and_then(|d| d.get("launch"))
            .and_then(|l| l.get("mode"))
            .and_then(|m| m.as_str());
        if mode == Some("prompt") {
            continue;
        }
        if let Some(sid) = worker.get("sessionId").and_then(|s| s.as_str()) {
            if looks_like_uuid(sid) {
                ids.insert(sid.to_string());
            }
        }
        // Resumed bg agents store the original session file path in dispatch.launch.sessionId
        if let Some(launch_sid) = worker
            .get("dispatch")
            .and_then(|d| d.get("launch"))
            .and_then(|l| l.get("sessionId"))
            .and_then(|s| s.as_str())
        {
            if looks_like_uuid(launch_sid) {
                ids.insert(launch_sid.to_string());
            } else if let Some(id) = uuid_from_jsonl_path(launch_sid) {
                ids.insert(id);
            }
        }
    }
    ids
}

fn uuid_from_jsonl_path(s: &str) -> Option<String> {
    let name = s.rsplit(['/', '\\']).next()?;
    let stem = name.strip_suffix(".jsonl")?;
    if looks_like_uuid(stem) { Some(stem.to_string()) } else { None }
}

fn looks_like_uuid(s: &str) -> bool {
    if s.len() != 36 {
        return false;
    }
    let bytes = s.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        match i {
            8 | 13 | 18 | 23 => {
                if b != b'-' {
                    return false;
                }
            }
            _ => {
                if !b.is_ascii_hexdigit() {
                    return false;
                }
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_all_does_not_panic_when_dirs_missing() {
        let _ = scan_all();
    }

    #[test]
    fn parse_one_returns_session_for_unreadable_lines() {
        use std::io::Write;
        let tmp = std::env::temp_dir().join(format!(
            "claude-hub-scanner-{}-{}.jsonl",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut f = std::fs::File::create(&tmp).unwrap();
        writeln!(f, "not json").unwrap();
        writeln!(f, "{{\"broken\":").unwrap();
        let cache = std::env::temp_dir().join("claude-hub-scanner-cache");
        let _ = std::fs::create_dir_all(&cache);
        let s = parse_one(&tmp, &cache).unwrap();
        assert_eq!(s.tokens, 0);
    }

    #[test]
    fn regular_interactive_session_excluded_from_bg_agents() {
        // Real roster format: interactive sessions have dispatch.launch.mode = "prompt"
        let json = r#"{
            "workers": {
                "f5ec5e6f": {
                    "sessionId": "f5ec5e6f-84b2-4375-8ed8-a47fcab8ae24",
                    "dispatch": { "launch": { "mode": "prompt" } }
                }
            }
        }"#;
        let ids = parse_bg_agent_ids_from_str(json);
        assert!(ids.is_empty());
    }

    #[test]
    fn bg_agent_session_is_detected() {
        // Background agents use a mode other than "prompt"
        let json = r#"{
            "workers": {
                "0b36e159": {
                    "sessionId": "0b36e159-8022-444a-a9f7-164faaa78e49",
                    "dispatch": { "launch": { "mode": "agent" } }
                }
            }
        }"#;
        let ids = parse_bg_agent_ids_from_str(json);
        assert!(ids.contains("0b36e159-8022-444a-a9f7-164faaa78e49"));
    }

    #[test]
    fn worker_without_mode_treated_as_bg_agent() {
        // If mode is absent, conservatively treat it as a bg agent
        let json = r#"{"workers":{"0b36e159":{"sessionId":"0b36e159-8022-444a-a9f7-164faaa78e49","dispatch":{}}}}"#;
        let ids = parse_bg_agent_ids_from_str(json);
        assert!(ids.contains("0b36e159-8022-444a-a9f7-164faaa78e49"));
    }

    #[test]
    fn parse_bg_agent_ids_extracts_id_from_jsonl_path() {
        // Resumed bg sessions store the original session path in dispatch.launch.sessionId
        let json = r#"{"workers":{"c48dbaf9":{"sessionId":"c48dbaf9-4c87-4c76-91c0-8e20a8de849a","dispatch":{"launch":{"mode":"resume","sessionId":"C:\\Users\\foo\\.claude\\projects\\bar\\1ed07f96-9cff-4d93-a7ca-d5d638aad040.jsonl"}}}}}"#;
        let ids = parse_bg_agent_ids_from_str(json);
        assert!(ids.contains("1ed07f96-9cff-4d93-a7ca-d5d638aad040"));
    }

    #[test]
    fn parse_bg_agent_ids_ignores_no_workers_key() {
        // JSON without a "workers" object yields nothing
        let json = r#"{"status":"running","id":"0b36e159-8022-444a-a9f7-164faaa78e49"}"#;
        let ids = parse_bg_agent_ids_from_str(json);
        assert!(ids.is_empty());
    }

    #[test]
    fn parse_bg_agent_ids_returns_empty_for_bad_json() {
        let ids = parse_bg_agent_ids_from_str("not json at all");
        assert!(ids.is_empty());
    }

    #[test]
    fn parse_bg_agent_ids_returns_empty_for_missing_file() {
        let ids = parse_bg_agent_ids_from_path(&std::env::temp_dir().join("__nonexistent_roster_for_test.json"));
        assert!(ids.is_empty());
    }
}
