use log::{debug, info, trace, warn};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use once_cell::sync::Lazy;

use crate::agent::AgentProcess;
use super::model::{AgentType, Session, SessionStatus, SessionsResponse, JsonlMessage};
use super::status::{determine_status, has_tool_use, has_tool_result, is_local_slash_command, is_interrupted_request, is_waiting_for_user_input};

/// Track previous status for each session to detect transitions
static PREVIOUS_STATUS: Lazy<Mutex<HashMap<String, SessionStatus>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// Cache git remote URLs by project path — remote URL never changes during app lifetime
static GIT_URL_CACHE: Lazy<Mutex<HashMap<String, Option<String>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// Clean up PREVIOUS_STATUS entries for sessions that no longer exist.
/// Call this after all agent detectors have run to prevent unbounded memory growth.
pub fn cleanup_stale_status_entries(active_session_ids: &std::collections::HashSet<String>) {
    let mut prev_status_map = PREVIOUS_STATUS.lock().unwrap();
    let before_count = prev_status_map.len();
    prev_status_map.retain(|id, _| active_session_ids.contains(id));
    let removed = before_count - prev_status_map.len();
    if removed > 0 {
        debug!("Cleaned up {} stale entries from PREVIOUS_STATUS (kept {})", removed, prev_status_map.len());
    }
}

/// Extract a preview of content for debugging
fn get_content_preview(content: &serde_json::Value) -> String {
    match content {
        serde_json::Value::String(s) => {
            let preview: String = s.chars().take(100).collect();
            format!("text: \"{}{}\"", preview, if s.len() > 100 { "..." } else { "" })
        }
        serde_json::Value::Array(arr) => {
            let types: Vec<String> = arr.iter()
                .filter_map(|v| v.get("type").and_then(|t| t.as_str()).map(String::from))
                .collect();
            format!("blocks: [{}]", types.join(", "))
        }
        _ => "unknown".to_string(),
    }
}

/// Extract the cwd (project path) from a JSONL file.
/// Scans from the beginning of the file for the first valid cwd field.
/// Claude Code writes cwd in every message, so it should appear early.
fn extract_cwd_from_jsonl(jsonl_path: &PathBuf) -> Option<String> {
    let file = File::open(jsonl_path).ok()?;
    let reader = BufReader::new(file);

    // Check first 50 lines for a cwd field
    for line in reader.lines().take(50).flatten() {
        if let Ok(msg) = serde_json::from_str::<JsonlMessage>(&line) {
            if let Some(cwd) = msg.cwd {
                if cwd.starts_with('/') {
                    return Some(cwd);
                }
            }
        }
    }
    None
}

/// Get GitHub URL from a project's git remote origin (cached)
fn get_github_url(project_path: &str) -> Option<String> {
    // Check cache first — avoids spawning a git subprocess on every poll
    {
        let cache = GIT_URL_CACHE.lock().unwrap();
        if let Some(cached) = cache.get(project_path) {
            return cached.clone();
        }
    }

    let result = get_github_url_uncached(project_path);

    // Cache the result (including None) so we don't retry failed lookups
    {
        let mut cache = GIT_URL_CACHE.lock().unwrap();
        cache.insert(project_path.to_string(), result.clone());
    }

    result
}

fn get_github_url_uncached(project_path: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(project_path)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let remote_url = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Convert SSH format to HTTPS
    // git@github.com:user/repo.git -> https://github.com/user/repo
    if remote_url.starts_with("git@github.com:") {
        let path = remote_url
            .strip_prefix("git@github.com:")?
            .strip_suffix(".git")
            .unwrap_or(&remote_url[15..]);
        return Some(format!("https://github.com/{}", path));
    }

    // Already HTTPS format
    // https://github.com/user/repo.git -> https://github.com/user/repo
    if remote_url.starts_with("https://github.com/") {
        let url = remote_url
            .strip_suffix(".git")
            .unwrap_or(&remote_url);
        return Some(url.to_string());
    }

    None
}

/// Convert a file system path like "/Users/ozan/Projects/my-project" to a directory name
/// This is the reverse of convert_dir_name_to_path
/// e.g., "/Users/ozan/Projects/my-project/.rsworktree/branch-name" -> "-Users-ozan-Projects-my-project--rsworktree-branch-name"
pub fn convert_path_to_dir_name(path: &str) -> String {
    // Remove leading slash and replace path separators with dashes
    let path = path.strip_prefix('/').unwrap_or(path);

    let mut result = String::from("-");
    let mut chars = path.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '/' => {
                // Check if next char starts a hidden folder (.)
                if chars.peek() == Some(&'.') {
                    // Hidden folder: use double dash and skip the dot
                    result.push('-');
                    result.push('-');
                    chars.next(); // skip the dot
                } else {
                    result.push('-');
                }
            }
            _ => result.push(c),
        }
    }

    result
}

/// Convert a directory name like "-Users-ozan-Projects-ai-image-dashboard" back to a path
/// The challenge is that both path separators AND project names can contain dashes
/// We handle this by recognizing that the path structure is predictable:
/// /Users/<username>/Projects/<project-name> or /Users/<username>/.../<project-name>
///
/// Special case: Double dashes (--) indicate a hidden folder (starting with .)
/// followed by subfolders separated by single dashes
/// e.g., "ai-image-dashboard--rsworktree-analytics" becomes "ai-image-dashboard/.rsworktree/analytics"
pub fn convert_dir_name_to_path(dir_name: &str) -> String {
    // Remove leading dash if present
    let name = dir_name.strip_prefix('-').unwrap_or(dir_name);

    // Split by dash
    let parts: Vec<&str> = name.split('-').collect();

    if parts.is_empty() {
        return String::new();
    }

    // Find "Projects" or "UnityProjects" index - everything after that is the project name
    let projects_idx = parts.iter().position(|&p| p == "Projects" || p == "UnityProjects");

    if let Some(idx) = projects_idx {
        // Path components are before and including "Projects"
        let path_parts = &parts[..=idx];
        // Project name is everything after "Projects"
        let project_parts = &parts[idx + 1..];

        let mut path = String::from("/");
        path.push_str(&path_parts.join("/"));

        if !project_parts.is_empty() {
            path.push('/');
            // Handle the project path with potential hidden folders
            // Double dash (empty string between dashes when split) indicates hidden folder
            // After a hidden folder marker, subsequent parts are subfolders
            let mut in_hidden_folder = false;
            let mut segments: Vec<String> = Vec::new();
            let mut current_segment = String::new();

            for part in project_parts {
                if part.is_empty() {
                    // Empty part means we hit a double dash - start hidden folder
                    if !current_segment.is_empty() {
                        segments.push(current_segment);
                        current_segment = String::new();
                    }
                    in_hidden_folder = true;
                } else if in_hidden_folder {
                    // After double dash, each part is a subfolder
                    // First part after -- gets the dot prefix
                    if current_segment.is_empty() {
                        current_segment = format!(".{}", part);
                    } else {
                        segments.push(current_segment);
                        current_segment = part.to_string();
                    }
                } else {
                    // Normal project name part - join with dashes
                    if current_segment.is_empty() {
                        current_segment = part.to_string();
                    } else {
                        current_segment.push('-');
                        current_segment.push_str(part);
                    }
                }
            }
            if !current_segment.is_empty() {
                segments.push(current_segment);
            }

            path.push_str(&segments.join("/"));
        }

        path
    } else {
        // Fallback: just replace dashes with slashes (old behavior)
        format!("/{}", name.replace('-', "/"))
    }
}

/// Normalize a CWD path for consistent comparison across sysinfo and JSONL sources.
///
/// Handles two common sources of mismatch:
/// 1. Trailing slashes ("/foo/bar/" vs "/foo/bar")
/// 2. macOS /private prefix on symlinked directories (/tmp vs /private/tmp, /var vs /private/var)
pub(crate) fn normalize_cwd(path: &str) -> String {
    let trimmed = path.trim_end_matches('/');
    #[cfg(target_os = "macos")]
    {
        if let Some(stripped) = trimmed.strip_prefix("/private/") {
            if stripped.starts_with("tmp")
                || stripped.starts_with("var")
                || stripped.starts_with("etc")
            {
                return format!("/{}", stripped);
            }
        }
    }
    trimmed.to_string()
}

/// Convert a process CWD to the path Claude Code uses for its project directory.
///
/// When a Claude Code session runs inside a git worktree (e.g., inside
/// `.claude/worktrees/<name>`), Claude Code stores the JSONL session files
/// under the main repository's encoded path, not the worktree subdirectory.
/// This function resolves the CWD to the path Claude Code actually uses.
pub(crate) fn resolve_project_root(cwd: &str) -> String {
    // If the CWD is inside a Claude Code worktree, use the parent repo path.
    // Worktree paths look like: /path/to/repo/.claude/worktrees/<branch-name>
    if let Some(idx) = cwd.find("/.claude/worktrees/") {
        let parent = &cwd[..idx];
        warn!("  WORKTREE detected: cwd={} -> using parent repo={}", cwd, parent);
        return normalize_cwd(parent);
    }
    normalize_cwd(cwd)
}

/// Match processes to their JSONL session files by checking which files each process
/// has open via `lsof`. Returns a map of PID → JSONL file path.
fn match_processes_to_files(processes: &[&crate::agent::AgentProcess], jsonl_files: &[PathBuf]) -> HashMap<u32, PathBuf> {
    debug!("match_processes_to_files: {} processes, {} jsonl files",
        processes.len(), jsonl_files.len());
    if processes.is_empty() || jsonl_files.is_empty() {
        debug!("  early return: empty input");
        return HashMap::new();
    }

    let jsonl_set: HashSet<&PathBuf> = jsonl_files.iter().collect();
    let pids: Vec<String> = processes.iter().map(|p| p.pid.to_string()).collect();
    let pid_list = pids.join(",");

    debug!("  running: lsof -p {} -Fn -l", pid_list);

    let output = match Command::new("lsof")
        .args(["-p", &pid_list, "-Fn", "-l"])
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            warn!("  lsof error: {:?}", e);
            return HashMap::new();
        }
    };

    debug!("  lsof exit: {}, stdout_len={}, stderr_len={}",
        output.status, output.stdout.len(), output.stderr.len());

    if !output.status.success() {
        warn!("  lsof non-zero exit, stderr: {}", String::from_utf8_lossy(&output.stderr));
        return HashMap::new();
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut current_pid: Option<u32> = None;
    let mut result: HashMap<u32, PathBuf> = HashMap::new();
    let mut jsonl_lines: Vec<String> = Vec::new();

    for line in output_str.lines() {
        if let Some(pid_str) = line.strip_prefix('p') {
            current_pid = pid_str.parse().ok();
        } else if let Some(file_path) = line.strip_prefix('n') {
            let path = PathBuf::from(file_path);
            if path.extension().map_or(false, |e| e == "jsonl") {
                jsonl_lines.push(format!("pid={:?} path={}", current_pid, file_path));
                if jsonl_set.contains(&path) {
                    if let Some(pid) = current_pid {
                        result.entry(pid).or_insert(path);
                    }
                }
            }
        }
    }

    debug!("  lsof found {} total jsonl files, {} matched our set:",
        jsonl_lines.len(), result.len());
    for line in &jsonl_lines {
        warn!("    {}", line);
    }

    result
}

/// Get all active Claude Code sessions (delegates to agent module)
pub fn get_sessions() -> SessionsResponse {
    crate::agent::get_all_sessions()
}

/// Internal function to get sessions for a specific agent type
/// Called by agent detectors (ClaudeDetector, OpenCodeDetector, etc.)
pub fn get_sessions_internal(processes: &[AgentProcess], agent_type: AgentType) -> Vec<Session> {
    info!("=== get_sessions_internal for {:?}: {} processes ===", agent_type, processes.len());
    for p in processes {
        debug!("  PROCESS: pid={}, cwd={:?}, cpu={:.1}%",
            p.pid, p.cwd, p.cpu_usage);
    }

    let mut sessions = Vec::new();

    // Build a map of cwd -> list of processes (multiple sessions can run in same folder)
    let mut cwd_to_processes: HashMap<String, Vec<&AgentProcess>> = HashMap::new();
    // Pre-compute expected project directory names from process CWDs.
    // This lets us skip scanning directories that can't match any running process.
    let mut expected_dir_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    // Reverse mapping: directory name → actual CWD path.
    // convert_dir_name_to_path() is lossy (dashes in path segments vs separators are ambiguous),
    // so when extract_cwd_from_jsonl fails, we look up the real CWD from this map instead.
    let mut dir_name_to_cwd: HashMap<String, String> = HashMap::new();
    for process in processes {
        if let Some(cwd) = &process.cwd {
            let cwd_raw = cwd.to_string_lossy().to_string();
            // Use the RAW CWD (not resolved) for directory name matching.
            // Claude Code stores worktree session files in a SEPARATE encoded
            // directory (e.g., -repo--claude-worktrees-branch), not under the
            // main repo directory. resolve_project_root would lose the worktree
            // suffix, causing us to scan the wrong directory for JSONL files.
            let dir_name = convert_path_to_dir_name(&cwd_raw);
            let project_root = resolve_project_root(&cwd_raw);
            debug!("  MAP: pid={} raw_cwd={:?} resolved_root={} dir={}",
                process.pid, cwd_raw, project_root, dir_name);
            expected_dir_names.insert(dir_name.clone());
            dir_name_to_cwd.insert(dir_name, cwd_raw.clone());
            // Use the raw (normalized) CWD as the matching key so both worktree
            // and main-repo processes match against JSONL files in their own directory.
            cwd_to_processes.entry(normalize_cwd(&cwd_raw)).or_default().push(process);
        } else {
            warn!("Process pid={} has no cwd, skipping", process.pid);
        }
    }

    debug!("cwd_to_processes has {} entries:", cwd_to_processes.len());
    for (cwd, procs) in &cwd_to_processes {
        debug!("  CWD={} -> {} processes: {:?}",
            cwd, procs.len(), procs.iter().map(|p| p.pid).collect::<Vec<_>>());
    }
    debug!("expected_dir_names: {:?}", expected_dir_names);

    // Scan ~/.claude/projects for session files
    let claude_dir = dirs::home_dir()
        .map(|h| h.join(".claude").join("projects"))
        .unwrap_or_default();

    debug!("Claude projects directory: {:?}", claude_dir);

    if !claude_dir.exists() {
        warn!("Claude projects directory does not exist: {:?}", claude_dir);
        return sessions;
    }

    // For each project directory
    if let Ok(entries) = fs::read_dir(&claude_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let dir_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            trace!("Checking project dir: {} (expected_set has it: {})",
                dir_name, expected_dir_names.contains(dir_name));

            // Skip directories that can't match any running process.
            // This avoids opening hundreds of JSONL files in inactive project directories.
            if !expected_dir_names.contains(dir_name) {
                trace!("  SKIP: dir {} not in expected set", dir_name);
                continue;
            }

            // Get all recent JSONL files and extract cwd from each.
            // Multiple real paths can collide into the same encoded directory
            // (e.g., agent-sessions and agent/sessions both encode to -...-agent-sessions)
            // so we need to match each file's cwd to the process's cwd individually.
            let jsonl_files = get_recently_active_jsonl_files(&path, 100);
            debug!("  Found {} jsonl files: {:?}",
                jsonl_files.len(),
                jsonl_files.iter().map(|f| f.file_name().unwrap_or_default().to_string_lossy()).collect::<Vec<_>>());
            if jsonl_files.is_empty() {
                trace!("  SKIP: no JSONL files in dir {}", dir_name);
                continue;
            }

            // Build a map of normalized cwd -> list of JSONL files with that cwd
            let mut cwd_to_files: HashMap<String, Vec<PathBuf>> = HashMap::new();
            for jsonl_file in &jsonl_files {
                let raw_cwd = extract_cwd_from_jsonl(jsonl_file);
                // Use the raw CWD from JSONL as-is (no resolve_project_root).
                // Claude Code stores worktree session JSONL files in a separate
                // directory that matches the full worktree path encoding. If we
                // resolved the CWD here, we'd look in the main repo directory
                // and miss the worktree's own session files entirely.
                let file_cwd = raw_cwd.clone()
                    .or_else(|| dir_name_to_cwd.get(dir_name).cloned())
                    .unwrap_or_else(|| convert_dir_name_to_path(dir_name));
                let normalized = normalize_cwd(&file_cwd);
                trace!("    FILE: {:?} -> raw_cwd={:?} normalized={}",
                    jsonl_file.file_name().unwrap_or_default(),
                    raw_cwd, normalized);
                cwd_to_files.entry(normalized).or_default().push(jsonl_file.clone());
            }

            debug!("  cwd_to_files has {} entries:", cwd_to_files.len());
            for (cwd, files) in &cwd_to_files {
                debug!("    CWD={} -> {} files: {:?}",
                    cwd, files.len(),
                    files.iter().map(|f| f.file_name().unwrap_or_default().to_string_lossy()).collect::<Vec<_>>());
            }

            // For each unique cwd, find matching processes and create sessions
            for (project_path, files_for_cwd) in &cwd_to_files {
                debug!("  Looking for processes with cwd={}", project_path);
                let matching_processes = match cwd_to_processes.get(project_path) {
                    Some(procs) => {
                        debug!("    FOUND {} matching processes: {:?}",
                            procs.len(), procs.iter().map(|p| p.pid).collect::<Vec<_>>());
                        procs
                    }
                    None => {
                        warn!("    NO matching processes (cwd key not in cwd_to_processes)");
                        continue;
                    }
                };

                // Match processes to JSONL files by checking open file descriptors.
                // Falls back to index-based matching when lsof is unavailable.
                let pid_to_file = match_processes_to_files(matching_processes, files_for_cwd);
                let has_lsof_results = !pid_to_file.is_empty();
                debug!("    lsof results: {} matches, has_results={}",
                    pid_to_file.len(), has_lsof_results);
                for (pid, file) in &pid_to_file {
                    debug!("      lsof: pid={} -> {:?}", pid, file.file_name().unwrap_or_default());
                }

                for (index, process) in matching_processes.iter().enumerate() {
                    // Try lsof-based match first, then fall back to index-based
                    let jsonl_path = if let Some(file) = pid_to_file.get(&process.pid) {
                        debug!("    MATCH lsof: pid={} index={} -> {:?}", process.pid, index, file.file_name().unwrap_or_default());
                        Some(file.clone())
                    } else if has_lsof_results {
                        // lsof ran successfully but didn't find this PID — no file to match
                        warn!("    SKIP pid={}: lsof ran but found no jsonl for this pid", process.pid);
                        continue;
                    } else {
                        // lsof unavailable, fall back to index-based matching
                        let fallback = files_for_cwd.get(index).cloned();
                        debug!("    MATCH fallback: pid={} index={} -> {:?}",
                            process.pid, index,
                            fallback.as_ref().and_then(|f| f.file_name()));
                        fallback
                    };

                    let Some(jsonl_path) = jsonl_path else {
                        warn!("    SKIP pid={}: no jsonl_path (index {} >= files.len {})",
                            process.pid, index, files_for_cwd.len());
                        continue;
                    };

                    if let Some(session) = find_session_for_process(
                        &jsonl_path,
                        &path,
                        &resolve_project_root(project_path),
                        process,
                        agent_type.clone(),
                    ) {
                        debug!("    SESSION CREATED: id={} pid={} project={} status={:?}",
                            session.id, session.pid, session.project_name, session.status);
                        // Track status transitions
                        let mut prev_status_map = PREVIOUS_STATUS.lock().unwrap();
                        let prev_status = prev_status_map.get(&session.id).cloned();

                        // Log status transition if it changed
                        if let Some(prev) = &prev_status {
                            if *prev != session.status {
                                warn!(
                                    "STATUS TRANSITION: project={}, {:?} -> {:?}, cpu={:.1}%, file_age=?, last_msg_role={:?}",
                                    session.project_name, prev, session.status, session.cpu_usage, session.last_message_role
                                );
                            }
                        }

                        // Update stored status
                        prev_status_map.insert(session.id.clone(), session.status.clone());
                        drop(prev_status_map);

                        info!(
                            "Session created: id={}, project={}, status={:?}, pid={}, cpu={:.1}%",
                            session.id, session.project_name, session.status, session.pid, session.cpu_usage
                        );
                        sessions.push(session);
                    } else {
                        warn!("    PARSE FAILED: pid={} file={:?} — parse_session_file returned None",
                            process.pid, jsonl_path.file_name().unwrap_or_default());
                    }
                }
            }
        }
    }

    warn!(
        "=== get_sessions_internal complete for {:?}: {} sessions ===",
        agent_type, sessions.len()
    );
    for s in &sessions {
        warn!("  SESSION: id={} pid={} project={} status={:?}",
            s.id, s.pid, s.project_name, s.status);
    }

    sessions
}

/// Check if a JSONL file is a subagent file (named agent-*.jsonl)
fn is_subagent_file(path: &PathBuf) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|name| name.starts_with("agent-") && name.ends_with(".jsonl"))
        .unwrap_or(false)
}

/// Extract sessionId from a subagent JSONL file by reading the first few lines
fn get_subagent_session_id(path: &PathBuf) -> Option<String> {
    let file = File::open(path).ok()?;
    let reader = BufReader::new(file);

    // Check first 5 lines for sessionId
    for line in reader.lines().take(5).flatten() {
        if let Ok(msg) = serde_json::from_str::<JsonlMessage>(&line) {
            if let Some(session_id) = msg.session_id {
                return Some(session_id);
            }
        }
    }
    None
}

/// Count active subagents for a given parent session
fn count_active_subagents(project_dir: &PathBuf, parent_session_id: &str) -> usize {
    use std::time::{Duration, SystemTime};

    let active_threshold = Duration::from_secs(30);
    let now = SystemTime::now();

    let count = fs::read_dir(project_dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| is_subagent_file(&e.path()))
        .filter(|e| {
            // Check if file was recently modified
            e.metadata()
                .and_then(|m| m.modified())
                .ok()
                .and_then(|modified| now.duration_since(modified).ok())
                .map(|d| d < active_threshold)
                .unwrap_or(false)
        })
        .filter(|e| {
            // Check if sessionId matches parent
            get_subagent_session_id(&e.path())
                .map(|id| id == parent_session_id)
                .unwrap_or(false)
        })
        .count();

    trace!("Found {} active subagents for session {}", count, parent_session_id);
    count
}

/// Get JSONL files for a project, sorted by modification time (newest first)
/// Excludes subagent files (agent-*.jsonl) as they are counted separately
fn get_recently_active_jsonl_files(project_dir: &PathBuf, _expected_count: usize) -> Vec<PathBuf> {
    let mut jsonl_files: Vec<_> = fs::read_dir(project_dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| {
            let path = e.path();
            path.extension()
                .map(|ext| ext == "jsonl")
                .unwrap_or(false)
                && !is_subagent_file(&path)
        })
        .filter_map(|e| {
            let path = e.path();
            let modified = e.metadata().and_then(|m| m.modified()).ok()?;
            Some((path, modified))
        })
        .collect();

    // Sort by modification time (newest first)
    jsonl_files.sort_by(|a, b| b.1.cmp(&a.1));

    jsonl_files
        .into_iter()
        .map(|(path, _)| path)
        .collect()
}

/// Create a session for a process from a specific JSONL file
fn find_session_for_process(
    jsonl_path: &PathBuf,
    project_dir: &PathBuf,
    project_path: &str,
    process: &crate::agent::AgentProcess,
    agent_type: AgentType,
) -> Option<Session> {
    let mut session = parse_session_file(jsonl_path, project_path, process.pid, process.cpu_usage, agent_type)?;

    // Count active subagents for this session
    session.active_subagent_count = count_active_subagents(project_dir, &session.id);

    Some(session)
}

/// Parse a JSONL session file and create a Session struct
pub fn parse_session_file(
    jsonl_path: &PathBuf,
    project_path: &str,
    pid: u32,
    cpu_usage: f32,
    agent_type: AgentType,
) -> Option<Session> {
    use std::time::SystemTime;

    debug!("Parsing JSONL file: {:?}", jsonl_path);

    // Check if the file was modified very recently (indicates active processing)
    let file_age_secs = jsonl_path
        .metadata()
        .and_then(|m| m.modified())
        .ok()
        .and_then(|modified| SystemTime::now().duration_since(modified).ok())
        .map(|d| d.as_secs_f32());

    let file_recently_modified = file_age_secs.map(|age| age < 3.0).unwrap_or(false);

    debug!(
        "File age: {:.1}s, recently_modified: {}",
        file_age_secs.unwrap_or(-1.0),
        file_recently_modified
    );

    // Parse the JSONL file to get session info
    let file = File::open(jsonl_path).ok()?;
    let file_size = file.metadata().ok().map(|m| m.len()).unwrap_or(0);
    let mut reader = BufReader::new(file);

    let mut session_id = None;
    let mut git_branch = None;
    let mut last_timestamp = None;
    let mut last_message = None;
    let mut last_role = None;
    let mut last_msg_type = None;
    let mut last_has_tool_use = false;
    let mut last_has_tool_result = false;
    let mut last_is_local_command = false;
    let mut last_is_interrupted = false;
    let mut last_is_user_input_tool = false;
    let mut found_status_info = false;
    let mut is_compacting = false;

    // Read last N lines for efficiency
    // Must be large enough to cover long stretches of progress entries during tool execution
    // (observed up to 275 consecutive non-content lines in real sessions)
    //
    // For large files, seek to near the end instead of reading the entire file.
    // 512KB is more than enough for 500 JSONL lines.
    const TAIL_BYTES: u64 = 512 * 1024;
    if file_size > TAIL_BYTES {
        let _ = reader.seek(SeekFrom::End(-(TAIL_BYTES as i64)));
        // Discard partial line after seeking into the middle of the file
        let mut _partial = String::new();
        let _ = reader.read_line(&mut _partial);
    }

    let lines: Vec<_> = reader.lines().flatten().collect();
    let recent_lines: Vec<_> = lines.iter().rev().take(500).collect();

    trace!("File has {} total lines, checking last {}", lines.len(), recent_lines.len());

    for line in &recent_lines {
        if let Ok(msg) = serde_json::from_str::<JsonlMessage>(line) {
            if session_id.is_none() {
                session_id = msg.session_id;
            }
            if git_branch.is_none() {
                git_branch = msg.git_branch;
            }
            if last_timestamp.is_none() {
                last_timestamp = msg.timestamp;
            }

            // Detect compaction: if we see compact_boundary before any content message
            // or isCompactSummary, the session is currently compacting.
            // Reading from newest to oldest: if compact_boundary comes first → compacting
            // If isCompactSummary comes first → compaction already finished
            if !found_status_info && !is_compacting {
                if msg.is_compact_summary == Some(true) {
                    // Compaction finished, summary already written - not compacting
                    // Continue to find status info normally
                } else if msg.subtype.as_deref() == Some("compact_boundary") {
                    is_compacting = true;
                    debug!("Detected active compaction (compact_boundary before any content)");
                }
            }

            // For status detection, we need to find the most recent message that has CONTENT
            if !found_status_info {
                if let Some(content) = &msg.message {
                    if let Some(c) = &content.content {
                        let has_content = match c {
                            serde_json::Value::String(s) => !s.is_empty(),
                            serde_json::Value::Array(arr) => !arr.is_empty(),
                            _ => false,
                        };

                        if has_content {
                            last_msg_type = msg.msg_type.clone();
                            last_role = content.role.clone();
                            last_has_tool_use = has_tool_use(c);
                            last_has_tool_result = has_tool_result(c);
                            last_is_local_command = is_local_slash_command(c);
                            last_is_interrupted = is_interrupted_request(c);
                            last_is_user_input_tool = is_waiting_for_user_input(c);
                            found_status_info = true;

                            // Enhanced logging with content preview
                            let content_preview = get_content_preview(c);
                            debug!(
                                "Found status info: type={:?}, role={:?}, has_tool_use={}, has_tool_result={}, is_local_cmd={}, is_interrupted={}, is_user_input={}, content={}",
                                last_msg_type, last_role, last_has_tool_use, last_has_tool_result, last_is_local_command, last_is_interrupted, last_is_user_input_tool, content_preview
                            );
                        }
                    }
                }
            }

            if session_id.is_some() && found_status_info {
                break;
            }
        }
    }

    // Now find the last meaningful text message (keep looking even after finding status)
    for line in &recent_lines {
        if let Ok(msg) = serde_json::from_str::<JsonlMessage>(line) {
            if let Some(content) = &msg.message {
                if let Some(c) = &content.content {
                    let text = match c {
                        serde_json::Value::String(s) if !s.is_empty() => Some(s.clone()),
                        serde_json::Value::Array(arr) => {
                            arr.iter().find_map(|v| {
                                v.get("text").and_then(|t| t.as_str())
                                    .filter(|s| !s.is_empty())
                                    .map(String::from)
                            })
                        }
                        _ => None,
                    };

                    if text.is_some() {
                        last_message = text;
                        break;
                    }
                }
            }
        }
    }

    let session_id = session_id?;

    // Determine status based on message content — no file age or CPU heuristics
    let status = if is_compacting {
        SessionStatus::Compacting
    } else {
        determine_status(
            last_msg_type.as_deref(),
            last_has_tool_use,
            last_has_tool_result,
            last_is_local_command,
            last_is_interrupted,
            last_is_user_input_tool,
            file_recently_modified,
        )
    };

    debug!(
        "Status determination: type={:?}, tool_use={}, tool_result={}, local_cmd={}, interrupted={}, user_input={}, recent={}, compacting={}, file_age={:.1}s -> {:?}",
        last_msg_type, last_has_tool_use, last_has_tool_result, last_is_local_command, last_is_interrupted, last_is_user_input_tool, file_recently_modified, is_compacting, file_age_secs.unwrap_or(-1.0), status
    );

    // Extract project name from path
    let project_name = project_path
        .split('/')
        .filter(|s| !s.is_empty())
        .last()
        .unwrap_or("Unknown")
        .to_string();

    // Truncate message for preview (respecting UTF-8 char boundaries)
    let last_message = last_message.map(|m| {
        if m.chars().count() > 100 {
            format!("{}...", m.chars().take(100).collect::<String>())
        } else {
            m
        }
    });

    // Get GitHub URL from git remote
    let github_url = get_github_url(project_path);

    Some(Session {
        id: session_id,
        agent_type,
        project_name,
        project_path: project_path.to_string(),
        git_branch,
        github_url,
        status,
        last_message,
        last_message_role: last_role,
        last_activity_at: last_timestamp.unwrap_or_else(|| "Unknown".to_string()),
        pid,
        cpu_usage,
        active_subagent_count: 0, // Set by find_session_for_process
    })
}
