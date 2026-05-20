use crate::sessions::Session;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheEntry {
    pub jsonl_path: String,
    pub source_mtime_unix: u64,
    pub session: Session,
}

fn key_for(jsonl_path: &Path) -> String {
    let mut h = Sha256::new();
    h.update(jsonl_path.to_string_lossy().as_bytes());
    format!("{:x}", h.finalize())
}

fn cache_file(cache_dir: &Path, jsonl_path: &Path) -> PathBuf {
    cache_dir.join(format!("{}.json", key_for(jsonl_path)))
}

fn mtime_unix(path: &Path) -> std::io::Result<u64> {
    let meta = std::fs::metadata(path)?;
    let mtime = meta.modified()?;
    Ok(mtime
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0))
}

pub fn read(cache_dir: &Path, jsonl_path: &Path) -> Option<Session> {
    let mtime = mtime_unix(jsonl_path).ok()?;
    let entry_path = cache_file(cache_dir, jsonl_path);
    let bytes = std::fs::read(&entry_path).ok()?;
    let entry: CacheEntry = serde_json::from_slice(&bytes).ok()?;
    if entry.source_mtime_unix == mtime {
        Some(entry.session)
    } else {
        None
    }
}

pub fn write(cache_dir: &Path, jsonl_path: &Path, session: &Session) -> std::io::Result<()> {
    std::fs::create_dir_all(cache_dir)?;
    let mtime = mtime_unix(jsonl_path)?;
    let entry = CacheEntry {
        jsonl_path: jsonl_path.to_string_lossy().into_owned(),
        source_mtime_unix: mtime,
        session: session.clone(),
    };
    let json = serde_json::to_vec(&entry)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write(cache_file(cache_dir, jsonl_path), json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn make_jsonl(dir: &Path, name: &str, body: &str) -> PathBuf {
        let p = dir.join(name);
        let mut f = std::fs::File::create(&p).unwrap();
        f.write_all(body.as_bytes()).unwrap();
        p
    }

    fn fresh_temp() -> PathBuf {
        let base = std::env::temp_dir().join(format!(
            "claude-hub-cache-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&base).unwrap();
        base
    }

    #[test]
    fn write_then_read_returns_session() {
        let tmp = fresh_temp();
        let cache = tmp.join("cache");
        let jsonl = make_jsonl(&tmp, "a.jsonl", "{}");
        let s = Session {
            id: "x".into(),
            jsonl_path: jsonl.to_string_lossy().into(),
            cwd: None,
            title: None,
            model: None,
            message_count: 0,
            tokens: 42,
            context_tokens: 0,
            max_prompt_tokens: 0,
            last_activity: None,
            live_context_window: None,
            live_model_id: None,
            is_bg_agent: false,
            live_status: None,
        };
        write(&cache, &jsonl, &s).unwrap();
        let got = read(&cache, &jsonl).unwrap();
        assert_eq!(got.tokens, 42);
    }

    #[test]
    fn read_returns_none_when_mtime_changes() {
        let tmp = fresh_temp();
        let cache = tmp.join("cache");
        let jsonl = make_jsonl(&tmp, "b.jsonl", "{}");
        let s = Session {
            id: "y".into(),
            jsonl_path: jsonl.to_string_lossy().into(),
            cwd: None,
            title: None,
            model: None,
            message_count: 0,
            tokens: 1,
            context_tokens: 0,
            max_prompt_tokens: 0,
            last_activity: None,
            live_context_window: None,
            live_model_id: None,
            is_bg_agent: false,
            live_status: None,
        };
        write(&cache, &jsonl, &s).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(1100));
        let mut f = std::fs::OpenOptions::new()
            .append(true)
            .open(&jsonl)
            .unwrap();
        f.write_all(b"\nmore\n").unwrap();
        drop(f);

        assert!(read(&cache, &jsonl).is_none());
    }

    #[test]
    fn read_returns_none_when_no_cache() {
        let tmp = fresh_temp();
        let cache = tmp.join("cache");
        let jsonl = make_jsonl(&tmp, "c.jsonl", "{}");
        assert!(read(&cache, &jsonl).is_none());
    }
}
