use crate::active_sessions::{self, LiveProcess};
use crate::cache;
use crate::paths;
use crate::sessions::{parse_session, Session};
use std::collections::HashMap;
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
    let live = active_sessions::read_all();
    apply_live_status_overlay(&mut out, &live);
    out
}

fn apply_live_status_overlay(sessions: &mut [Session], live: &HashMap<String, LiveProcess>) {
    for s in sessions {
        s.live_status = live.get(&s.id).map(|p| p.status);
    }
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
    let jobs_dir = match paths::claude_jobs_dir() {
        Some(p) if p.exists() => p,
        _ => return HashSet::new(),
    };
    let entries = match std::fs::read_dir(&jobs_dir) {
        Ok(e) => e,
        Err(_) => return HashSet::new(),
    };
    let mut ids = HashSet::new();
    for entry in entries.filter_map(|e| e.ok()) {
        collect_ids_from_job_state(&entry.path().join("state.json"), &mut ids);
    }
    ids
}

fn collect_ids_from_job_state(path: &Path, ids: &mut HashSet<String>) {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(_) => return,
    };
    let v: serde_json::Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(_) => return,
    };
    if let Some(sid) = v.get("sessionId").and_then(|s| s.as_str()) {
        if looks_like_uuid(sid) {
            ids.insert(sid.to_string());
        }
    }
    // For resumed sessions, linkScanPath points to the original JSONL the agent is writing to
    if let Some(link) = v.get("linkScanPath").and_then(|s| s.as_str()) {
        if let Some(id) = uuid_from_jsonl_path(link) {
            ids.insert(id);
        }
    }
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

    fn write_temp_state(name: &str, content: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("claude-hub-job-state-{}.json", name));
        std::fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn job_state_session_id_is_collected() {
        let path = write_temp_state(
            "session-id",
            r#"{"sessionId":"0b36e159-8022-444a-a9f7-164faaa78e49","state":"running"}"#,
        );
        let mut ids = HashSet::new();
        collect_ids_from_job_state(&path, &mut ids);
        assert!(ids.contains("0b36e159-8022-444a-a9f7-164faaa78e49"));
    }

    #[test]
    fn job_state_link_scan_path_is_collected_for_resumed_sessions() {
        let path = write_temp_state(
            "link-scan",
            r#"{"sessionId":"c48dbaf9-4c87-4c76-91c0-8e20a8de849a","linkScanPath":"C:\\Users\\foo\\.claude\\projects\\bar\\1ed07f96-9cff-4d93-a7ca-d5d638aad040.jsonl"}"#,
        );
        let mut ids = HashSet::new();
        collect_ids_from_job_state(&path, &mut ids);
        assert!(ids.contains("c48dbaf9-4c87-4c76-91c0-8e20a8de849a"));
        assert!(ids.contains("1ed07f96-9cff-4d93-a7ca-d5d638aad040"));
    }

    #[test]
    fn job_state_missing_file_does_not_panic() {
        let mut ids = HashSet::new();
        collect_ids_from_job_state(
            &std::env::temp_dir().join("__nonexistent_state_for_test.json"),
            &mut ids,
        );
        assert!(ids.is_empty());
    }

    #[test]
    fn job_state_bad_json_does_not_panic() {
        let path = write_temp_state("bad-json", "not json at all");
        let mut ids = HashSet::new();
        collect_ids_from_job_state(&path, &mut ids);
        assert!(ids.is_empty());
    }

    #[test]
    fn live_status_overlay_marks_matching_sessions() {
        use crate::active_sessions::{LiveProcess, LiveStatus};
        use std::collections::HashMap;

        let mut sessions = vec![
            Session {
                id: "alive-id".into(),
                jsonl_path: String::new(), cwd: None, title: None, model: None,
                message_count: 0, tokens: 0, context_tokens: 0, max_prompt_tokens: 0,
                last_activity: None, live_context_window: None, live_model_id: None,
                is_bg_agent: false, live_status: None,
            },
            Session {
                id: "closed-id".into(),
                jsonl_path: String::new(), cwd: None, title: None, model: None,
                message_count: 0, tokens: 0, context_tokens: 0, max_prompt_tokens: 0,
                last_activity: None, live_context_window: None, live_model_id: None,
                is_bg_agent: false, live_status: None,
            },
        ];
        let mut live = HashMap::new();
        live.insert(
            "alive-id".to_string(),
            LiveProcess { pid: 1, status: LiveStatus::Busy, session_id: "alive-id".into() },
        );
        apply_live_status_overlay(&mut sessions, &live);

        assert_eq!(sessions[0].live_status, Some(LiveStatus::Busy));
        assert_eq!(sessions[1].live_status, None);
    }
}
