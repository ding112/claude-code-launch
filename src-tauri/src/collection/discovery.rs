use super::*;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub(super) struct DiscoveredSession {
    pub session_id: String,
    pub project_path: String,
    pub start_time_ms: i64,
    pub duration_minutes: u32,
    pub first_prompt: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub goal: String,
    pub outcome: String,
    pub summary: String,
    pub transcript_path: String,
}

#[derive(Debug, Serialize)]
pub(super) struct DiscoverResult {
    pub accepted: bool,
    pub scanned: usize,
    pub imported: usize,
    pub updated: usize,
    pub errors: usize,
}

fn claude_home() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude"))
}

fn session_meta_dir() -> Option<PathBuf> {
    claude_home().map(|h| h.join("usage-data").join("session-meta"))
}

fn facets_dir() -> Option<PathBuf> {
    claude_home().map(|h| h.join("usage-data").join("facets"))
}

fn projects_dir() -> Option<PathBuf> {
    claude_home().map(|h| h.join("projects"))
}

fn encode_project_path(project_path: &str) -> String {
    project_path.replace('/', "-")
}

pub(super) fn scan_session_meta() -> Vec<DiscoveredSession> {
    let Some(meta_dir) = session_meta_dir() else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(&meta_dir) else {
        return Vec::new();
    };

    let facets_base = facets_dir();
    let projects_base = projects_dir();
    let mut sessions = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let meta: Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let session_id = match meta.get("session_id").and_then(Value::as_str) {
            Some(id) if !id.trim().is_empty() => id.to_string(),
            _ => continue,
        };

        let project_path = meta
            .get("project_path")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();

        let start_time_ms = parse_start_time(&meta);

        let duration_minutes = meta
            .get("duration_minutes")
            .and_then(Value::as_u64)
            .unwrap_or(0) as u32;

        let first_prompt = meta
            .get("first_prompt")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();

        let input_tokens = meta
            .get("input_tokens")
            .and_then(Value::as_i64)
            .unwrap_or(0);

        let output_tokens = meta
            .get("output_tokens")
            .and_then(Value::as_i64)
            .unwrap_or(0);

        let (goal, outcome, summary) = facets_base
            .as_ref()
            .map(|dir| enrich_with_facets(dir, &session_id))
            .unwrap_or_default();

        let transcript_path = projects_base
            .as_ref()
            .map(|dir| resolve_transcript_path(dir, &project_path, &session_id))
            .unwrap_or_default();

        sessions.push(DiscoveredSession {
            session_id,
            project_path,
            start_time_ms,
            duration_minutes,
            first_prompt,
            input_tokens,
            output_tokens,
            goal,
            outcome,
            summary,
            transcript_path,
        });
    }

    sessions
}

fn parse_start_time(meta: &Value) -> i64 {
    if let Some(ms) = meta.get("start_time_ms").and_then(Value::as_i64) {
        return ms;
    }
    if let Some(iso) = meta.get("start_time").and_then(Value::as_str) {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(iso) {
            return dt.timestamp_millis();
        }
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(iso, "%Y-%m-%dT%H:%M:%S%.fZ") {
            return dt.and_utc().timestamp_millis();
        }
    }
    0
}

fn enrich_with_facets(facets_dir: &Path, session_id: &str) -> (String, String, String) {
    let facet_path = facets_dir.join(format!("{session_id}.json"));
    let content = match std::fs::read_to_string(&facet_path) {
        Ok(c) => c,
        Err(_) => return Default::default(),
    };
    let facet: Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return Default::default(),
    };

    let goal = facet
        .get("underlying_goal")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let outcome = facet
        .get("outcome")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let summary = facet
        .get("brief_summary")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    (goal, outcome, summary)
}

fn resolve_transcript_path(projects_dir: &Path, project_path: &str, session_id: &str) -> String {
    if project_path.is_empty() {
        return String::new();
    }
    let encoded = encode_project_path(project_path);
    let transcript = projects_dir
        .join(&encoded)
        .join(format!("{session_id}.jsonl"));
    if transcript.exists() {
        transcript.to_string_lossy().to_string()
    } else {
        String::new()
    }
}

fn extract_project_name(project_path: &str) -> String {
    if project_path.is_empty() {
        return "unknown".to_string();
    }
    project_path
        .rsplit('/')
        .find(|s| !s.is_empty())
        .unwrap_or(project_path)
        .to_string()
}

pub(super) fn import_discovered_sessions(
    db: &mut Connection,
    sessions: &[DiscoveredSession],
) -> DiscoverResult {
    let mut imported = 0usize;
    let mut updated = 0usize;
    let mut errors = 0usize;

    let existing_ids: HashSet<String> = {
        let mut stmt = match db.prepare("SELECT session_id FROM sessions") {
            Ok(s) => s,
            Err(error) => {
                eprintln!("level=error event=discovery stage=load_existing error={error:?}");
                return DiscoverResult {
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
            eprintln!("level=error event=discovery stage=begin_tx error={error:?}");
            return DiscoverResult {
                accepted: true,
                scanned: sessions.len(),
                imported: 0,
                updated: 0,
                errors: sessions.len(),
            };
        }
    };

    for session in sessions {
        let project_name = extract_project_name(&session.project_path);
        let is_existing = existing_ids.contains(&session.session_id);

        let result = if is_existing {
            update_existing_session(&tx, session)
        } else {
            insert_new_session(&tx, session, &project_name)
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
                    "level=error event=discovery stage=persist session_id={} error={error:?}",
                    session.session_id
                );
                errors += 1;
            }
        }

        if !session.transcript_path.is_empty() {
            if let Err(error) = ensure_transcript_record(&tx, session) {
                eprintln!(
                    "level=error event=discovery stage=transcript_record session_id={} error={error:?}",
                    session.session_id
                );
            }
        }
    }

    if let Err(error) = tx.commit() {
        eprintln!("level=error event=discovery stage=commit error={error:?}");
        errors = sessions.len();
        imported = 0;
        updated = 0;
    }

    DiscoverResult {
        accepted: true,
        scanned: sessions.len(),
        imported,
        updated,
        errors,
    }
}

fn insert_new_session(
    tx: &Connection,
    session: &DiscoveredSession,
    project_name: &str,
) -> rusqlite::Result<bool> {
    let end_ms = if session.duration_minutes > 0 {
        session.start_time_ms + (session.duration_minutes as i64) * 60_000
    } else {
        session.start_time_ms
    };

    let changed = tx.execute(
        "INSERT INTO sessions(
            session_id, project_name, agent_type,
            first_seen_at_ms, last_active_at_ms,
            first_prompt, duration_minutes, input_tokens, output_tokens,
            goal, summary, outcome, source
        ) VALUES (?1, ?2, 'claude-code', ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 'discovery')
        ON CONFLICT(session_id) DO NOTHING",
        params![
            session.session_id,
            project_name,
            session.start_time_ms,
            end_ms,
            session.first_prompt,
            session.duration_minutes,
            session.input_tokens,
            session.output_tokens,
            session.goal,
            session.summary,
            session.outcome,
        ],
    )?;
    Ok(changed > 0)
}

fn update_existing_session(
    tx: &Connection,
    session: &DiscoveredSession,
) -> rusqlite::Result<bool> {
    let changed = tx.execute(
        "UPDATE sessions SET
            first_prompt = CASE WHEN first_prompt = '' THEN ?2 ELSE first_prompt END,
            duration_minutes = CASE WHEN duration_minutes = 0 THEN ?3 ELSE duration_minutes END,
            input_tokens = CASE WHEN input_tokens = 0 THEN ?4 ELSE input_tokens END,
            output_tokens = CASE WHEN output_tokens = 0 THEN ?5 ELSE output_tokens END,
            goal = CASE WHEN goal = '' THEN ?6 ELSE goal END,
            summary = CASE WHEN summary = '' THEN ?7 ELSE summary END,
            outcome = CASE WHEN outcome = '' THEN ?8 ELSE outcome END
        WHERE session_id = ?1",
        params![
            session.session_id,
            session.first_prompt,
            session.duration_minutes,
            session.input_tokens,
            session.output_tokens,
            session.goal,
            session.summary,
            session.outcome,
        ],
    )?;
    Ok(changed > 0)
}

fn ensure_transcript_record(
    tx: &Connection,
    session: &DiscoveredSession,
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
