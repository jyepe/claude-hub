use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prefs {
    pub hidden_projects: BTreeSet<String>,
    pub noise_threshold: u32,
    #[serde(default)]
    pub pinned_session_ids: Vec<String>,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            hidden_projects: BTreeSet::new(),
            noise_threshold: 2,
            pinned_session_ids: Vec::new(),
        }
    }
}

pub fn read(path: &Path) -> Prefs {
    std::fs::read(path)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

pub fn write(path: &Path, prefs: &Prefs) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_vec_pretty(prefs)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write(path, json)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_path() -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "claude-hub-prefs-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn read_returns_default_when_missing() {
        let p = read(&fresh_path());
        assert_eq!(p.noise_threshold, 2);
        assert!(p.hidden_projects.is_empty());
    }

    #[test]
    fn write_then_read_roundtrips() {
        let path = fresh_path();
        let mut p = Prefs::default();
        p.hidden_projects.insert("/x/y".to_string());
        p.noise_threshold = 5;
        write(&path, &p).unwrap();
        let r = read(&path);
        assert_eq!(r.noise_threshold, 5);
        assert!(r.hidden_projects.contains("/x/y"));
    }

    #[test]
    fn pinned_session_ids_default_empty_and_roundtrip() {
        let path = fresh_path();
        let p_default = read(&path);
        assert!(p_default.pinned_session_ids.is_empty());

        let mut p = Prefs::default();
        p.pinned_session_ids.push("sess-aaa".to_string());
        p.pinned_session_ids.push("sess-bbb".to_string());
        write(&path, &p).unwrap();
        let r = read(&path);
        assert_eq!(r.pinned_session_ids, vec!["sess-aaa", "sess-bbb"]);
    }

    #[test]
    fn legacy_prefs_file_without_pinned_field_deserializes() {
        let path = fresh_path();
        let legacy_json = br#"{"hidden_projects":["/x/y"],"noise_threshold":3}"#;
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, legacy_json).unwrap();
        let p = read(&path);
        assert_eq!(p.noise_threshold, 3);
        assert!(p.hidden_projects.contains("/x/y"));
        assert!(p.pinned_session_ids.is_empty());
    }
}
