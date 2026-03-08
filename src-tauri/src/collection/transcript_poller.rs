use super::*;
use std::time::Duration;

const POLL_INTERVAL_SECS: u64 = 5;

pub(super) fn spawn_transcript_poller(db: Arc<Mutex<Connection>>) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
            let db = db.clone();
            let _ = tokio::task::spawn_blocking(move || {
                poll_all_transcripts(&db);
            })
            .await;
        }
    });
}

fn poll_all_transcripts(db: &Arc<Mutex<Connection>>) {
    let sessions = {
        let db = match db.lock() {
            Ok(db) => db,
            Err(error) => {
                eprintln!(
                    "level=error event=transcript_poller stage=lock_sqlite error={error:?}"
                );
                return;
            }
        };
        match load_all_sync_states(&db) {
            Ok(sessions) => sessions,
            Err(error) => {
                eprintln!(
                    "level=error event=transcript_poller stage=load_sync_states error={error:?}"
                );
                return;
            }
        }
    };

    for (session_id, sync_state) in &sessions {
        let path = &sync_state.transcript_path;
        let read_result = match transcript::read_transcript_increment(path, Some(sync_state)) {
            Ok(result) => result,
            Err(error) => {
                if error.kind() != std::io::ErrorKind::NotFound {
                    eprintln!(
                        "level=warn event=transcript_poller stage=read_increment session_id={session_id} path={path} error={error:?}"
                    );
                }
                continue;
            }
        };

        if read_result.lines.is_empty() {
            continue;
        }

        let mut db = match db.lock() {
            Ok(db) => db,
            Err(error) => {
                eprintln!(
                    "level=error event=transcript_poller stage=lock_sqlite_upsert session_id={session_id} error={error:?}"
                );
                continue;
            }
        };

        if let Err(error) =
            transcript::upsert_transcript_sync_state(&mut db, session_id, path, &read_result)
        {
            eprintln!(
                "level=error event=transcript_poller stage=upsert session_id={session_id} error={error:?}"
            );
        }
    }
}

fn load_all_sync_states(
    db: &Connection,
) -> rusqlite::Result<Vec<(String, transcript::TranscriptSyncState)>> {
    let mut statement = db.prepare(
        "
        SELECT session_id, transcript_path, imported_offset_bytes, file_mtime_ms, pending_fragment
        FROM session_transcripts
        WHERE transcript_path != ''
        ",
    )?;
    let mut rows = statement.query([])?;
    let mut results = Vec::new();
    while let Some(row) = rows.next()? {
        results.push((
            row.get::<_, String>(0)?,
            transcript::TranscriptSyncState {
                transcript_path: row.get(1)?,
                imported_offset_bytes: row.get::<_, i64>(2)?.max(0),
                file_mtime_ms: row.get(3)?,
                pending_fragment: row.get(4)?,
            },
        ));
    }
    Ok(results)
}
