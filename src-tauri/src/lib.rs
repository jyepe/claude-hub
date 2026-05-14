mod cache;
mod paths;
mod prefs;
mod projects;
mod scanner;
mod sessions;
mod stats;
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

#[tauri::command]
async fn list_projects() -> Vec<Project> {
    spawn_blocking(|| {
        let _ = paths::ensure_hub_dirs();
        let prefs = paths::hub_prefs_path()
            .map(|p| prefs::read(&p))
            .unwrap_or_default();
        projects::group(scanner::scan_all(), &prefs)
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
        let projs = projects::group(scanner::scan_all(), &prefs);
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
