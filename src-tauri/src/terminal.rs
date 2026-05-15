use std::path::Path;
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SpawnError {
    #[error("no terminal emulator found")]
    #[allow(dead_code)]
    NoTerminal,
    #[error("invalid session id")]
    InvalidSessionId,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn open_in_terminal(cwd: &Path, resume_id: Option<&str>) -> Result<(), SpawnError> {
    let claude_invocation = match resume_id {
        Some(id) => format!("claude --resume {}", shell_escape(id)),
        None => "claude".to_string(),
    };
    spawn_platform(cwd, &claude_invocation)
}

pub fn attach_in_terminal(cwd: &Path, session_id: &str) -> Result<(), SpawnError> {
    if !session_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Err(SpawnError::InvalidSessionId);
    }
    let cmd = format!("claude agents attach {}", session_id);
    spawn_platform(cwd, &cmd)
}

fn shell_escape(s: &str) -> String {
    let safe = s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if safe {
        s.to_string()
    } else {
        format!("\"{}\"", s.replace('"', "\\\""))
    }
}

#[cfg(target_os = "windows")]
fn spawn_platform(cwd: &Path, cmdline: &str) -> Result<(), SpawnError> {
    let cwd_str = cwd.to_string_lossy().into_owned();
    if Command::new("wt.exe")
        .args(["-d", &cwd_str, "cmd", "/k", cmdline])
        .spawn()
        .is_ok()
    {
        return Ok(());
    }
    Command::new("cmd")
        .args([
            "/c",
            "start",
            "cmd",
            "/k",
            &format!("cd /d \"{}\" && {}", cwd_str, cmdline),
        ])
        .spawn()?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn spawn_platform(cwd: &Path, cmdline: &str) -> Result<(), SpawnError> {
    let script = format!(
        "tell application \"Terminal\" to do script \"cd {} && {}\"",
        shell_quote_unix(&cwd.to_string_lossy()),
        cmdline
    );
    Command::new("osascript").args(["-e", &script]).spawn()?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn spawn_platform(cwd: &Path, cmdline: &str) -> Result<(), SpawnError> {
    let cwd_str = cwd.to_string_lossy().into_owned();
    let bash = format!("cd {} && {}; exec bash", shell_quote_unix(&cwd_str), cmdline);
    for term in [
        "gnome-terminal",
        "konsole",
        "xfce4-terminal",
        "alacritty",
        "kitty",
        "wezterm",
    ] {
        let result = match term {
            "gnome-terminal" => Command::new(term).args(["--", "bash", "-c", &bash]).spawn(),
            "konsole" => Command::new(term).args(["-e", "bash", "-c", &bash]).spawn(),
            "xfce4-terminal" => Command::new(term)
                .args(["--command", &format!("bash -c {}", shell_quote_unix(&bash))])
                .spawn(),
            _ => Command::new(term).args(["-e", "bash", "-c", &bash]).spawn(),
        };
        if result.is_ok() {
            return Ok(());
        }
    }
    Err(SpawnError::NoTerminal)
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn shell_quote_unix(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_escape_allows_safe_ids() {
        assert_eq!(shell_escape("0b36e159-8022"), "0b36e159-8022");
    }

    #[test]
    fn shell_escape_quotes_unsafe_input() {
        assert_eq!(shell_escape("a;b"), "\"a;b\"");
    }

    #[test]
    fn attach_command_format() {
        let id = "0b36e159-8022-444a-a9f7-164faaa78e49";
        let cmd = format!("claude agents attach {}", shell_escape(id));
        assert_eq!(cmd, "claude agents attach 0b36e159-8022-444a-a9f7-164faaa78e49");
    }

    #[test]
    fn attach_rejects_unsafe_id() {
        let result = attach_in_terminal(std::path::Path::new("."), "id with spaces");
        assert!(matches!(result, Err(SpawnError::InvalidSessionId)));
    }
}
