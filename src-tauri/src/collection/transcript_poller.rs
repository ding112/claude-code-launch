use super::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Spawn the linemux-based transcript watcher.
///
/// On startup, loads all known transcript paths from SQLite and registers them.
/// Receives new paths dynamically via `register_rx` when events carry a
/// `transcript_path`. Uses `linemux::MuxedLines` for real-time, line-safe
/// file tailing driven by OS filesystem events (notify crate).
pub(super) fn spawn_transcript_watcher(
    db: Arc<Mutex<Connection>>,
    mut register_rx: mpsc::Receiver<transcript::TranscriptRegisterRequest>,
) {
    tokio::spawn(async move {
        let mut muxed = match linemux::MuxedLines::new() {
            Ok(m) => m,
            Err(error) => {
                eprintln!("level=error event=linemux_init error={error:?}");
                return;
            }
        };

        let mut path_to_session: HashMap<PathBuf, String> = HashMap::new();

        let initial_sessions = {
            let db_guard = match db.lock() {
                Ok(g) => g,
                Err(error) => {
                    eprintln!("level=error event=linemux_init_load stage=lock error={error:?}");
                    return;
                }
            };
            match load_all_tracked_sessions(&db_guard) {
                Ok(sessions) => sessions,
                Err(error) => {
                    eprintln!("level=error event=linemux_init_load stage=query error={error:?}");
                    Vec::new()
                }
            }
        };

        for (session_id, transcript_path) in &initial_sessions {
            register_file(&mut muxed, &mut path_to_session, session_id, transcript_path).await;
        }

        loop {
            tokio::select! {
                line_result = muxed.next_line() => {
                    match line_result {
                        Ok(Some(line)) => {
                            let source = line.source();
                            if let Some(session_id) = path_to_session.get(source) {
                                let line_str = line.line();
                                transcript::persist_linemux_line(&db, session_id, source, line_str);
                            }
                        }
                        Ok(None) => break,
                        Err(error) => {
                            eprintln!("level=warn event=linemux_read_error error={error:?}");
                        }
                    }
                }
                Some(request) = register_rx.recv() => {
                    register_file(
                        &mut muxed,
                        &mut path_to_session,
                        &request.session_id,
                        &request.transcript_path,
                    ).await;
                }
            }
        }
    });
}

async fn register_file(
    muxed: &mut linemux::MuxedLines,
    path_to_session: &mut HashMap<PathBuf, String>,
    session_id: &str,
    transcript_path: &str,
) {
    let path = Path::new(transcript_path);

    // If path already tracked, update the session_id mapping
    if path_to_session.contains_key(path) {
        path_to_session.insert(path.to_path_buf(), session_id.to_string());
        return;
    }

    // Register new file with linemux
    match muxed.add_file(path).await {
        Ok(_) => {
            path_to_session.insert(path.to_path_buf(), session_id.to_string());
        }
        Err(error) => {
            if !matches!(error.kind(), std::io::ErrorKind::NotFound) {
                eprintln!(
                    "level=warn event=linemux_register session_id={session_id} path={transcript_path} error={error:?}"
                );
            }
        }
    }
}

fn load_all_tracked_sessions(
    db: &Connection,
) -> rusqlite::Result<Vec<(String, String)>> {
    let mut statement = db.prepare(
        "SELECT session_id, transcript_path
         FROM session_transcripts
         WHERE transcript_path != ''
         ORDER BY updated_at_ms DESC
         LIMIT 100",
    )?;
    let mut rows = statement.query([])?;
    let mut results = Vec::new();
    while let Some(row) = rows.next()? {
        results.push((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
        ));
    }
    Ok(results)
}
