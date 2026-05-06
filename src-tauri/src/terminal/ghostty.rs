use super::applescript::execute_applescript;

/// Ghostty does not expose TTY via AppleScript, so TTY-based matching is not possible.
pub fn focus_ghostty_by_tty(_tty: &str) -> Result<(), String> {
    Err("Ghostty does not support TTY matching".to_string())
}

/// Focus Ghostty tab by matching working directory
pub fn focus_ghostty_by_path(path: &str) -> Result<(), String> {
    let path = path.trim_end_matches('/');
    let dir_name = path.split('/').last().unwrap_or(path);

    // Escape backslashes and double quotes for AppleScript string safety
    let dir_name_escaped = dir_name.replace('\\', "\\\\").replace('"', "\\\"");

    let script = format!(
        r#"
        tell application "Ghostty"
            -- First pass: match by working directory
            repeat with w in windows
                repeat with t in tabs of w
                    repeat with term in terminals of t
                        try
                            set wd to working directory of term
                        on error
                            set wd to ""
                        end try
                        if wd is not "" and (wd ends with "/{0}" or wd ends with "{0}") then
                            activate
                            select tab t
                            focus term
                            return "found:wd"
                        end if
                    end repeat
                end repeat
            end repeat
            -- Second pass: match by terminal title (often contains dir name)
            repeat with w in windows
                repeat with t in tabs of w
                    repeat with term in terminals of t
                        if name of term contains "{0}" then
                            activate
                            select tab t
                            focus term
                            return "found:title"
                        end if
                    end repeat
                end repeat
            end repeat
        end tell
        return "not found"
    "#,
        dir_name_escaped
    );

    execute_applescript(&script)
}
