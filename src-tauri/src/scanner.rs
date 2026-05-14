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
}
