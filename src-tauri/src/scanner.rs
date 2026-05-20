use crate::active_sessions::{self, LiveProcess};
use crate::cache;
use crate::paths;
use crate::sessions::{parse_session, Session};
use std::collections::HashMap;
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
    let bg_info = read_bg_agent_info();
    for s in &mut out {
        if let Some(info) = bg_info.get(&s.id) {
            s.is_bg_agent = true;
            s.bg_state = info.state.clone();
            s.bg_detail = info.detail.clone();
            s.bg_tempo = info.tempo.clone();
            s.bg_intent = info.intent.clone();
            s.bg_name = info.name.clone();
        }
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

#[derive(Debug, Clone, Default)]
pub(crate) struct BgAgentInfo {
    pub(crate) state: Option<String>,
    pub(crate) detail: Option<String>,
    pub(crate) tempo: Option<String>,
    pub(crate) intent: Option<String>,
    pub(crate) name: Option<String>,
}

fn collect_info_from_job_state(path: &Path, out: &mut HashMap<String, BgAgentInfo>) {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(_) => return,
    };
    let v: serde_json::Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(_) => return,
    };

    let info = BgAgentInfo {
        state: v.get("state").and_then(|s| s.as_str()).map(String::from),
        detail: v.get("detail").and_then(|s| s.as_str()).map(String::from),
        tempo: v.get("tempo").and_then(|s| s.as_str()).map(String::from),
        intent: v.get("intent").and_then(|s| s.as_str()).map(String::from),
        name: v.get("name").and_then(|s| s.as_str()).map(String::from),
    };

    if let Some(sid) = v.get("sessionId").and_then(|s| s.as_str()) {
        if looks_like_uuid(sid) {
            out.insert(sid.to_string(), info.clone());
        }
    }
    // For resumed sessions, linkScanPath points to the original JSONL the agent writes to.
    if let Some(link) = v.get("linkScanPath").and_then(|s| s.as_str()) {
        if let Some(id) = uuid_from_jsonl_path(link) {
            out.insert(id, info);
        }
    }
}

fn read_bg_agent_info() -> HashMap<String, BgAgentInfo> {
    let jobs_dir = match paths::claude_jobs_dir() {
        Some(p) if p.exists() => p,
        _ => return HashMap::new(),
    };
    let entries = match std::fs::read_dir(&jobs_dir) {
        Ok(e) => e,
        Err(_) => return HashMap::new(),
    };
    let mut map = HashMap::new();
    for entry in entries.filter_map(|e| e.ok()) {
        collect_info_from_job_state(&entry.path().join("state.json"), &mut map);
    }
    map
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
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "claude-hub-job-state-{}-{}-{}.json",
            name,
            std::process::id(),
            nanos
        ));
        std::fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn collect_info_extracts_all_fields() {
        let path = write_temp_state(
            "full-fields",
            r#"{
                "sessionId":"0b36e159-8022-444a-a9f7-164faaa78e49",
                "state":"running",
                "detail":"task in progress",
                "tempo":"idle",
                "intent":"/wiki what was the last job i applied to?",
                "name":"job application history"
            }"#,
        );
        let mut map = HashMap::new();
        collect_info_from_job_state(&path, &mut map);
        let info = map
            .get("0b36e159-8022-444a-a9f7-164faaa78e49")
            .expect("session id key present");
        assert_eq!(info.state.as_deref(), Some("running"));
        assert_eq!(info.detail.as_deref(), Some("task in progress"));
        assert_eq!(info.tempo.as_deref(), Some("idle"));
        assert_eq!(info.intent.as_deref(), Some("/wiki what was the last job i applied to?"));
        assert_eq!(info.name.as_deref(), Some("job application history"));
    }

    #[test]
    fn collect_info_handles_missing_optional_fields() {
        let path = write_temp_state(
            "only-session-id",
            r#"{"sessionId":"0b36e159-8022-444a-a9f7-164faaa78e49"}"#,
        );
        let mut map = HashMap::new();
        collect_info_from_job_state(&path, &mut map);
        let info = map
            .get("0b36e159-8022-444a-a9f7-164faaa78e49")
            .expect("session id key present");
        assert_eq!(info.state, None);
        assert_eq!(info.detail, None);
        assert_eq!(info.tempo, None);
        assert_eq!(info.intent, None);
        assert_eq!(info.name, None);
    }

    #[test]
    fn collect_info_inserts_same_info_under_link_scan_path_uuid() {
        let path = write_temp_state(
            "link-scan-double-insert",
            r#"{
                "sessionId":"c48dbaf9-4c87-4c76-91c0-8e20a8de849a",
                "state":"running",
                "name":"resumed agent",
                "linkScanPath":"C:\\Users\\foo\\.claude\\projects\\bar\\1ed07f96-9cff-4d93-a7ca-d5d638aad040.jsonl"
            }"#,
        );
        let mut map = HashMap::new();
        collect_info_from_job_state(&path, &mut map);

        let primary = map
            .get("c48dbaf9-4c87-4c76-91c0-8e20a8de849a")
            .expect("sessionId key present");
        assert_eq!(primary.state.as_deref(), Some("running"));
        assert_eq!(primary.name.as_deref(), Some("resumed agent"));

        let linked = map
            .get("1ed07f96-9cff-4d93-a7ca-d5d638aad040")
            .expect("linkScanPath uuid key present");
        assert_eq!(linked.state.as_deref(), Some("running"));
        assert_eq!(linked.name.as_deref(), Some("resumed agent"));
    }

    #[test]
    fn collect_info_missing_file_does_not_panic() {
        let mut map = HashMap::new();
        collect_info_from_job_state(
            &std::env::temp_dir().join("__nonexistent_state_for_test.json"),
            &mut map,
        );
        assert!(map.is_empty());
    }

    #[test]
    fn collect_info_bad_json_does_not_panic() {
        let path = write_temp_state("bad-json-info", "not json at all");
        let mut map = HashMap::new();
        collect_info_from_job_state(&path, &mut map);
        assert!(map.is_empty());
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
                bg_state: None, bg_detail: None, bg_tempo: None,
                bg_intent: None, bg_name: None,
                recent_excerpt: None,
            },
            Session {
                id: "closed-id".into(),
                jsonl_path: String::new(), cwd: None, title: None, model: None,
                message_count: 0, tokens: 0, context_tokens: 0, max_prompt_tokens: 0,
                last_activity: None, live_context_window: None, live_model_id: None,
                is_bg_agent: false, live_status: None,
                bg_state: None, bg_detail: None, bg_tempo: None,
                bg_intent: None, bg_name: None,
                recent_excerpt: None,
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
