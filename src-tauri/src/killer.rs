use std::process::Command;

/// Returns true if a process with `pid` is currently running on this OS.
/// Uses `tasklist` on Windows and `kill -0` on Unix to avoid taking on
/// extra crate dependencies.
pub fn pid_alive(pid: u32) -> bool {
    pid_alive_impl(pid)
}

#[cfg(target_os = "windows")]
fn pid_alive_impl(pid: u32) -> bool {
    let output = Command::new("tasklist")
        .args(["/FI", &format!("PID eq {}", pid), "/NH", "/FO", "CSV"])
        .output();
    let Ok(output) = output else { return false; };
    if !output.status.success() { return false; }
    // tasklist prints "INFO: No tasks ..." to stdout when no match; matches
    // produce a CSV row starting with the image name in quotes.
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.lines().any(|l| l.contains(&format!(",\"{}\",", pid)))
}

#[cfg(not(target_os = "windows"))]
fn pid_alive_impl(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_process_pid_is_alive() {
        let me = std::process::id();
        assert!(pid_alive(me), "pid_alive should be true for our own pid");
    }

    #[test]
    fn nonexistent_pid_is_not_alive() {
        // u32::MAX - 1 is unrealistic and will not be assigned by either kernel.
        assert!(!pid_alive(u32::MAX - 1));
    }
}
