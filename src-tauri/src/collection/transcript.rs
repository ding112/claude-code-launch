use super::*;

#[derive(Debug, Clone)]
pub(super) struct TranscriptSyncState {
    pub(super) transcript_path: String,
    pub(super) imported_offset_bytes: i64,
    pub(super) file_mtime_ms: i64,
    pub(super) pending_fragment: String,
}

#[derive(Debug)]
pub(super) struct TranscriptReadResult {
    pub(super) lines: Vec<String>,
    pub(super) pending_fragment: String,
    pub(super) next_offset_bytes: i64,
    pub(super) file_mtime_ms: i64,
    pub(super) file_size_bytes: i64,
    pub(super) reset_content: bool,
}

pub(super) fn sync_transcript_after_event(
    db: &Arc<Mutex<Connection>>,
    event: &IncomingEvent,
) {
    let Some(transcript_path) = extract_transcript_path(&event.payload) else {
        return;
    };
    let session_id = event.session_id.clone();

    let existing_state = {
        let db_guard = match db.lock() {
            Ok(guard) => guard,
            Err(error) => {
                eprintln!(
                    "level=error event=transcript_sync stage=lock_sqlite_lookup session_id={session_id} error={error:?}"
                );
                return;
            }
        };
        load_transcript_sync_state(&db_guard, &session_id)
    };

    let existing_state = match existing_state {
        Ok(state) => state,
        Err(error) => {
            record_transcript_error_with_db(
                db, &session_id, &transcript_path,
                format!("failed to load transcript sync state: {error:?}"),
                format!("{error:?}"),
            );
            return;
        }
    };

    let read_result = match read_transcript_increment(&transcript_path, existing_state.as_ref()) {
        Ok(result) => result,
        Err(error) => {
            record_transcript_error_with_db(
                db, &session_id, &transcript_path,
                error.to_string(),
                format!("{error:?}"),
            );
            return;
        }
    };

    let mut db_guard = match db.lock() {
        Ok(guard) => guard,
        Err(error) => {
            eprintln!(
                "level=error event=transcript_sync stage=lock_sqlite_upsert session_id={session_id} error={error:?}"
            );
            return;
        }
    };
    if let Err(error) =
        upsert_transcript_sync_state(&mut db_guard, &session_id, &transcript_path, &read_result)
    {
        eprintln!(
            "level=error event=transcript_sync stage=upsert session_id={session_id} error={error:?}"
        );
    }
}

fn record_transcript_error_with_db(
    db: &Arc<Mutex<Connection>>,
    session_id: &str,
    transcript_path: &str,
    error_message: String,
    error_stack: String,
) {
    let db_guard = match db.lock() {
        Ok(guard) => guard,
        Err(error) => {
            eprintln!(
                "level=error event=transcript_sync_error_record_failed stage=lock_sqlite session_id={session_id} error={error:?}"
            );
            return;
        }
    };
    if let Err(error) = db_guard.execute(
        "
        INSERT INTO session_transcripts(
            session_id,
            transcript_path,
            imported_offset_bytes,
            file_mtime_ms,
            file_size_bytes,
            pending_fragment,
            updated_at_ms,
            last_error_message,
            last_error_stack
        ) VALUES (?1, ?2, 0, 0, 0, '', ?3, ?4, ?5)
        ON CONFLICT(session_id) DO UPDATE SET
            transcript_path = excluded.transcript_path,
            updated_at_ms = excluded.updated_at_ms,
            last_error_message = excluded.last_error_message,
            last_error_stack = excluded.last_error_stack
        ",
        params![
            session_id,
            transcript_path,
            now_timestamp_ms(),
            error_message,
            error_stack
        ],
    ) {
        eprintln!(
            "level=error event=transcript_sync_error_record_failed stage=upsert_error_state session_id={session_id} error={error:?}"
        );
    }
}

fn extract_transcript_path(payload: &Value) -> Option<String> {
    let raw = payload
        .get("stdin_json")
        .and_then(|stdin_json| stdin_json.get("transcript_path"))
        .and_then(Value::as_str)
        .filter(|path| !path.trim().is_empty())?;

    validate_transcript_path(raw)
}

pub(super) fn validate_transcript_path(raw: &str) -> Option<String> {
    let path = std::path::Path::new(raw);

    if path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        eprintln!("level=warn event=transcript_path_rejected reason=parent_dir_component path={raw}");
        return None;
    }

    if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
        eprintln!("level=warn event=transcript_path_rejected reason=invalid_extension path={raw}");
        return None;
    }

    let home = dirs::home_dir().map(|h| h.canonicalize().unwrap_or(h));

    if let Ok(canonical) = path.canonicalize() {
        if let Some(home) = &home {
            if !canonical.starts_with(home) {
                eprintln!("level=warn event=transcript_path_rejected reason=outside_home_dir path={raw}");
                return None;
            }
        }
        return Some(canonical.to_string_lossy().to_string());
    }

    if let Some(home) = &home {
        if !path.starts_with(home) {
            eprintln!("level=warn event=transcript_path_rejected reason=outside_home_dir_unresolved path={raw}");
            return None;
        }
    }

    Some(raw.to_string())
}

pub(super) fn load_transcript_sync_state(
    db: &Connection,
    session_id: &str,
) -> rusqlite::Result<Option<TranscriptSyncState>> {
    let mut statement = db.prepare(
        "
        SELECT transcript_path, imported_offset_bytes, file_mtime_ms, pending_fragment
        FROM session_transcripts
        WHERE session_id = ?1
        ",
    )?;
    let mut rows = statement.query(params![session_id])?;
    if let Some(row) = rows.next()? {
        return Ok(Some(TranscriptSyncState {
            transcript_path: row.get(0)?,
            imported_offset_bytes: row.get::<_, i64>(1)?.max(0),
            file_mtime_ms: row.get(2)?,
            pending_fragment: row.get(3)?,
        }));
    }
    Ok(None)
}

pub(super) fn read_transcript_increment(
    transcript_path: &str,
    existing_state: Option<&TranscriptSyncState>,
) -> std::io::Result<TranscriptReadResult> {
    if validate_transcript_path(transcript_path).is_none() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            format!("transcript path rejected by validation: {transcript_path}"),
        ));
    }
    let mut file = File::open(transcript_path)?;
    let metadata = file.metadata()?;
    let file_size_bytes = metadata.len() as i64;
    let file_mtime_ms = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default();

    let mut offset = existing_state
        .map(|state| state.imported_offset_bytes.max(0))
        .unwrap_or(0);
    let mut pending_fragment = existing_state
        .map(|state| state.pending_fragment.clone())
        .unwrap_or_default();
    let mut reset_content = false;

    if let Some(state) = existing_state {
        if state.transcript_path != transcript_path
            || file_size_bytes < state.imported_offset_bytes
            || (state.file_mtime_ms > 0 && file_mtime_ms > 0 && file_mtime_ms < state.file_mtime_ms)
        {
            offset = 0;
            pending_fragment.clear();
            reset_content = true;
        }
    }

    if offset > file_size_bytes {
        offset = 0;
        pending_fragment.clear();
        reset_content = true;
    }

    file.seek(SeekFrom::Start(offset as u64))?;
    let mut chunk_bytes = Vec::new();
    file.take(super::TRANSCRIPT_SYNC_MAX_BYTES as u64)
        .read_to_end(&mut chunk_bytes)?;
    let chunk = String::from_utf8_lossy(&chunk_bytes).to_string();
    let (lines, next_pending_fragment) = split_transcript_lines(&pending_fragment, &chunk);
    let next_offset_bytes = offset + chunk_bytes.len() as i64;

    Ok(TranscriptReadResult {
        lines,
        pending_fragment: next_pending_fragment,
        next_offset_bytes,
        file_mtime_ms,
        file_size_bytes,
        reset_content,
    })
}

fn split_transcript_lines(pending_fragment: &str, chunk: &str) -> (Vec<String>, String) {
    let combined = format!("{pending_fragment}{chunk}");
    if combined.is_empty() {
        return (Vec::new(), String::new());
    }

    let mut lines = Vec::new();
    let mut start = 0usize;
    for (index, ch) in combined.char_indices() {
        if ch == '\n' {
            let line_end = if index > start && combined.as_bytes().get(index - 1) == Some(&b'\r') {
                index - 1
            } else {
                index
            };
            let mut line = combined[start..line_end].to_string();
            line.push('\n');
            lines.push(line);
            start = index + 1;
        }
    }

    let pending = combined[start..].to_string();
    (lines, pending)
}

pub(super) fn upsert_transcript_sync_state(
    db: &mut Connection,
    session_id: &str,
    transcript_path: &str,
    result: &TranscriptReadResult,
) -> rusqlite::Result<()> {
    let now = now_timestamp_ms();
    let tx = db.transaction()?;

    if result.reset_content {
        tx.execute(
            "DELETE FROM session_transcript_lines WHERE session_id = ?1",
            params![session_id],
        )?;
    }

    let mut next_line_no = tx.query_row(
        "SELECT COALESCE(MAX(line_no), 0) + 1 FROM session_transcript_lines WHERE session_id = ?1",
        params![session_id],
        |row| row.get::<_, i64>(0),
    )?;

    if !result.lines.is_empty() {
        let mut insert_statement = tx.prepare(
            "
            INSERT INTO session_transcript_lines(session_id, line_no, line_content, created_at_ms)
            VALUES (?1, ?2, ?3, ?4)
            ",
        )?;
        for line in &result.lines {
            insert_statement.execute(params![session_id, next_line_no, line, now])?;
            next_line_no += 1;
        }
    }

    tx.execute(
        "
        INSERT INTO session_transcripts(
            session_id,
            transcript_path,
            imported_offset_bytes,
            file_mtime_ms,
            file_size_bytes,
            pending_fragment,
            updated_at_ms,
            last_error_message,
            last_error_stack
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, NULL)
        ON CONFLICT(session_id) DO UPDATE SET
            transcript_path = excluded.transcript_path,
            imported_offset_bytes = excluded.imported_offset_bytes,
            file_mtime_ms = excluded.file_mtime_ms,
            file_size_bytes = excluded.file_size_bytes,
            pending_fragment = excluded.pending_fragment,
            updated_at_ms = excluded.updated_at_ms,
            last_error_message = NULL,
            last_error_stack = NULL
        ",
        params![
            session_id,
            transcript_path,
            result.next_offset_bytes,
            result.file_mtime_ms,
            result.file_size_bytes,
            &result.pending_fragment,
            now,
        ],
    )?;
    tx.commit()?;
    Ok(())
}

pub(super) fn record_transcript_error(
    state: &AppState,
    session_id: &str,
    transcript_path: &str,
    error_message: String,
    error_stack: String,
) {
    let db = match state.db.lock() {
        Ok(db) => db,
        Err(error) => {
            eprintln!(
                "level=error event=transcript_sync_error_record_failed stage=lock_sqlite session_id={} transcript_path={} error={:?}",
                session_id, transcript_path, error
            );
            return;
        }
    };
    if let Err(error) = db.execute(
        "
        INSERT INTO session_transcripts(
            session_id,
            transcript_path,
            imported_offset_bytes,
            file_mtime_ms,
            file_size_bytes,
            pending_fragment,
            updated_at_ms,
            last_error_message,
            last_error_stack
        ) VALUES (?1, ?2, 0, 0, 0, '', ?3, ?4, ?5)
        ON CONFLICT(session_id) DO UPDATE SET
            transcript_path = excluded.transcript_path,
            updated_at_ms = excluded.updated_at_ms,
            last_error_message = excluded.last_error_message,
            last_error_stack = excluded.last_error_stack
        ",
        params![
            session_id,
            transcript_path,
            now_timestamp_ms(),
            error_message,
            error_stack
        ],
    ) {
        eprintln!(
            "level=error event=transcript_sync_error_record_failed stage=upsert_error_state session_id={} transcript_path={} error={:?}",
            session_id, transcript_path, error
        );
    }
}
