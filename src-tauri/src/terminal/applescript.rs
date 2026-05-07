use log::{info, error};
use std::process::Command;

/// Execute an AppleScript and return Ok if successful
pub fn execute_applescript(script: &str) -> Result<(), String> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| format!("Failed to execute AppleScript: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    info!("AppleScript stdout: {:?}, stderr: {:?}", stdout, stderr);

    if output.status.success() {
        if stdout == "not found" {
            error!("AppleScript returned 'not found'");
            Err("Tab not found".to_string())
        } else {
            Ok(())
        }
    } else {
        error!("AppleScript failed with status {:?}: {}", output.status, stderr);
        Err(format!("AppleScript error: {}", stderr))
    }
}
