use super::applescript::execute_applescript;

/// Focus Ghostty terminal by TTY
pub fn focus_ghostty_by_tty(tty: &str) -> Result<(), String> {
    let script = format!(r#"
        tell application "System Events"
            if not (exists process "Ghostty") then
                error "Ghostty not running"
            end if
        end tell

        tell application "Ghostty"
            activate
            repeat with w in windows
                repeat with t in tabs of w
                    repeat with term in terminals of t
                        if tty of term contains "{}" then
                            select tab t
                            focus term
                            return "found"
                        end if
                    end repeat
                end repeat
            end repeat
        end tell
        return "not found"
    "#, tty);

    execute_applescript(&script)
}
