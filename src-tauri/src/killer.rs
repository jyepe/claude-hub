use std::collections::HashSet;
use std::process::Command;
use std::thread::sleep;
use std::time::{Duration, Instant};

/// Returns true if a process with `pid` is currently running on this OS.
/// Uses `tasklist` on Windows and `kill -0` on Unix to avoid taking on
/// extra crate dependencies.
pub fn pid_alive(pid: u32) -> bool {
    pid_alive_impl(pid)
}

/// Snapshot every live PID in a single OS call. Use this when checking many
/// PIDs at once — spawning `tasklist`/`kill -0` per pid is O(N) subprocesses
/// per scan, which is the bottleneck for `active_sessions::read_all` when
/// the sessions directory has many stale files. Returns an empty set if the
/// OS call fails; callers should treat that as "no live sessions detected"
/// rather than erroring out.
pub fn live_pids_snapshot() -> HashSet<u32> {
    live_pids_snapshot_impl().unwrap_or_default()
}

/// Terminate the process tree rooted at `pid`. Tries a graceful signal first;
/// if the process is still alive after the grace window, escalates to a
/// forceful kill. Returns Ok(()) if the process is gone by the time we
/// return, regardless of whether the graceful or forceful path got us there.
pub fn kill_tree(pid: u32) -> Result<(), String> {
    if !pid_alive(pid) {
        return Ok(()); // already gone — treat as success
    }
    // Ignore graceful kill errors — console-only processes (e.g. ping on
    // Windows) may reject WM_CLOSE. If the process is still alive after the
    // grace window, the forceful path takes over.
    let _ = graceful_kill(pid);
    if wait_until_dead(pid, Duration::from_millis(2000)) {
        return Ok(());
    }
    // Same rationale as graceful_kill: the process may have exited between
    // the aliveness check and this call, which would make taskkill/kill
    // return non-zero. The wait_until_dead check below is the real verdict.
    let _ = forceful_kill(pid);
    if wait_until_dead(pid, Duration::from_millis(1000)) {
        return Ok(());
    }
    Err(format!("pid {pid} still alive after TERM and KILL"))
}

fn wait_until_dead(pid: u32, max: Duration) -> bool {
    let start = Instant::now();
    while start.elapsed() < max {
        if !pid_alive(pid) { return true; }
        sleep(Duration::from_millis(100));
    }
    !pid_alive(pid)
}

#[cfg(target_os = "windows")]
fn graceful_kill(pid: u32) -> std::io::Result<()> {
    // /T kills the entire tree. Without /F, taskkill asks the process to
    // close gracefully via WM_CLOSE.
    let status = Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/T"])
        .status()?;
    if !status.success() {
        return Err(std::io::Error::other(format!(
            "taskkill exited with status {status}"
        )));
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn forceful_kill(pid: u32) -> std::io::Result<()> {
    let status = Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/T", "/F"])
        .status()?;
    if !status.success() {
        return Err(std::io::Error::other(format!(
            "taskkill /F exited with status {status}"
        )));
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn graceful_kill(pid: u32) -> std::io::Result<()> {
    let status = Command::new("kill")
        .args(["-TERM", &pid.to_string()])
        .status()?;
    if !status.success() {
        return Err(std::io::Error::other(format!(
            "kill -TERM exited with status {status}"
        )));
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn forceful_kill(pid: u32) -> std::io::Result<()> {
    let status = Command::new("kill")
        .args(["-KILL", &pid.to_string()])
        .status()?;
    if !status.success() {
        return Err(std::io::Error::other(format!(
            "kill -KILL exited with status {status}"
        )));
    }
    Ok(())
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

#[cfg(target_os = "windows")]
fn live_pids_snapshot_impl() -> Option<HashSet<u32>> {
    // `tasklist /NH /FO CSV` prints rows like:
    //   "image.exe","1234","Console","1","12,345 K"
    // We only need the PID (second column).
    let output = Command::new("tasklist")
        .args(["/NH", "/FO", "CSV"])
        .output()
        .ok()?;
    if !output.status.success() { return None; }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Some(parse_tasklist_csv(&stdout))
}

#[cfg(target_os = "windows")]
fn parse_tasklist_csv(stdout: &str) -> HashSet<u32> {
    stdout
        .lines()
        .filter_map(|line| {
            // Split on `","` then strip the leading `"` from the first field.
            let mut fields = line.split("\",\"");
            let _image = fields.next()?;
            let pid_field = fields.next()?.trim_matches('"');
            pid_field.parse::<u32>().ok()
        })
        .collect()
}

#[cfg(not(target_os = "windows"))]
fn live_pids_snapshot_impl() -> Option<HashSet<u32>> {
    // `ps -A -o pid=` prints one PID per line (header suppressed by `=`).
    let output = Command::new("ps").args(["-A", "-o", "pid="]).output().ok()?;
    if !output.status.success() { return None; }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Some(
        stdout
            .lines()
            .filter_map(|l| l.trim().parse::<u32>().ok())
            .collect(),
    )
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

    #[test]
    fn live_pids_snapshot_includes_current_process() {
        let me = std::process::id();
        let snap = live_pids_snapshot();
        assert!(!snap.is_empty(), "snapshot should not be empty");
        assert!(snap.contains(&me), "snapshot should contain our own pid");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn parse_tasklist_csv_extracts_pids() {
        let sample = "\"svchost.exe\",\"1234\",\"Services\",\"0\",\"5,000 K\"\n\
                      \"chrome.exe\",\"5678\",\"Console\",\"1\",\"100,000 K\"\n";
        let set = parse_tasklist_csv(sample);
        assert!(set.contains(&1234));
        assert!(set.contains(&5678));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn kill_tree_terminates_a_spawned_sleep() {
        // Spawn a long-sleeping child appropriate for the platform.
        #[cfg(target_os = "windows")]
        let mut child = Command::new("ping")
            .args(["-n", "31", "127.0.0.1"])
            .spawn()
            .expect("spawn ping");
        #[cfg(not(target_os = "windows"))]
        let mut child = Command::new("sleep")
            .arg("30")
            .spawn()
            .expect("spawn sleep");

        let pid = child.id();
        assert!(pid_alive(pid), "child should be alive immediately after spawn");

        kill_tree(pid).expect("kill_tree should succeed");

        // Reap zombie on unix; on windows the handle gets released by wait().
        let _ = child.wait();
        assert!(!pid_alive(pid), "child should be dead after kill_tree returns");
    }
}
