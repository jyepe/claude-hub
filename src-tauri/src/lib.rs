mod active_sessions;
mod cache;
mod claude_config;
mod killer;
mod paths;
mod prefs;
mod projects;
mod scanner;
mod sessions;
mod stats;
mod statusline_cache;
mod terminal;
mod worktree;

use projects::Project;
use stats::Stats;
use std::path::PathBuf;
use tauri::State;
use tauri::async_runtime::{Mutex, spawn_blocking};

struct AppState {
    prefs_lock: Mutex<()>,
}

fn apply_live_overlay(sessions: &mut [sessions::Session]) {
    let live = statusline_cache::read_all();
    if live.is_empty() { return; }
    for s in sessions {
        let Some(entry) = live.get(&s.id) else { continue; };
        if let Some(pct) = entry.used_percentage {
            s.live_context_window = statusline_cache::derive_window(s.context_tokens, pct);
        }
        if entry.model_id.is_some() {
            s.live_model_id = entry.model_id.clone();
        }
    }
}

#[tauri::command]
async fn list_projects() -> Vec<Project> {
    spawn_blocking(|| {
        let _ = paths::ensure_hub_dirs();
        let prefs = paths::hub_prefs_path()
            .map(|p| prefs::read(&p))
            .unwrap_or_default();
        let used_1m = claude_config::read_used_1m_projects();
        let mut sessions = scanner::scan_all();
        apply_live_overlay(&mut sessions);
        projects::group(sessions, &prefs, &used_1m)
    })
    .await
    .unwrap_or_default()
}

#[tauri::command]
async fn get_stats() -> Stats {
    spawn_blocking(|| {
        let _ = paths::ensure_hub_dirs();
        let prefs = paths::hub_prefs_path()
            .map(|p| prefs::read(&p))
            .unwrap_or_default();
        let used_1m = claude_config::read_used_1m_projects();
        let mut sessions = scanner::scan_all();
        apply_live_overlay(&mut sessions);
        let projs = projects::group(sessions, &prefs, &used_1m);
        stats::aggregate(&projs)
    })
    .await
    .unwrap_or_default()
}

#[tauri::command]
fn get_prefs() -> prefs::Prefs {
    paths::hub_prefs_path()
        .map(|p| prefs::read(&p))
        .unwrap_or_default()
}

#[tauri::command]
async fn set_prefs(new: prefs::Prefs, state: State<'_, AppState>) -> Result<(), String> {
    let _guard = state.prefs_lock.lock().await;
    let path = paths::hub_prefs_path().ok_or_else(|| "no home dir".to_string())?;
    prefs::write(&path, &new).map_err(|e| e.to_string())
}

#[tauri::command]
async fn hide_project(path: String, state: State<'_, AppState>) -> Result<(), String> {
    let _guard = state.prefs_lock.lock().await;
    let prefs_path = paths::hub_prefs_path().ok_or_else(|| "no home dir".to_string())?;
    let mut prefs = prefs::read(&prefs_path);
    prefs.hidden_projects.insert(path);
    prefs::write(&prefs_path, &prefs).map_err(|e| e.to_string())
}

#[tauri::command]
async fn unhide_project(path: String, state: State<'_, AppState>) -> Result<(), String> {
    let _guard = state.prefs_lock.lock().await;
    let prefs_path = paths::hub_prefs_path().ok_or_else(|| "no home dir".to_string())?;
    let mut prefs = prefs::read(&prefs_path);
    prefs.hidden_projects.remove(&path);
    prefs::write(&prefs_path, &prefs).map_err(|e| e.to_string())
}

#[tauri::command]
fn open_session(cwd: String, resume_id: Option<String>) -> Result<(), String> {
    terminal::open_in_terminal(&PathBuf::from(cwd), resume_id.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn attach_agent(cwd: String, session_id: String) -> Result<(), String> {
    terminal::attach_in_terminal(&PathBuf::from(cwd), &session_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn close_session(session_id: String) -> Result<(), String> {
    spawn_blocking(move || {
        let live = active_sessions::read_all();
        let proc = live
            .get(&session_id)
            .ok_or_else(|| "Session is no longer running".to_string())?;
        let claude_pid = proc.pid;
        // Best-effort: kill the hosting shell (cmd.exe/bash/…) so the terminal
        // tab closes too. On Windows `taskkill /T` cascades, so this single
        // call also kills claude. On Unix `kill -KILL` does NOT cascade, so we
        // still need the second call below to guarantee claude is gone.
        if let Some(shell_pid) = killer::find_shell_ancestor(claude_pid) {
            let _ = killer::kill_tree(shell_pid);
        }
        // Authoritative: ensure the claude process itself is dead. On Windows
        // after the cascade above this is a no-op (early-returns on dead pid).
        killer::kill_tree(claude_pid)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            prefs_lock: Mutex::new(()),
        })
        .invoke_handler(tauri::generate_handler![
            list_projects,
            get_stats,
            get_prefs,
            set_prefs,
            hide_project,
            unhide_project,
            open_session,
            attach_agent,
            close_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
