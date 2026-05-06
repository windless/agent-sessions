use super::applescript::execute_applescript;

/// Ghostty does not expose TTY via AppleScript, so TTY-based matching is not possible.
pub fn focus_ghostty_by_tty(_tty: &str) -> Result<(), String> {
    Err("Ghostty does not support TTY matching".to_string())
}

/// Focus Ghostty tab by matching working directory
pub fn focus_ghostty_by_path(path: &str) -> Result<(), String> {
    let dir_name = path.split('/').last().unwrap_or(path);

    let script = format!(r#"
        set targetTabIndex to 0
        tell application "Ghostty"
            repeat with w in windows
                repeat with t in tabs of w
                    repeat with term in terminals of t
                        if working directory of term contains "{0}" then
                            set targetTabIndex to index of t
                            exit repeat
                        end if
                    end repeat
                    if targetTabIndex > 0 then exit repeat
                end repeat
                if targetTabIndex > 0 then exit repeat
            end repeat
        end tell

        if targetTabIndex is 0 then
            return "not found"
        end if

        tell application "Ghostty" to activate
        delay 0.2

        tell application "System Events"
            tell process "Ghostty"
                if targetTabIndex ≤ 9 then
                    keystroke (targetTabIndex as string) using command down
                else
                    keystroke "1" using command down
                    delay 0.1
                    repeat (targetTabIndex - 1) times
                        keystroke "]" using {{command down, shift down}}
                        delay 0.05
                    end repeat
                end if
            end tell
        end tell

        return "found"
    "#, dir_name);

    execute_applescript(&script)
}
