use super::*;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

#[derive(Debug, Clone, Serialize)]
pub(super) struct CursorDiscoveredSession {
    pub session_id: String,
    pub project_name: String,
    pub encoded_project_dir: String,
    pub start_time_ms: i64,
    pub last_active_at_ms: i64,
    pub first_prompt: String,
    pub transcript_path: String,
    pub line_count: usize,
}

#[derive(Debug, Serialize)]
pub(super) struct CursorDiscoverResult {
    pub accepted: bool,
    pub scanned: usize,
    pub imported: usize,
    pub updated: usize,
    pub errors: usize,
}

fn cursor_home() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".cursor"))
}

fn cursor_projects_dir() -> Option<PathBuf> {
    cursor_home().map(|h| h.join("projects"))
}

fn extract_project_name(encoded_dir: &str) -> String {
    if encoded_dir.is_empty() {
        return "unknown".to_string();
    }
    encoded_dir
        .rsplit('-')
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or(encoded_dir)
        .to_string()
}

fn extract_first_prompt(transcript_path: &Path) -> String {
    let file = match std::fs::File::open(transcript_path) {
        Ok(f) => f,
        Err(_) => return String::new(),
    };
    let reader = std::io::BufReader::new(file);
    use std::io::BufRead;

    for line in reader.lines().take(20) {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        let parsed: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if parsed.get("role").and_then(Value::as_str) != Some("user") {
            continue;
        }
        let content = parsed
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(Value::as_array);
        let Some(blocks) = content else { continue };

        for block in blocks {
            if block.get("type").and_then(Value::as_str) != Some("text") {
                continue;
            }
            let Some(text) = block.get("text").and_then(Value::as_str) else {
                continue;
            };
            if let Some(start) = text.find("<user_query>") {
                let after_tag = &text[start + "<user_query>".len()..];
                if let Some(end) = after_tag.find("</user_query>") {
                    let prompt = after_tag[..end].trim();
                    if !prompt.is_empty() {
                        return truncate_prompt(prompt);
                    }
                }
            }
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                return truncate_prompt(trimmed);
            }
        }
    }
    String::new()
}

fn truncate_prompt(s: &str) -> String {
    const MAX_LEN: usize = 500;
    if s.len() <= MAX_LEN {
        s.to_string()
    } else {
        let boundary = s.char_indices()
            .take_while(|(i, _)| *i < MAX_LEN)
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(MAX_LEN);
        format!("{}…", &s[..boundary])
    }
}

fn count_lines(path: &Path) -> usize {
    use std::io::BufRead;
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return 0,
    };
    std::io::BufReader::new(file).lines().count()
}

fn file_time_ms(metadata: &std::fs::Metadata, use_created: bool) -> i64 {
    let time = if use_created {
        metadata.created().ok()
    } else {
        metadata.modified().ok()
    };
    time.and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

pub(super) fn scan_cursor_sessions() -> Vec<CursorDiscoveredSession> {
    let Some(projects_dir) = cursor_projects_dir() else {
        return Vec::new();
    };
    let Ok(project_entries) = std::fs::read_dir(&projects_dir) else {
        return Vec::new();
    };

    let mut sessions = Vec::new();

    for project_entry in project_entries.flatten() {
        let project_path = project_entry.path();
        if !project_path.is_dir() {
            continue;
        }

        let encoded_dir = match project_path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        let transcripts_dir = project_path.join("agent-transcripts");
        if !transcripts_dir.is_dir() {
            continue;
        }

        let Ok(session_entries) = std::fs::read_dir(&transcripts_dir) else {
            continue;
        };

        let project_name = extract_project_name(&encoded_dir);

        for session_entry in session_entries.flatten() {
            let session_dir = session_entry.path();
            if !session_dir.is_dir() {
                continue;
            }

            let session_id = match session_dir.file_name().and_then(|n| n.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };

            let jsonl_path = session_dir.join(format!("{session_id}.jsonl"));
            if !jsonl_path.exists() {
                continue;
            }

            let metadata = match std::fs::metadata(&jsonl_path) {
                Ok(m) => m,
                Err(_) => continue,
            };

            let start_time_ms = file_time_ms(&metadata, true);
            let last_active_at_ms = file_time_ms(&metadata, false);
            let first_prompt = extract_first_prompt(&jsonl_path);
            let line_count = count_lines(&jsonl_path);
            let transcript_path = jsonl_path.to_string_lossy().to_string();

            sessions.push(CursorDiscoveredSession {
                session_id,
                project_name: project_name.clone(),
                encoded_project_dir: encoded_dir.clone(),
                start_time_ms,
                last_active_at_ms,
                first_prompt,
                transcript_path,
                line_count,
            });
        }
    }

    sessions
}

pub(super) fn import_cursor_sessions(
    db: &mut Connection,
    sessions: &[CursorDiscoveredSession],
) -> CursorDiscoverResult {
    let mut imported = 0usize;
    let mut updated = 0usize;
    let mut errors = 0usize;

    let existing_ids: HashSet<String> = {
        let mut stmt = match db.prepare("SELECT session_id FROM sessions") {
            Ok(s) => s,
            Err(error) => {
                eprintln!("level=error event=cursor_discovery stage=load_existing error={error:?}");
                return CursorDiscoverResult {
                    accepted: true,
                    scanned: sessions.len(),
                    imported: 0,
                    updated: 0,
                    errors: sessions.len(),
                };
            }
        };
        let rows = stmt.query_map([], |row| row.get::<_, String>(0));
        match rows {
            Ok(rows) => rows.flatten().collect(),
            Err(_) => HashSet::new(),
        }
    };

    let tx = match db.transaction() {
        Ok(tx) => tx,
        Err(error) => {
            eprintln!("level=error event=cursor_discovery stage=begin_tx error={error:?}");
            return CursorDiscoverResult {
                accepted: true,
                scanned: sessions.len(),
                imported: 0,
                updated: 0,
                errors: sessions.len(),
            };
        }
    };

    for session in sessions {
        let is_existing = existing_ids.contains(&session.session_id);

        let result = if is_existing {
            update_existing_cursor_session(&tx, session)
        } else {
            insert_new_cursor_session(&tx, session)
        };

        match result {
            Ok(true) => {
                if is_existing {
                    updated += 1;
                } else {
                    imported += 1;
                }
            }
            Ok(false) => {}
            Err(error) => {
                eprintln!(
                    "level=error event=cursor_discovery stage=persist session_id={} error={error:?}",
                    session.session_id
                );
                errors += 1;
            }
        }

        if !session.transcript_path.is_empty() {
            if let Err(error) = ensure_cursor_transcript_record(&tx, session) {
                eprintln!(
                    "level=error event=cursor_discovery stage=transcript_record session_id={} error={error:?}",
                    session.session_id
                );
            }
        }
    }

    if let Err(error) = tx.commit() {
        eprintln!("level=error event=cursor_discovery stage=commit error={error:?}");
        errors = sessions.len();
        imported = 0;
        updated = 0;
    }

    CursorDiscoverResult {
        accepted: true,
        scanned: sessions.len(),
        imported,
        updated,
        errors,
    }
}

fn insert_new_cursor_session(
    tx: &Connection,
    session: &CursorDiscoveredSession,
) -> rusqlite::Result<bool> {
    let changed = tx.execute(
        "INSERT INTO sessions(
            session_id, project_name, agent_type,
            first_seen_at_ms, last_active_at_ms,
            first_prompt, duration_minutes, input_tokens, output_tokens,
            goal, summary, outcome, source
        ) VALUES (?1, ?2, 'cursor', ?3, ?4, ?5, 0, 0, 0, '', '', '', 'cursor-discovery')
        ON CONFLICT(session_id) DO NOTHING",
        params![
            session.session_id,
            session.project_name,
            session.start_time_ms,
            session.last_active_at_ms,
            session.first_prompt,
        ],
    )?;
    Ok(changed > 0)
}

fn update_existing_cursor_session(
    tx: &Connection,
    session: &CursorDiscoveredSession,
) -> rusqlite::Result<bool> {
    let changed = tx.execute(
        "UPDATE sessions SET
            first_prompt = CASE WHEN first_prompt = '' THEN ?2 ELSE first_prompt END,
            last_active_at_ms = MAX(last_active_at_ms, ?3)
        WHERE session_id = ?1",
        params![
            session.session_id,
            session.first_prompt,
            session.last_active_at_ms,
        ],
    )?;
    Ok(changed > 0)
}

fn ensure_cursor_transcript_record(
    tx: &Connection,
    session: &CursorDiscoveredSession,
) -> rusqlite::Result<()> {
    let updated_at = now_timestamp_ms();
    tx.execute(
        "INSERT INTO session_transcripts(
            session_id, transcript_path,
            imported_offset_bytes, file_mtime_ms, file_size_bytes,
            pending_fragment, updated_at_ms
        ) VALUES (?1, ?2, 0, 0, 0, '', ?3)
        ON CONFLICT(session_id) DO UPDATE SET
            transcript_path = CASE
                WHEN session_transcripts.transcript_path = '' THEN excluded.transcript_path
                ELSE session_transcripts.transcript_path
            END",
        params![session.session_id, session.transcript_path, updated_at],
    )?;
    Ok(())
}
