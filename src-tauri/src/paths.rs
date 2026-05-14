use std::path::PathBuf;

pub fn home() -> Option<PathBuf> {
    dirs::home_dir()
}

#[allow(dead_code)]
pub fn claude_json_path() -> Option<PathBuf> {
    home().map(|h| h.join(".claude.json"))
}

pub fn claude_dir() -> Option<PathBuf> {
    home().map(|h| h.join(".claude"))
}

pub fn claude_daemon_roster_path() -> Option<PathBuf> {
    claude_dir().map(|c| c.join("daemon").join("roster.json"))
}

pub fn claude_projects_dir() -> Option<PathBuf> {
    claude_dir().map(|c| c.join("projects"))
}

pub fn hub_dir() -> Option<PathBuf> {
    home().map(|h| h.join(".claude-hub"))
}

pub fn hub_cache_dir() -> Option<PathBuf> {
    hub_dir().map(|h| h.join("cache"))
}

pub fn hub_ctx_cache_dir() -> Option<PathBuf> {
    hub_dir().map(|h| h.join("ctx-cache"))
}

pub fn hub_prefs_path() -> Option<PathBuf> {
    hub_dir().map(|h| h.join("prefs.json"))
}

pub fn ensure_hub_dirs() -> std::io::Result<()> {
    if let Some(c) = hub_cache_dir() {
        std::fs::create_dir_all(c)?;
    }
    if let Some(c) = hub_ctx_cache_dir() {
        std::fs::create_dir_all(c)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_descend_from_home() {
        let h = home().expect("home dir resolves");
        assert!(claude_dir().unwrap().starts_with(&h));
        assert!(claude_projects_dir().unwrap().ends_with("projects"));
        assert!(hub_cache_dir().unwrap().ends_with("cache"));
        assert!(hub_prefs_path().unwrap().ends_with("prefs.json"));
    }

    #[test]
    fn ensure_hub_dirs_is_idempotent() {
        ensure_hub_dirs().unwrap();
        ensure_hub_dirs().unwrap();
        assert!(hub_cache_dir().unwrap().exists());
    }

    #[test]
    fn daemon_roster_path_resolves() {
        let p = claude_daemon_roster_path().unwrap();
        assert!(p.ends_with("roster.json"));
        assert!(p.to_string_lossy().contains("daemon"));
    }
}
