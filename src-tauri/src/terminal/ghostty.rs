use log::{info, error};
use std::process::Command;

use super::applescript::execute_applescript;

/// Try focusing Ghostty terminal using the `pid` property (available in Ghostty 1.4.0+).
/// Returns Ok(()) on success, or an error describing why it failed.
fn try_focus_by_pid_property(pid: u32) -> Result<(), String> {
    let script = format!(
        r#"
        tell application "Ghostty"
            set matching to every terminal whose pid is {}
            if (count of matching) > 0 then
                activate
                focus item 1 of matching
                return "found"
            end if
        end tell
        return "not found"
    "#,
        pid
    );

    execute_applescript(&script)
}

/// Get the TTY of a process by PID
fn get_tty_for_pid(pid: u32) -> Result<String, String> {
    let output = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "tty="])
        .output()
        .map_err(|e| format!("ps failed: {}", e))?;

    if !output.status.success() {
        return Err("ps exited non-zero".to_string());
    }

    let tty = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if tty.is_empty() || tty == "??" {
        Err(format!("No TTY for pid={}", pid))
    } else {
        Ok(tty)
    }
}

/// Find the PID of the shell process (direct child of Ghostty's login) on a given TTY
fn find_shell_for_tty(tty: &str) -> Result<u32, String> {
    // Find Ghostty's PID
    let output = Command::new("pgrep")
        .args(["-x", "Ghostty"])
        .output()
        .map_err(|e| format!("pgrep failed: {}", e))?;

    let ghostty_pid = String::from_utf8_lossy(&output.stdout)
        .trim()
        .lines()
        .next()
        .ok_or("Ghostty not running")?
        .trim()
        .to_string();

    // List direct children of Ghostty (login processes, each with a TTY)
    let output = Command::new("pgrep")
        .args(["-P", &ghostty_pid])
        .output()
        .map_err(|e| format!("pgrep children failed: {}", e))?;

    let children: Vec<u32> = String::from_utf8_lossy(&output.stdout)
        .trim()
        .lines()
        .filter_map(|l| l.trim().parse().ok())
        .collect();

    info!("Ghostty: {} child processes", children.len());

    // Find the child whose TTY matches
    for child_pid in &children {
        if let Ok(child_tty) = get_tty_for_pid(*child_pid) {
            info!("Ghostty: child pid={} tty={}", child_pid, child_tty);
            if child_tty == tty {
                info!("Ghostty: found matching child pid={} for tty={}", child_pid, tty);
                return Ok(*child_pid);
            }
        }
    }

    Err(format!("No Ghostty child process found for tty={}", tty))
}

/// Get working directory of a process via lsof
fn get_cwd_for_pid(pid: u32) -> Result<String, String> {
    let output = Command::new("lsof")
        .args(["-a", "-p", &pid.to_string(), "-d", "cwd", "-Fn"])
        .output()
        .map_err(|e| format!("lsof failed: {}", e))?;

    if !output.status.success() {
        return Err("lsof exited non-zero".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some(path) = line.strip_prefix('n') {
            return Ok(path.to_string());
        }
    }
    Err(format!("No cwd found for pid={}", pid))
}

/// Focus Ghostty terminal. Tries two strategies:
/// 1. Direct `pid` property matching (Ghostty 1.4.0+, exact match)
/// 2. TTY-based fallback via process tree (Ghostty 1.3.x, resolves to working directory)
pub fn focus_ghostty_by_pid(pid: u32) -> Result<(), String> {
    info!("Ghostty: searching for terminal with pid={}", pid);

    // Strategy 1: Use the `pid` AppleScript property (Ghostty 1.4.0+)
    match try_focus_by_pid_property(pid) {
        Ok(()) => {
            info!("Ghostty: focused via pid property");
            return Ok(());
        }
        Err(e) => {
            // If the error indicates pid property doesn't exist, fall back.
            // Ghostty < 1.4.0 returns "变量"pid"没有定义" (-2753).
            if e.contains("2753") || e.contains("pid") {
                info!("Ghostty: pid property not available, falling back to TTY-based matching: {}", e);
            } else {
                // Tab not found via pid — no match at all
                error!("Ghostty: pid matching failed: {}", e);
                return Err(e);
            }
        }
    }

    // Strategy 2: TTY-based fallback for Ghostty < 1.4.0
    let tty = get_tty_for_pid(pid)?;
    let _shell_pid = find_shell_for_tty(&tty)?;

    // Get the foreground process's working directory to match against Ghostty
    let cwd = get_cwd_for_pid(pid)?;

    let script = format!(
        r#"
        tell application "Ghostty"
            repeat with w in windows
                repeat with t in tabs of w
                    repeat with term in terminals of t
                        if working directory of term contains "{0}" then
                            activate
                            set index of w to 1
                            set selected of t to true
                            focus item 1 of (terminals of t whose working directory contains "{0}")
                            return "found"
                        end if
                    end repeat
                end repeat
            end repeat
        end tell
        return "not found"
    "#,
        cwd
    );

    match execute_applescript(&script) {
        Ok(()) => {
            info!("Ghostty: focused via TTY→cwd fallback (tty={}, cwd={})", tty, cwd);
            Ok(())
        }
        Err(e) => {
            error!("Ghostty: TTY fallback failed for pid={} tty={} cwd={}: {}", pid, tty, cwd, e);
            Err(e)
        }
    }
}
