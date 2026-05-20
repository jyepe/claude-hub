use crate::killer;
use crate::paths;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LiveStatus {
    Idle,
    Busy,
}

#[derive(Debug, Clone)]
pub struct LiveProcess {
    pub pid: u32,
    pub status: LiveStatus,
    #[allow(dead_code)]
    pub session_id: String,
}

/// Load every `~/.claude/sessions/{pid}.json`, drop entries whose pid is no
/// longer running on the OS, and return a map keyed by `sessionId`.
///
/// Aliveness is checked against a single snapshot of all live PIDs taken
/// once per call. This keeps the cost at O(1) subprocess spawns regardless
/// of how many stale session files are sitting in the directory.
pub fn read_all() -> HashMap<String, LiveProcess> {
    let live = killer::live_pids_snapshot();
    read_all_from(paths::claude_sessions_dir().as_deref(), |pid| {
        live.contains(&pid)
    })
}

/// Inner form factored out for tests: directory and aliveness check are
/// injected so we don't need real processes.
fn read_all_from(
    dir: Option<&std::path::Path>,
    is_alive: impl Fn(u32) -> bool,
) -> HashMap<String, LiveProcess> {

    let mut out = HashMap::new();
    let Some(dir) = dir else { return out; };
    let Ok(entries) = fs::read_dir(dir) else { return out; };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else { continue; };
        if stem.starts_with(".claude-hub-") { continue; }
        let Ok(text) = fs::read_to_string(&path) else { continue; };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else { continue; };
        let Some(proc) = parse_entry(&v) else { continue; };
        if !is_alive(proc.pid) { continue; }
        out.insert(proc.session_id.clone(), proc);
    }
    out
}

fn parse_entry(v: &serde_json::Value) -> Option<LiveProcess> {
    let pid = v.get("pid").and_then(|n| n.as_u64())? as u32;
    let session_id = v.get("sessionId").and_then(|s| s.as_str())?.to_string();
    let status = match v.get("status").and_then(|s| s.as_str()) {
        Some("busy") => LiveStatus::Busy,
        Some("idle") => LiveStatus::Idle,
        // Unknown / missing status — assume idle so the dot still renders for
        // live processes. Better to show "live" than to hide a real session.
        _ => LiveStatus::Idle,
    };
    Some(LiveProcess { pid, status, session_id })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn make_fixture_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "claude-hub-active-sessions-{}-{}",
            name,
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_file(dir: &std::path::Path, name: &str, contents: &str) {
        let mut f = std::fs::File::create(dir.join(name)).unwrap();
        f.write_all(contents.as_bytes()).unwrap();
    }

    #[test]
    fn parses_valid_busy_entry() {
        let v: serde_json::Value = serde_json::from_str(
            r#"{"pid":1234,"sessionId":"abc","status":"busy"}"#,
        )
        .unwrap();
        let p = parse_entry(&v).unwrap();
        assert_eq!(p.pid, 1234);
        assert_eq!(p.session_id, "abc");
        assert_eq!(p.status, LiveStatus::Busy);
    }

    #[test]
    fn parses_valid_idle_entry() {
        let v: serde_json::Value = serde_json::from_str(
            r#"{"pid":1,"sessionId":"x","status":"idle"}"#,
        )
        .unwrap();
        assert_eq!(parse_entry(&v).unwrap().status, LiveStatus::Idle);
    }

    #[test]
    fn unknown_status_defaults_to_idle() {
        let v: serde_json::Value = serde_json::from_str(
            r#"{"pid":1,"sessionId":"x","status":"weird"}"#,
        )
        .unwrap();
        assert_eq!(parse_entry(&v).unwrap().status, LiveStatus::Idle);
    }

    #[test]
    fn missing_pid_is_rejected() {
        let v: serde_json::Value = serde_json::from_str(r#"{"sessionId":"x"}"#).unwrap();
        assert!(parse_entry(&v).is_none());
    }

    #[test]
    fn missing_session_id_is_rejected() {
        let v: serde_json::Value = serde_json::from_str(r#"{"pid":1}"#).unwrap();
        assert!(parse_entry(&v).is_none());
    }

    #[test]
    fn read_all_includes_alive_skips_dead_and_malformed() {
        let dir = make_fixture_dir("filtering");
        write_file(&dir, "100.json", r#"{"pid":100,"sessionId":"alive","status":"busy"}"#);
        write_file(&dir, "200.json", r#"{"pid":200,"sessionId":"dead","status":"idle"}"#);
        write_file(&dir, "300.json", r#"{"not":"json"#); // malformed
        write_file(&dir, "readme.txt", r#"not json at all"#); // wrong extension
        write_file(&dir, ".claude-hub-temp.json", r#"{"pid":400,"sessionId":"tmp","status":"idle"}"#);

        let alive = |pid: u32| pid == 100;
        let map = read_all_from(Some(&dir), alive);
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("alive"));
        assert_eq!(map["alive"].status, LiveStatus::Busy);
    }

    #[test]
    fn read_all_returns_empty_when_dir_missing() {
        let bogus = std::env::temp_dir().join("__claude_hub_definitely_not_a_dir__");
        let _ = std::fs::remove_dir_all(&bogus);
        let map = read_all_from(Some(&bogus), |_| true);
        assert!(map.is_empty());
    }

    #[test]
    fn read_all_returns_empty_when_dir_none() {
        let map = read_all_from(None, |_| true);
        assert!(map.is_empty());
    }
}
