use crate::prefs::Prefs;
use crate::sessions::Session;
use crate::worktree;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct Project {
    pub path: String,
    pub display_name: String,
    pub session_count: u32,
    pub total_tokens: u64,
    pub last_activity: Option<DateTime<Utc>>,
    pub sessions: Vec<Session>,
    pub worktrees: Vec<Worktree>,
    pub hidden: bool,
    pub used_1m_recently: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Worktree {
    pub path: String,
    pub sessions: Vec<Session>,
}

pub fn normalize_project_path(p: &Path) -> String {
    let s: String = p.to_string_lossy().replace('\\', "/");
    if cfg!(target_os = "windows") {
        s.to_lowercase()
    } else {
        s
    }
}

pub fn group(sessions: Vec<Session>, prefs: &Prefs, used_1m_paths: &HashSet<String>) -> Vec<Project> {
    let mut by_path: BTreeMap<PathBuf, Vec<Session>> = BTreeMap::new();
    for s in sessions {
        let cwd = match s.cwd.as_deref() {
            Some(c) => PathBuf::from(c),
            None => match decode_folder(&s.jsonl_path) {
                Some(p) => p,
                None => continue,
            },
        };
        by_path.entry(cwd).or_default().push(s);
    }

    let mut wt_assignments: BTreeMap<PathBuf, Vec<(PathBuf, Vec<Session>)>> = BTreeMap::new();
    let owned: Vec<(PathBuf, Vec<Session>)> = by_path.into_iter().collect();
    let mut roots: BTreeMap<PathBuf, Vec<Session>> = BTreeMap::new();
    for (cwd, sess) in owned {
        if let Some(parent) = worktree::parent_repo(&cwd) {
            wt_assignments
                .entry(parent)
                .or_default()
                .push((cwd, sess));
        } else {
            roots.insert(cwd, sess);
        }
    }
    for parent in wt_assignments.keys() {
        roots.entry(parent.clone()).or_default();
    }

    let mut out = Vec::new();
    for (path, mut sessions) in roots {
        sessions.sort_by(|a, b| b.last_activity.cmp(&a.last_activity));

        let mut wts: Vec<Worktree> = wt_assignments
            .remove(&path)
            .unwrap_or_default()
            .into_iter()
            .map(|(p, mut s)| {
                s.sort_by(|a, b| b.last_activity.cmp(&a.last_activity));
                Worktree {
                    path: p.to_string_lossy().into_owned(),
                    sessions: s,
                }
            })
            .collect();
        wts.sort_by(|a, b| a.path.cmp(&b.path));

        let total_sessions: u32 = sessions.len() as u32
            + wts.iter().map(|w| w.sessions.len() as u32).sum::<u32>();
        let total_tokens: u64 = sessions.iter().map(|s| s.tokens).sum::<u64>()
            + wts
                .iter()
                .flat_map(|w| w.sessions.iter().map(|s| s.tokens))
                .sum::<u64>();
        let last_activity = sessions
            .iter()
            .chain(wts.iter().flat_map(|w| w.sessions.iter()))
            .filter_map(|s| s.last_activity)
            .max();

        let path_str = path.to_string_lossy().into_owned();
        let hidden = prefs.hidden_projects.contains(&path_str)
            || total_sessions < prefs.noise_threshold;
        let used_1m_recently = used_1m_paths.contains(&normalize_project_path(&path));

        out.push(Project {
            display_name: path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| path_str.clone()),
            path: path_str,
            session_count: total_sessions,
            total_tokens,
            last_activity,
            sessions,
            worktrees: wts,
            hidden,
            used_1m_recently,
        });
    }
    out.sort_by(|a, b| b.last_activity.cmp(&a.last_activity));
    out
}

fn decode_folder(jsonl_path: &str) -> Option<PathBuf> {
    let parent = Path::new(jsonl_path).parent()?;
    let folder = parent.file_name()?.to_string_lossy();
    decode_encoded_cwd(&folder)
}

#[cfg(target_os = "windows")]
fn decode_encoded_cwd(folder: &str) -> Option<PathBuf> {
    let trimmed = folder.trim_start_matches('-');
    let parts: Vec<&str> = trimmed.split('-').collect();
    if parts.len() < 2 {
        return None;
    }
    let drive = parts[0];
    let rest = parts[1..].join("\\");
    Some(PathBuf::from(format!("{}:\\{}", drive, rest)))
}

#[cfg(not(target_os = "windows"))]
fn decode_encoded_cwd(folder: &str) -> Option<PathBuf> {
    Some(PathBuf::from(folder.replace('-', "/")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn make(id: &str, cwd: &str, tokens: u64, t: i64) -> Session {
        Session {
            id: id.into(),
            jsonl_path: format!("/tmp/{}.jsonl", id),
            cwd: Some(cwd.into()),
            title: None,
            model: None,
            message_count: 1,
            tokens,
            context_tokens: 0,
            max_prompt_tokens: 0,
            last_activity: Some(Utc.timestamp_opt(t, 0).single().unwrap()),
            live_context_window: None,
            live_model_id: None,
        }
    }

    fn empty_1m() -> HashSet<String> {
        HashSet::new()
    }

    #[test]
    fn groups_sessions_by_cwd_and_aggregates() {
        let prefs = Prefs::default();
        let sessions = vec![
            make("a", "/p/one", 10, 100),
            make("b", "/p/one", 20, 200),
            make("c", "/p/two", 5, 50),
        ];
        let projects = group(sessions, &prefs, &empty_1m());
        assert_eq!(projects.len(), 2);
        let one = projects.iter().find(|p| p.path == "/p/one").unwrap();
        assert_eq!(one.session_count, 2);
        assert_eq!(one.total_tokens, 30);
    }

    #[test]
    fn hides_below_threshold() {
        let prefs = Prefs {
            noise_threshold: 2,
            ..Default::default()
        };
        let sessions = vec![make("a", "/p/lonely", 1, 1)];
        let projects = group(sessions, &prefs, &empty_1m());
        assert!(projects[0].hidden);
    }

    #[test]
    fn hides_when_user_listed() {
        let mut prefs = Prefs::default();
        prefs.hidden_projects.insert("/p/x".into());
        prefs.noise_threshold = 0;
        let sessions = vec![make("a", "/p/x", 1, 1), make("b", "/p/x", 2, 2)];
        let projects = group(sessions, &prefs, &empty_1m());
        assert!(projects[0].hidden);
    }

    #[test]
    fn sessions_within_project_sorted_by_last_activity_desc() {
        let prefs = Prefs { noise_threshold: 0, ..Default::default() };
        let sessions = vec![
            make("old", "/p/proj", 1, 100),
            make("mid", "/p/proj", 1, 500),
            make("new", "/p/proj", 1, 999),
        ];
        let projects = group(sessions, &prefs, &empty_1m());
        let ids: Vec<&str> = projects[0].sessions.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(ids, vec!["new", "mid", "old"]);
    }

    #[test]
    fn sorts_by_last_activity_desc() {
        let prefs = Prefs {
            noise_threshold: 0,
            ..Default::default()
        };
        let sessions = vec![
            make("a", "/p/old", 1, 100),
            make("b", "/p/new", 1, 9999),
        ];
        let projects = group(sessions, &prefs, &empty_1m());
        assert_eq!(projects[0].path, "/p/new");
    }

    #[test]
    fn marks_project_when_path_in_1m_set() {
        let prefs = Prefs { noise_threshold: 0, ..Default::default() };
        let sessions = vec![make("a", "/p/one", 1, 1)];
        let mut hint = HashSet::new();
        hint.insert("/p/one".to_string());
        let projects = group(sessions, &prefs, &hint);
        assert!(projects[0].used_1m_recently);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn normalizes_path_for_1m_match_on_windows() {
        let prefs = Prefs { noise_threshold: 0, ..Default::default() };
        let sessions = vec![make("a", "C:\\Users\\Foo\\Proj", 1, 1)];
        let mut hint = HashSet::new();
        hint.insert("c:/users/foo/proj".to_string());
        let projects = group(sessions, &prefs, &hint);
        assert!(projects[0].used_1m_recently);
    }
}
