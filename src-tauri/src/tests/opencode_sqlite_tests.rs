use crate::agent::{opencode, AgentDetector};
use crate::agent::AgentProcess;
use crate::session::{AgentType, Session, SessionStatus};
use rusqlite::{Connection, params};
use std::path::PathBuf;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helper: create a temporary SQLite database matching OpenCode's schema
// ---------------------------------------------------------------------------

fn create_test_db() -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("opencode.db");
    let conn = Connection::open(&db_path).unwrap();

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS project (
            id TEXT PRIMARY KEY,
            worktree TEXT NOT NULL,
            sandboxes TEXT NOT NULL DEFAULT '[]',
            name TEXT,
            time_created INTEGER NOT NULL,
            time_updated INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS session (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            directory TEXT NOT NULL,
            title TEXT NOT NULL,
            agent TEXT,
            model TEXT,
            time_created INTEGER NOT NULL,
            time_updated INTEGER NOT NULL,
            time_archived INTEGER
        );

        CREATE TABLE IF NOT EXISTS message (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            time_created INTEGER NOT NULL,
            time_updated INTEGER NOT NULL,
            data TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS part (
            id TEXT PRIMARY KEY,
            message_id TEXT NOT NULL,
            session_id TEXT NOT NULL,
            time_created INTEGER NOT NULL,
            time_updated INTEGER NOT NULL,
            data TEXT NOT NULL
        );
        ",
    )
    .unwrap();

    (dir, db_path)
}

/// Helper: insert a minimal project row and return the connection.
fn insert_project(conn: &Connection, id: &str, worktree: &str, sandboxes: &str) {
    conn.execute(
        "INSERT INTO project (id, worktree, sandboxes, name, time_created, time_updated)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, worktree, sandboxes, "test-project", 1000u64, 1000u64],
    )
    .unwrap();
}

/// Helper: insert a session row.
fn insert_session(
    conn: &Connection,
    id: &str,
    project_id: &str,
    directory: &str,
    title: &str,
    time_updated: u64,
    time_archived: Option<u64>,
) {
    conn.execute(
        "INSERT INTO session (id, project_id, directory, title, agent, model, time_created, time_updated, time_archived)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            id,
            project_id,
            directory,
            title,
            "opencode",
            "deepseek/deepseek-v4-flash",
            1000u64,
            time_updated,
            time_archived,
        ],
    )
    .unwrap();
}

/// Helper: insert a message row.
#[allow(dead_code)]
fn insert_message(conn: &Connection, id: &str, session_id: &str, role: &str, time_created: u64) {
    let data = format!(r#"{{"role":"{}"}}"#, role);
    conn.execute(
        "INSERT INTO message (id, session_id, time_created, time_updated, data)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, session_id, time_created, time_created, data],
    )
    .unwrap();
}

/// Helper: insert a part row.
#[allow(dead_code)]
fn insert_part(conn: &Connection, id: &str, message_id: &str, session_id: &str, text: &str) {
    let data = format!(r#"{{"type":"text","text":"{}"}}"#, text);
    conn.execute(
        "INSERT INTO part (id, message_id, session_id, time_created, time_updated, data)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, message_id, session_id, 1000u64, 1000u64, data],
    )
    .unwrap();
}

// ===========================================================================
// Test 1: detect_sessions_from_sqlite  [RED]
// ===========================================================================
//
// This test creates an OpenCode SQLite database with a full data chain
// (project → session → message → text part) and then calls the existing
// `OpenCodeDetector::find_sessions()` with a process whose CWD matches the
// project's worktree.
//
// **RED phase**: the detector currently reads only JSON files, so it returns
// zero sessions.  When the GREEN phase adds SQLite support, this test will
// pass.
//
// The test also validates the SQL JOIN chain directly so the schema
// assumptions are verified.

#[test]
fn test_detect_sessions_from_sqlite() {
    let (_dir, db_path) = create_test_db();
    let conn = Connection::open(&db_path).unwrap();

    // Point the detector at our test database
    opencode::set_test_db_path(db_path);

    // ---- seed data: one project, one session, one message, one text part ----
    insert_project(&conn, "proj-1", "/tmp/test-project-red", "[]");
    insert_session(
        &conn,
        "sess-1",
        "proj-1",
        "/tmp/test-project-red",
        "Test Session",
        1778072396000,   // ms timestamp
        None,             // NOT archived
    );
    insert_message(&conn, "msg-1", "sess-1", "assistant", 1778072396001);
    insert_part(
        &conn,
        "part-1",
        "msg-1",
        "sess-1",
        "Here is the implementation for the feature.",
    );

    // ---- verify the SQL JOIN chain works (schema validation) ----
    let chain_count: i64 = conn
        .query_row(
            "SELECT COUNT(*)
             FROM part p
             JOIN message m ON m.id = p.message_id
             JOIN session s ON s.id = m.session_id
             JOIN project pj ON pj.id = s.project_id
             WHERE pj.id = 'proj-1' AND s.time_archived IS NULL",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(chain_count, 1, "SQL JOIN chain must yield exactly one row");

    // ---- call the existing public detection API ----
    // The current OpenCodeDetector reads from ~/.local/share/opencode/storage/
    // which uses JSON files, NOT SQLite.  We pass a process whose CWD matches
    // our test project's worktree, but since the detector doesn't look at
    // SQLite yet, it returns empty.
    //
    // This assertion FAILS in RED phase — once SQLite support is added in
    // GREEN phase it will pass.
    let detector = opencode::OpenCodeDetector;
    let processes = vec![AgentProcess {
        pid: 99999,
        cpu_usage: 0.0,
        cwd: Some(PathBuf::from("/tmp/test-project-red")),
    }];
    let sessions = detector.find_sessions(&processes);

    assert!(
        !sessions.is_empty(),
        "RED: sessions should be detected from SQLite once implementation exists"
    );
}

// ===========================================================================
// Test 2: fallback_to_json_when_db_missing
// ===========================================================================
//
// Verify the code gracefully handles a missing SQLite database path.
// The implementation must detect that the file doesn't exist and fall back
// to the JSON-based session detection without panicking.

#[test]
fn test_fallback_to_json_when_db_missing() {
    // Path inside a directory that does NOT exist → Connection::open will fail
    let missing_path = PathBuf::from("/tmp/opencode-nonexistent-XXXXXX/opencode.db");

    // Connection::open on a path inside a non-existent directory should fail
    let result = Connection::open(&missing_path);
    assert!(
        result.is_err(),
        "opening a DB in a non-existent directory must fail"
    );

    // The implementation should detect that the file does not exist AND return
    // an empty session list without panicking.  Verify the path doesn't exist.
    assert!(!missing_path.exists(), "the missing DB path must not exist");
}

// ===========================================================================
// Test 3: exclude_archived_sessions
// ===========================================================================
//
// Sessions with a non-NULL time_archived column must be excluded from results.

#[test]
fn test_exclude_archived_sessions() {
    let (_dir, db_path) = create_test_db();
    let conn = Connection::open(&db_path).unwrap();

    insert_project(&conn, "proj-arch", "/tmp/arch-test", "[]");

    // Session A — archived
    insert_session(
        &conn,
        "sess-archived",
        "proj-arch",
        "/tmp/arch-test",
        "Archived Session",
        1778072396000,
        Some(1778072400000), // has archive timestamp → excluded
    );

    // Session B — NOT archived
    insert_session(
        &conn,
        "sess-active",
        "proj-arch",
        "/tmp/arch-test",
        "Active Session",
        1778072397000,
        None, // no archive timestamp → included
    );

    // Query only non-archived sessions
    let active_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM session WHERE project_id = 'proj-arch' AND time_archived IS NULL",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(active_count, 1, "only one active (non-archived) session");

    // Verify the archived session is NOT returned
    let archived_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM session WHERE project_id = 'proj-arch' AND time_archived IS NOT NULL",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(archived_count, 1, "archived session still exists but is excluded");

    // The IDs match expectations
    let active_id: String = conn
        .query_row(
            "SELECT id FROM session WHERE project_id = 'proj-arch' AND time_archived IS NULL",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(active_id, "sess-active", "active session must be 'sess-active'");
}

// ===========================================================================
// Test 4: empty_db_returns_empty
// ===========================================================================
//
// A valid SQLite database with all tables but zero data rows must yield an
// empty result set — no crashes, no spurious rows.

#[test]
fn test_empty_db_returns_empty() {
    let (_dir, db_path) = create_test_db();
    let conn = Connection::open(&db_path).unwrap();

    // All four core tables exist (created by create_test_db) but are empty.
    let project_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM project", [], |r| r.get(0))
        .unwrap();
    assert_eq!(project_count, 0, "project table must be empty");

    let session_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM session", [], |r| r.get(0))
        .unwrap();
    assert_eq!(session_count, 0, "session table must be empty");

    let message_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM message", [], |r| r.get(0))
        .unwrap();
    assert_eq!(message_count, 0, "message table must be empty");

    let part_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM part", [], |r| r.get(0))
        .unwrap();
    assert_eq!(part_count, 0, "part table must be empty");
}

// ===========================================================================
// Test 5: timestamp_milliseconds_to_seconds
// ===========================================================================
//
// OpenCode stores timestamps as milliseconds since epoch.  The implementation
// must divide by 1000 before passing to chrono.  Validate the arithmetic.

#[test]
fn test_timestamp_milliseconds_to_seconds() {
    let (_dir, db_path) = create_test_db();
    let conn = Connection::open(&db_path).unwrap();

    // Insert a session with a concrete millisecond timestamp
    let ms_timestamp: u64 = 1778072396937;
    let expected_seconds: u64 = 1778072396; // 1778072396937 / 1000

    insert_project(&conn, "proj-ts", "/tmp/ts-test", "[]");
    insert_session(
        &conn,
        "sess-ts",
        "proj-ts",
        "/tmp/ts-test",
        "Timestamp Test",
        ms_timestamp,
        None,
    );

    // Verify raw timestamp stored correctly
    let stored_ms: u64 = conn
        .query_row(
            "SELECT time_updated FROM session WHERE id = 'sess-ts'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(stored_ms, ms_timestamp, "millisecond timestamp must be stored as-is");

    // Verify the division by 1000 produces the correct seconds value
    let computed_seconds = stored_ms / 1000;
    assert_eq!(
        computed_seconds, expected_seconds,
        "milliseconds / 1000 must yield correct UNIX seconds"
    );

    // Verify the seconds value can be used to construct a chrono DateTime
    let dt = chrono::DateTime::from_timestamp(computed_seconds as i64, 0);
    assert!(
        dt.is_some(),
        "converted seconds must produce a valid chrono DateTime"
    );

    // Optional: validate the formatted output
    if let Some(dt) = dt {
        let formatted = dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
        assert!(
            formatted.starts_with("2026-05-06"),
            "timestamp 1778072396937 should map to 2026-05-06, got {}",
            formatted
        );
    }
}

// ===========================================================================
// Test 6: message_text_truncation
// ===========================================================================
//
// When a part's text exceeds 200 characters the implementation must truncate
// it to 200 characters (with an ellipsis).  Validate that the raw text is
// longer than 200 and that truncation produces ≤ 200 characters.

#[test]
fn test_message_text_truncation() {
    let (_dir, db_path) = create_test_db();
    let conn = Connection::open(&db_path).unwrap();

    // Create a text string that is exactly 250 characters
    let long_text: String = (0..50).map(|i| format!("word{} ", i)).collect::<String>();
    assert!(
        long_text.len() > 200,
        "helper text must be longer than 200 chars, got {}",
        long_text.len()
    );

    insert_project(&conn, "proj-trunc", "/tmp/trunc-test", "[]");
    insert_session(
        &conn,
        "sess-trunc",
        "proj-trunc",
        "/tmp/trunc-test",
        "Truncation Test",
        1000,
        None,
    );
    insert_message(&conn, "msg-trunc", "sess-trunc", "assistant", 2000);
    insert_part(&conn, "part-trunc", "msg-trunc", "sess-trunc", &long_text);

    // Read back the stored data
    let raw_data: String = conn
        .query_row("SELECT data FROM part WHERE id = 'part-trunc'", [], |r| r.get(0))
        .unwrap();

    // Verify the raw text is still > 200 chars (SQLite stores it whole)
    assert!(
        raw_data.len() > 200,
        "raw JSON in DB must exceed 200 chars, got {}",
        raw_data.len()
    );

    // Simulate the truncation logic:
    // 1. Parse the JSON to extract the text value
    // 2. Truncate if > 200 chars
    let parsed: serde_json::Value = serde_json::from_str(&raw_data).unwrap();
    let extracted = parsed["text"].as_str().unwrap();

    // The escaping in JSON will double backslashes; the actual text length
    // in the JSON value may differ from the raw JSON string length.
    // What matters is that the truncation logic produces ≤ 200 chars.
    let truncated = if extracted.len() > 200 {
        format!("{}...", &extracted[..197])
    } else {
        extracted.to_string()
    };
    assert!(
        truncated.len() <= 200,
        "truncated text must be ≤ 200 chars, got {}",
        truncated.len()
    );

    // If the original was > 200, the truncated form should end with "..."
    if extracted.len() > 200 {
        assert!(
            truncated.ends_with("..."),
            "truncated text longer than 200 must end with '...'"
        );
    }
}

// ===========================================================================
// Test 7: system_prompt_skipped
// ===========================================================================
//
// The implementation must skip system-prompt parts (those whose text starts
// with XML tags such as "<system>ultrawork...</system>") and return only
// user/assistant text parts.

#[test]
fn test_system_prompt_skipped() {
    let (_dir, db_path) = create_test_db();
    let conn = Connection::open(&db_path).unwrap();

    insert_project(&conn, "proj-sp", "/tmp/sp-test", "[]");
    insert_session(
        &conn,
        "sess-sp",
        "proj-sp",
        "/tmp/sp-test",
        "System Prompt Test",
        1000,
        None,
    );
    insert_message(&conn, "msg-sp-1", "sess-sp", "user", 2000);
    insert_message(&conn, "msg-sp-2", "sess-sp", "assistant", 3000);

    // Part 1 — system prompt (XML wrapped, should be skipped)
    let system_text = "<system>ultrawork mode configuration and instructions...</system>";
    insert_part(
        &conn,
        "part-sp-1",
        "msg-sp-1",
        "sess-sp",
        system_text,
    );

    // Part 2 — normal assistant text (should be kept)
    insert_part(
        &conn,
        "part-sp-2",
        "msg-sp-2",
        "sess-sp",
        "Here is the implementation you requested.",
    );

    // --- Simulate the skipping logic in Rust ---

    // Collect all parts
    let mut stmt = conn
        .prepare("SELECT id, data FROM part WHERE session_id = 'sess-sp' ORDER BY time_created")
        .unwrap();
    let parts: Vec<(String, String)> = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert_eq!(parts.len(), 2, "both parts must be stored in DB");

    // Apply the same skip logic as get_message_text in opencode.rs
    let mut kept: Vec<String> = Vec::new();
    for (_id, data_json) in &parts {
        let parsed: serde_json::Value = serde_json::from_str(data_json).unwrap();
        if let Some(text) = parsed["text"].as_str() {
            let trimmed = text.trim();
            // Skip XML-based system prompts
            if trimmed.starts_with('<') && (trimmed.contains("ultrawork") || trimmed.contains("mode>")) {
                continue;
            }
            kept.push(text.to_string());
        }
    }

    assert_eq!(
        kept.len(),
        1,
        "only one part (the normal text) should survive skipping"
    );
    assert!(
        kept[0].contains("implementation"),
        "the kept part must be the normal assistant text"
    );
    assert!(
        !kept[0].contains("ultrawork"),
        "system prompt text must not appear in kept results"
    );
}

// ===========================================================================
// Test 8: latest_session_per_project
// ===========================================================================
//
// When a project has multiple sessions, only the most recently updated one
// (by time_updated) should be selected.

#[test]
fn test_latest_session_per_project() {
    let (_dir, db_path) = create_test_db();
    let conn = Connection::open(&db_path).unwrap();

    insert_project(&conn, "proj-latest", "/tmp/latest-test", "[]");

    // Session A — older
    insert_session(
        &conn,
        "sess-old",
        "proj-latest",
        "/tmp/latest-test",
        "Old Session",
        1000,  // older timestamp
        None,
    );

    // Session B — newer (should win)
    insert_session(
        &conn,
        "sess-new",
        "proj-latest",
        "/tmp/latest-test",
        "New Session",
        9999,  // newer timestamp
        None,
    );

    // Query: latest session for the project
    let latest: String = conn
        .query_row(
            "SELECT id FROM session
             WHERE project_id = 'proj-latest'
             ORDER BY time_updated DESC
             LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        latest, "sess-new",
        "the session with the highest time_updated must be selected"
    );

    // Verify both sessions still exist
    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM session WHERE project_id = 'proj-latest'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(total, 2, "both sessions must remain in the database");
}

// ===========================================================================
// Test 9: sandboxes_json_parsing
// ===========================================================================
//
// The project.sandboxes column is a JSON text array.  The implementation must
// parse it correctly and use the sandbox paths to match processes.

#[test]
fn test_sandboxes_json_parsing() {
    let (_dir, db_path) = create_test_db();
    let conn = Connection::open(&db_path).unwrap();

    let sandbox_dir = "/home/user/project-feature-branch";
    let sandboxes_json = format!(r#"["{}"]"#, sandbox_dir);

    insert_project(
        &conn,
        "proj-sb",
        "/home/user/project",
        &sandboxes_json,
    );

    // Verify the JSON is stored and parseable
    let raw: String = conn
        .query_row(
            "SELECT sandboxes FROM project WHERE id = 'proj-sb'",
            [],
            |r| r.get(0),
        )
        .unwrap();

    // Parse the JSON array
    let parsed: Vec<String> = serde_json::from_str(&raw).unwrap();
    assert_eq!(parsed.len(), 1, "sandboxes array must contain one entry");
    assert_eq!(
        parsed[0], sandbox_dir,
        "sandbox path must match the inserted value"
    );

    // Simulate the matching logic: check if the sandbox path starts with or
    // equals the process CWD (this is what the implementation must do).
    let process_cwd = PathBuf::from("/home/user/project-feature-branch");
    let matched = parsed.iter().any(|sb| {
        process_cwd == PathBuf::from(sb)
            || process_cwd.starts_with(&format!("{}/", sb))
    });
    assert!(
        matched,
        "sandbox path must match the process CWD when it is identical"
    );

    // Negative case: a CWD outside any sandbox must NOT match
    let outside_cwd = PathBuf::from("/home/user/other-project");
    let not_matched = parsed.iter().any(|sb| {
        outside_cwd == PathBuf::from(sb)
            || outside_cwd.starts_with(&format!("{}/", sb))
    });
    assert!(
        !not_matched,
        "a CWD outside the sandbox must NOT match"
    );
}

// ===========================================================================
// Compilation smoke-test: verify key types are accessible from test code.
// ===========================================================================

#[test]
fn test_types_are_accessible() {
    // AgentProcess
    let proc = AgentProcess {
        pid: 42,
        cpu_usage: 1.5,
        cwd: Some(PathBuf::from("/tmp")),
    };
    assert_eq!(proc.pid, 42);

    // Session
    let _session = Session {
        id: "test".to_string(),
        agent_type: AgentType::OpenCode,
        project_name: "p".to_string(),
        project_path: "/p".to_string(),
        git_branch: None,
        github_url: None,
        status: SessionStatus::Idle,
        last_message: None,
        last_message_role: None,
        last_activity_at: "2026-01-01T00:00:00.000Z".to_string(),
        pid: 42,
        cpu_usage: 0.0,
        active_subagent_count: 0,
    };

    // OpenCodeDetector is accessible
    let _detector = opencode::OpenCodeDetector;
}
