use std::collections::{HashMap, HashSet};
use std::process::Command;
use std::thread::sleep;
use std::time::{Duration, Instant};

/// Process-table entry: parent pid + executable name (basename only).
#[derive(Debug, Clone)]
pub struct ProcInfo {
    pub ppid: u32,
    pub name: String,
}

/// Walk up the ancestor chain from `pid` and return the first ancestor whose
/// executable name is a known interactive shell (`cmd.exe`, `bash`, …).
///
/// Returns `None` if no shell ancestor is found within a small depth bound,
/// or if the OS process table couldn't be read. `pid` itself is *not*
/// considered — we always start from its parent — so this won't match the
/// claude process even if it were itself a shell.
///
/// Used by `close_session` to kill the shell hosting a Claude session so the
/// terminal tab/window closes along with the session.
pub fn find_shell_ancestor(pid: u32) -> Option<u32> {
    let table = process_table();
    find_shell_ancestor_in(pid, &table)
}

fn find_shell_ancestor_in(pid: u32, table: &HashMap<u32, ProcInfo>) -> Option<u32> {
    const MAX_DEPTH: usize = 8;
    let mut current = table.get(&pid)?.ppid;
    for _ in 0..MAX_DEPTH {
        if current == 0 { return None; }
        let info = table.get(&current)?;
        if is_shell(&info.name) { return Some(current); }
        if info.ppid == current { return None; } // self-parent guard
        current = info.ppid;
    }
    None
}

fn is_shell(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    let stem = lower.strip_suffix(".exe").unwrap_or(&lower);
    matches!(
        stem,
        "cmd" | "powershell" | "pwsh"
            | "bash" | "zsh" | "sh" | "fish" | "dash" | "tcsh" | "ksh"
    )
}

fn process_table() -> HashMap<u32, ProcInfo> {
    process_table_impl().unwrap_or_default()
}

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

#[cfg(target_os = "windows")]
fn process_table_impl() -> Option<HashMap<u32, ProcInfo>> {
    // Powered by Get-CimInstance instead of the deprecated wmic. Slower cold-
    // start than tasklist (~500ms), but we only run this on user click.
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Get-CimInstance Win32_Process | Select-Object ProcessId,ParentProcessId,Name | ConvertTo-Csv -NoTypeInformation",
        ])
        .output()
        .ok()?;
    if !output.status.success() { return None; }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Some(parse_cim_csv(&stdout))
}

#[cfg(target_os = "windows")]
fn parse_cim_csv(stdout: &str) -> HashMap<u32, ProcInfo> {
    let mut out = HashMap::new();
    let mut lines = stdout.lines();
    let _header = lines.next();
    for line in lines {
        // Each row: "1234","5678","cmd.exe"
        let trimmed = line.trim();
        if trimmed.len() < 2 { continue; }
        let body = &trimmed[1..trimmed.len() - 1]; // strip outer quotes
        let parts: Vec<&str> = body.split("\",\"").collect();
        if parts.len() < 3 { continue; }
        let Ok(pid) = parts[0].parse::<u32>() else { continue; };
        let Ok(ppid) = parts[1].parse::<u32>() else { continue; };
        let name = parts[2].to_string();
        out.insert(pid, ProcInfo { ppid, name });
    }
    out
}

#[cfg(target_os = "linux")]
fn process_table_impl() -> Option<HashMap<u32, ProcInfo>> {
    let mut out = HashMap::new();
    let entries = std::fs::read_dir("/proc").ok()?;
    for entry in entries.flatten() {
        let fname = entry.file_name();
        let Some(name_str) = fname.to_str() else { continue; };
        let Ok(pid) = name_str.parse::<u32>() else { continue; };
        let stat = match std::fs::read_to_string(entry.path().join("stat")) {
            Ok(s) => s,
            Err(_) => continue,
        };
        // /proc/<pid>/stat: "PID (comm) STATE PPID ..."
        // comm can contain spaces/parens, so locate the last ')' first.
        let Some(close) = stat.rfind(')') else { continue; };
        let Some(open) = stat[..close].rfind('(') else { continue; };
        let proc_name = stat[open + 1..close].to_string();
        let rest = stat[close + 1..].trim();
        let mut fields = rest.split_ascii_whitespace();
        let _state = fields.next();
        let Some(ppid_str) = fields.next() else { continue; };
        let Ok(ppid) = ppid_str.parse::<u32>() else { continue; };
        out.insert(pid, ProcInfo { ppid, name: proc_name });
    }
    Some(out)
}

#[cfg(target_os = "macos")]
fn process_table_impl() -> Option<HashMap<u32, ProcInfo>> {
    let output = Command::new("ps")
        .args(["-A", "-o", "pid=,ppid=,comm="])
        .output()
        .ok()?;
    if !output.status.success() { return None; }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Some(parse_ps_full(&stdout))
}

#[cfg(target_os = "macos")]
fn parse_ps_full(stdout: &str) -> HashMap<u32, ProcInfo> {
    let mut out = HashMap::new();
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        let mut iter = line.split_ascii_whitespace();
        let Some(pid_str) = iter.next() else { continue; };
        let Some(ppid_str) = iter.next() else { continue; };
        let Ok(pid) = pid_str.parse::<u32>() else { continue; };
        let Ok(ppid) = ppid_str.parse::<u32>() else { continue; };
        let rest: String = iter.collect::<Vec<_>>().join(" ");
        let basename = std::path::Path::new(&rest)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&rest)
            .to_string();
        out.insert(pid, ProcInfo { ppid, name: basename });
    }
    out
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

    fn make_table(rows: &[(u32, u32, &str)]) -> HashMap<u32, ProcInfo> {
        rows.iter()
            .map(|(pid, ppid, name)| (*pid, ProcInfo { ppid: *ppid, name: name.to_string() }))
            .collect()
    }

    #[test]
    fn is_shell_recognizes_common_shells() {
        assert!(is_shell("cmd.exe"));
        assert!(is_shell("CMD.EXE"));
        assert!(is_shell("powershell.exe"));
        assert!(is_shell("pwsh.exe"));
        assert!(is_shell("bash"));
        assert!(is_shell("zsh"));
        assert!(!is_shell("node.exe"));
        assert!(!is_shell("claude"));
        assert!(!is_shell("WindowsTerminal.exe"));
    }

    #[test]
    fn find_shell_ancestor_walks_up_to_cmd() {
        // node(100) -> claude.cmd(200) -> cmd.exe(300) -> WindowsTerminal(400)
        let table = make_table(&[
            (100, 200, "node.exe"),
            (200, 300, "claude.cmd"),
            (300, 400, "cmd.exe"),
            (400, 1, "WindowsTerminal.exe"),
        ]);
        assert_eq!(find_shell_ancestor_in(100, &table), Some(300));
    }

    #[test]
    fn find_shell_ancestor_returns_none_when_no_shell() {
        let table = make_table(&[
            (100, 200, "node.exe"),
            (200, 1, "WindowsTerminal.exe"),
        ]);
        assert_eq!(find_shell_ancestor_in(100, &table), None);
    }

    #[test]
    fn find_shell_ancestor_skips_pid_itself() {
        // If the pid itself is a shell, we still walk past it — we want the
        // *hosting* shell, not the session process.
        let table = make_table(&[
            (100, 200, "bash"),
            (200, 1, "bash"),
        ]);
        assert_eq!(find_shell_ancestor_in(100, &table), Some(200));
    }

    #[test]
    fn find_shell_ancestor_handles_self_parent_cycle() {
        // Init-like process where ppid == pid would otherwise loop forever.
        let table = make_table(&[
            (100, 200, "node.exe"),
            (200, 200, "init"),
        ]);
        assert_eq!(find_shell_ancestor_in(100, &table), None);
    }

    #[test]
    fn find_shell_ancestor_respects_depth_limit() {
        // 10 non-shell ancestors then a shell — MAX_DEPTH=8 should stop short.
        let mut rows: Vec<(u32, u32, &str)> = (1..=10)
            .map(|i| (i, i + 1, "node.exe"))
            .collect();
        rows.push((11, 1, "bash"));
        let table = make_table(&rows);
        assert_eq!(find_shell_ancestor_in(1, &table), None);
    }

    #[test]
    fn find_shell_ancestor_missing_pid_returns_none() {
        let table = make_table(&[(100, 200, "node.exe")]);
        assert_eq!(find_shell_ancestor_in(999, &table), None);
    }

    #[test]
    fn process_table_includes_current_process() {
        let me = std::process::id();
        let table = process_table();
        assert!(!table.is_empty(), "process table should not be empty");
        let info = table.get(&me).expect("our pid should be in the table");
        assert!(info.ppid > 0, "we should have a parent process");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn parse_cim_csv_extracts_pid_ppid_name() {
        let sample = "\"ProcessId\",\"ParentProcessId\",\"Name\"\n\
                      \"1234\",\"5678\",\"cmd.exe\"\n\
                      \"9000\",\"1\",\"WindowsTerminal.exe\"\n";
        let table = parse_cim_csv(sample);
        assert_eq!(table.len(), 2);
        assert_eq!(table[&1234].ppid, 5678);
        assert_eq!(table[&1234].name, "cmd.exe");
        assert_eq!(table[&9000].name, "WindowsTerminal.exe");
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
