use crate::paths;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub fn claude_json_path() -> Option<std::path::PathBuf> {
    paths::home().map(|h| h.join(".claude.json"))
}

pub fn read_used_1m_projects() -> HashSet<String> {
    let Some(path) = claude_json_path() else { return HashSet::new(); };
    if !path.exists() { return HashSet::new(); }
    let Ok(text) = fs::read_to_string(&path) else { return HashSet::new(); };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else { return HashSet::new(); };
    parse_used_1m_projects(&v)
}

fn parse_used_1m_projects(v: &serde_json::Value) -> HashSet<String> {
    let mut out = HashSet::new();
    let Some(projects) = v.get("projects").and_then(|p| p.as_object()) else { return out; };
    for (raw_path, meta) in projects {
        let Some(usage) = meta.get("lastModelUsage").and_then(|u| u.as_object()) else { continue; };
        let has_1m = usage.keys().any(|k| k.contains("[1m]"));
        if has_1m {
            out.insert(normalize(raw_path));
        }
    }
    out
}

fn normalize(p: &str) -> String {
    let s = p.replace('\\', "/");
    if cfg!(target_os = "windows") {
        s.to_lowercase()
    } else {
        s
    }
}

#[allow(dead_code)] // exposed for use during scanner.rs integration
pub fn normalize_path(p: &Path) -> String {
    let s: String = p.to_string_lossy().replace('\\', "/");
    if cfg!(target_os = "windows") {
        s.to_lowercase()
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extracts_projects_with_1m_model_usage() {
        let v = json!({
            "projects": {
                "C:/Users/a/proj-one": {
                    "lastModelUsage": {
                        "claude-opus-4-7[1m]": { "inputTokens": 1 }
                    }
                },
                "/home/u/proj-two": {
                    "lastModelUsage": {
                        "claude-opus-4-7": { "inputTokens": 1 }
                    }
                },
                "/home/u/proj-three": {
                    "lastModelUsage": {
                        "claude-haiku-4-5-20251001": { "inputTokens": 1 },
                        "claude-sonnet-4-6[1m]": { "inputTokens": 1 }
                    }
                }
            }
        });
        let set = parse_used_1m_projects(&v);
        // Normalized: forward-slashes, lowercased on Windows.
        let want_one = if cfg!(target_os = "windows") { "c:/users/a/proj-one" } else { "C:/Users/a/proj-one" };
        let want_three = "/home/u/proj-three";
        assert!(set.contains(want_one), "expected {want_one} in {set:?}");
        assert!(set.contains(want_three));
        assert!(!set.iter().any(|p| p.contains("proj-two")));
    }

    #[test]
    fn empty_when_no_projects_key() {
        let set = parse_used_1m_projects(&json!({}));
        assert!(set.is_empty());
    }

    #[test]
    fn empty_when_project_has_no_lastModelUsage() {
        let v = json!({ "projects": { "/x": { "other": 1 } } });
        assert!(parse_used_1m_projects(&v).is_empty());
    }
}
