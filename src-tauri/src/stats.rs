use crate::projects::Project;
use chrono::{Duration, Utc};
use serde::Serialize;

#[derive(Debug, Default, Serialize)]
pub struct Stats {
    pub project_count: u32,
    pub session_count: u32,
    pub tokens_7d: u64,
    pub tokens_all_time: u64,
}

pub fn aggregate(projects: &[Project]) -> Stats {
    let cutoff = Utc::now() - Duration::days(7);
    let mut session_count: u32 = 0;
    let mut tokens_7d: u64 = 0;
    let mut tokens_all_time: u64 = 0;

    for p in projects {
        if p.hidden {
            continue;
        }
        for s in p.sessions.iter().chain(p.worktrees.iter().flat_map(|w| w.sessions.iter())) {
            session_count += 1;
            tokens_all_time += s.tokens;
            if let Some(ts) = s.last_activity {
                if ts >= cutoff {
                    tokens_7d += s.tokens;
                }
            }
        }
    }

    Stats {
        project_count: projects.iter().filter(|p| !p.hidden).count() as u32,
        session_count,
        tokens_7d,
        tokens_all_time,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sessions::Session;
    use chrono::Utc;

    fn proj(hidden: bool, sessions: Vec<Session>) -> Project {
        Project {
            path: "/p".into(),
            display_name: "p".into(),
            session_count: sessions.len() as u32,
            total_tokens: sessions.iter().map(|s| s.tokens).sum(),
            last_activity: sessions.iter().filter_map(|s| s.last_activity).max(),
            sessions,
            worktrees: vec![],
            hidden,
            used_1m_recently: false,
        }
    }

    fn s(tokens: u64, days_ago: i64) -> Session {
        Session {
            id: "x".into(),
            jsonl_path: "/x".into(),
            cwd: None,
            title: None,
            model: None,
            message_count: 0,
            tokens,
            context_tokens: 0,
            max_prompt_tokens: 0,
            last_activity: Some(Utc::now() - chrono::Duration::days(days_ago)),
            live_context_window: None,
            live_model_id: None,
            is_bg_agent: false,
            live_status: None,
        }
    }

    #[test]
    fn excludes_hidden_projects() {
        let stats = aggregate(&[
            proj(true, vec![s(100, 1)]),
            proj(false, vec![s(50, 1)]),
        ]);
        assert_eq!(stats.project_count, 1);
        assert_eq!(stats.tokens_all_time, 50);
    }

    #[test]
    fn tokens_7d_window() {
        let stats = aggregate(&[proj(false, vec![s(10, 1), s(99, 30)])]);
        assert_eq!(stats.tokens_7d, 10);
        assert_eq!(stats.tokens_all_time, 109);
    }
}
