use super::*;

pub(super) fn init_schema(db: &Connection) -> rusqlite::Result<()> {
    db.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        CREATE TABLE IF NOT EXISTS sessions (
            session_id TEXT PRIMARY KEY,
            project_name TEXT NOT NULL,
            first_seen_at_ms INTEGER NOT NULL,
            last_active_at_ms INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS events (
            event_id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            event_type TEXT NOT NULL,
            payload TEXT NOT NULL,
            created_at_ms INTEGER NOT NULL,
            is_archived INTEGER NOT NULL DEFAULT 0,
            received_at_ms INTEGER NOT NULL,
            FOREIGN KEY(session_id) REFERENCES sessions(session_id)
        );

        CREATE INDEX IF NOT EXISTS idx_events_session_time
            ON events(session_id, created_at_ms DESC);

        CREATE TABLE IF NOT EXISTS evaluations (
            evaluation_id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            event_id TEXT,
            provider TEXT NOT NULL,
            model TEXT NOT NULL,
            base_url TEXT NOT NULL DEFAULT '',
            risk_level TEXT NOT NULL,
            risk_category TEXT NOT NULL DEFAULT 'normal',
            efficiency_level TEXT NOT NULL,
            suggestion TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'success',
            error_message TEXT,
            error_stack TEXT,
            input_summary TEXT NOT NULL DEFAULT '{}',
            retry_count INTEGER NOT NULL DEFAULT 0,
            created_at_ms INTEGER NOT NULL,
            updated_at_ms INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_evaluations_session_time
            ON evaluations(session_id, created_at_ms DESC);

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at_ms INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS session_transcripts (
            session_id TEXT PRIMARY KEY,
            transcript_path TEXT NOT NULL DEFAULT '',
            imported_offset_bytes INTEGER NOT NULL DEFAULT 0,
            file_mtime_ms INTEGER NOT NULL DEFAULT 0,
            file_size_bytes INTEGER NOT NULL DEFAULT 0,
            pending_fragment TEXT NOT NULL DEFAULT '',
            updated_at_ms INTEGER NOT NULL,
            last_error_message TEXT,
            last_error_stack TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_session_transcripts_updated
            ON session_transcripts(updated_at_ms DESC);

        CREATE TABLE IF NOT EXISTS session_transcript_lines (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id TEXT NOT NULL,
            line_no INTEGER NOT NULL,
            line_content TEXT NOT NULL,
            created_at_ms INTEGER NOT NULL,
            UNIQUE(session_id, line_no)
        );
        CREATE INDEX IF NOT EXISTS idx_session_transcript_lines_session
            ON session_transcript_lines(session_id, line_no);
        ",
    )?;

    ensure_session_columns(db)?;
    ensure_event_columns(db)?;
    ensure_evaluation_columns(db)?;
    ensure_session_transcript_columns(db)?;
    Ok(())
}

fn ensure_session_columns(db: &Connection) -> rusqlite::Result<()> {
    let mut columns = Vec::new();
    let mut statement = db.prepare("PRAGMA table_info(sessions)")?;
    let rows = statement.query_map([], |row| row.get::<_, String>(1))?;
    for row in rows {
        columns.push(row?);
    }

    add_column_if_missing(
        db,
        "sessions",
        &columns,
        "agent_type",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    Ok(())
}

fn ensure_event_columns(db: &Connection) -> rusqlite::Result<()> {
    let mut columns = Vec::new();
    let mut statement = db.prepare("PRAGMA table_info(events)")?;
    let rows = statement.query_map([], |row| row.get::<_, String>(1))?;
    for row in rows {
        columns.push(row?);
    }

    add_column_if_missing(
        db,
        "events",
        &columns,
        "is_archived",
        "INTEGER NOT NULL DEFAULT 0",
    )?;
    Ok(())
}

fn ensure_evaluation_columns(db: &Connection) -> rusqlite::Result<()> {
    let mut columns = Vec::new();
    let mut statement = db.prepare("PRAGMA table_info(evaluations)")?;
    let rows = statement.query_map([], |row| row.get::<_, String>(1))?;
    for row in rows {
        columns.push(row?);
    }

    add_column_if_missing(
        db,
        "evaluations",
        &columns,
        "risk_category",
        "TEXT NOT NULL DEFAULT 'normal'",
    )?;
    add_column_if_missing(
        db,
        "evaluations",
        &columns,
        "status",
        "TEXT NOT NULL DEFAULT 'success'",
    )?;
    add_column_if_missing(
        db,
        "evaluations",
        &columns,
        "base_url",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    add_column_if_missing(db, "evaluations", &columns, "error_message", "TEXT")?;
    add_column_if_missing(db, "evaluations", &columns, "error_stack", "TEXT")?;
    add_column_if_missing(
        db,
        "evaluations",
        &columns,
        "input_summary",
        "TEXT NOT NULL DEFAULT '{}'",
    )?;
    add_column_if_missing(
        db,
        "evaluations",
        &columns,
        "retry_count",
        "INTEGER NOT NULL DEFAULT 0",
    )?;
    add_column_if_missing(
        db,
        "evaluations",
        &columns,
        "updated_at_ms",
        "INTEGER NOT NULL DEFAULT 0",
    )?;
    Ok(())
}

fn ensure_session_transcript_columns(db: &Connection) -> rusqlite::Result<()> {
    let mut columns = Vec::new();
    let mut statement = db.prepare("PRAGMA table_info(session_transcripts)")?;
    let rows = statement.query_map([], |row| row.get::<_, String>(1))?;
    for row in rows {
        columns.push(row?);
    }

    add_column_if_missing(
        db,
        "session_transcripts",
        &columns,
        "transcript_path",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    add_column_if_missing(
        db,
        "session_transcripts",
        &columns,
        "imported_offset_bytes",
        "INTEGER NOT NULL DEFAULT 0",
    )?;
    add_column_if_missing(
        db,
        "session_transcripts",
        &columns,
        "file_mtime_ms",
        "INTEGER NOT NULL DEFAULT 0",
    )?;
    add_column_if_missing(
        db,
        "session_transcripts",
        &columns,
        "file_size_bytes",
        "INTEGER NOT NULL DEFAULT 0",
    )?;
    add_column_if_missing(
        db,
        "session_transcripts",
        &columns,
        "pending_fragment",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    add_column_if_missing(
        db,
        "session_transcripts",
        &columns,
        "updated_at_ms",
        "INTEGER NOT NULL DEFAULT 0",
    )?;
    add_column_if_missing(
        db,
        "session_transcripts",
        &columns,
        "last_error_message",
        "TEXT",
    )?;
    add_column_if_missing(
        db,
        "session_transcripts",
        &columns,
        "last_error_stack",
        "TEXT",
    )?;
    Ok(())
}

fn add_column_if_missing(
    db: &Connection,
    table_name: &str,
    columns: &[String],
    column: &str,
    schema: &str,
) -> rusqlite::Result<()> {
    if columns.iter().any(|existing| existing == column) {
        return Ok(());
    }
    db.execute(
        &format!("ALTER TABLE {table_name} ADD COLUMN {column} {schema}"),
        [],
    )?;
    Ok(())
}

fn detect_agent_type(payload: &Value) -> String {
    let cursor_version = payload
        .get("stdin_json")
        .and_then(|v| v.get("cursor_version"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if !cursor_version.is_empty() {
        return "cursor".to_string();
    }
    "claude-code".to_string()
}

pub(super) fn persist_event(db: &Connection, event: &IncomingEvent) -> rusqlite::Result<bool> {
    let now_ms = now_timestamp_ms();
    let payload_json = serde_json::to_string(&event.payload).unwrap_or_else(|_| "{}".to_string());
    let agent_type = detect_agent_type(&event.payload);

    let mut statement = db.prepare(
        "
        INSERT INTO sessions(session_id, project_name, agent_type, first_seen_at_ms, last_active_at_ms)
        VALUES (?1, ?2, ?3, ?4, ?4)
        ON CONFLICT(session_id) DO UPDATE SET
            project_name = excluded.project_name,
            last_active_at_ms = MAX(last_active_at_ms, excluded.last_active_at_ms)
        ",
    )?;

    statement.execute(params![
        event.session_id,
        event.project_name,
        agent_type,
        event.created_at_ms
    ])?;

    let inserted = db.execute(
        "
        INSERT INTO events(event_id, session_id, event_type, payload, created_at_ms, is_archived, received_at_ms)
        VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6)
        ON CONFLICT(event_id) DO NOTHING
        ",
        params![
            event.event_id,
            event.session_id,
            event.event_type,
            payload_json,
            event.created_at_ms,
            now_ms
        ],
    )?;

    Ok(inserted > 0)
}

pub(super) fn load_eval_config(db: &Connection) -> rusqlite::Result<EvalConfig> {
    let mut config = EvalConfig::default();
    let mut statement = db.prepare("SELECT key, value FROM settings")?;
    let mut rows = statement.query([])?;
    while let Some(row) = rows.next()? {
        let key: String = row.get(0)?;
        let value: String = row.get(1)?;
        match key.as_str() {
            "eval_enabled" => config.enabled = value == "true",
            "eval_sampling_rate" => config.sampling_rate = value.parse::<u32>().unwrap_or(1).max(1),
            "eval_provider" => config.provider = value,
            "eval_model" => config.model = value,
            "eval_base_url" => config.base_url = value,
            "eval_api_key" => {
                if value.trim().is_empty() {
                    config.api_key = None;
                } else {
                    config.api_key = Some(value);
                }
            }
            "eval_timeout_ms" => {
                config.timeout_ms = value.parse::<u64>().unwrap_or(8_000).clamp(500, 120_000)
            }
            _ => {}
        }
    }
    Ok(config)
}

pub(super) fn persist_eval_config(db: &Connection, config: &EvalConfig) -> rusqlite::Result<()> {
    let updated_at_ms = now_timestamp_ms();
    persist_setting(
        db,
        "eval_enabled",
        if config.enabled { "true" } else { "false" },
        updated_at_ms,
    )?;
    persist_setting(
        db,
        "eval_sampling_rate",
        &config.sampling_rate.max(1).to_string(),
        updated_at_ms,
    )?;
    persist_setting(db, "eval_provider", &config.provider, updated_at_ms)?;
    persist_setting(db, "eval_model", &config.model, updated_at_ms)?;
    persist_setting(db, "eval_base_url", &config.base_url, updated_at_ms)?;
    persist_setting(
        db,
        "eval_api_key",
        config.api_key.as_deref().unwrap_or(""),
        updated_at_ms,
    )?;
    persist_setting(
        db,
        "eval_timeout_ms",
        &config.timeout_ms.clamp(500, 120_000).to_string(),
        updated_at_ms,
    )?;
    Ok(())
}

fn persist_setting(
    db: &Connection,
    key: &str,
    value: &str,
    updated_at_ms: i64,
) -> rusqlite::Result<()> {
    db.execute(
        "
        INSERT INTO settings(key, value, updated_at_ms)
        VALUES (?1, ?2, ?3)
        ON CONFLICT(key) DO UPDATE SET
            value = excluded.value,
            updated_at_ms = excluded.updated_at_ms
        ",
        params![key, value, updated_at_ms],
    )?;
    Ok(())
}
