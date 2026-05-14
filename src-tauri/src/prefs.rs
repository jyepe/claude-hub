use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prefs {
    pub hidden_projects: BTreeSet<String>,
    pub noise_threshold: u32,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            hidden_projects: BTreeSet::new(),
            noise_threshold: 2,
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
}
