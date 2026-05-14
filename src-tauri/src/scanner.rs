use crate::cache;
use crate::paths;
use crate::sessions::{parse_session, Session};
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

pub fn read_bg_agent_ids() -> std::collections::HashSet<String> {
    let path = match paths::claude_daemon_roster_path() {
        Some(p) => p,
        None => return std::collections::HashSet::new(),
    };
    parse_bg_agent_ids_from_path(&path)
}

fn parse_bg_agent_ids_from_path(path: &std::path::Path) -> std::collections::HashSet<String> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(_) => return std::collections::HashSet::new(),
    };
    parse_bg_agent_ids_from_str(&text)
}

fn parse_bg_agent_ids_from_str(text: &str) -> std::collections::HashSet<String> {
    let v: serde_json::Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(_) => return std::collections::HashSet::new(),
    };
    let mut ids = std::collections::HashSet::new();
    collect_ids(&v, &mut ids);
    ids
}

fn collect_ids(v: &serde_json::Value, out: &mut std::collections::HashSet<String>) {
    match v {
        serde_json::Value::String(s) if is_session_id(s) => {
            out.insert(s.clone());
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                collect_ids(item, out);
            }
        }
        serde_json::Value::Object(map) => {
            for (_, val) in map {
                collect_ids(val, out);
            }
        }
        _ => {}
    }
}

fn is_session_id(s: &str) -> bool {
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
    fn parse_bg_agent_ids_handles_array_of_strings() {
        let json = r#"["0b36e159-8022-444a-a9f7-164faaa78e49","aaaabbbb-cccc-dddd-eeee-ffffffffffff"]"#;
        let ids = parse_bg_agent_ids_from_str(json);
        assert!(ids.contains("0b36e159-8022-444a-a9f7-164faaa78e49"));
        assert!(ids.contains("aaaabbbb-cccc-dddd-eeee-ffffffffffff"));
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn parse_bg_agent_ids_handles_nested_object() {
        let json = r#"{"sessions":[{"id":"0b36e159-8022-444a-a9f7-164faaa78e49"}]}"#;
        let ids = parse_bg_agent_ids_from_str(json);
        assert!(ids.contains("0b36e159-8022-444a-a9f7-164faaa78e49"));
    }

    #[test]
    fn parse_bg_agent_ids_ignores_non_uuid_strings() {
        let json = r#"{"status":"running","id":"0b36e159-8022-444a-a9f7-164faaa78e49"}"#;
        let ids = parse_bg_agent_ids_from_str(json);
        assert!(ids.contains("0b36e159-8022-444a-a9f7-164faaa78e49"));
        assert!(!ids.contains("running"));
    }

    #[test]
    fn parse_bg_agent_ids_returns_empty_for_bad_json() {
        let ids = parse_bg_agent_ids_from_str("not json at all");
        assert!(ids.is_empty());
    }

    #[test]
    fn parse_bg_agent_ids_returns_empty_for_missing_file() {
        let ids = parse_bg_agent_ids_from_path(std::path::Path::new("/nonexistent/roster.json"));
        assert!(ids.is_empty());
    }
}
