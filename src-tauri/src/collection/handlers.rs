use super::*;

pub(super) async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        accepted: true,
        status: "ok",
    })
}

pub(super) async fn post_event(
    State(state): State<AppState>,
    Json(mut event): Json<IncomingEvent>,
) -> Result<Json<EventAck>, ApiError> {
    super::validate_event(&event)?;
    super::sanitize_json_value(&mut event.payload);

    let event_id = event.event_id.clone();
    match state.event_tx.try_send(event) {
        Ok(_) => {}
        Err(TrySendError::Full(_)) => {
            return Err(ApiError::QueueFull(
                "event queue is full, please retry later".to_string(),
            ));
        }
        Err(TrySendError::Closed(_)) => {
            return Err(ApiError::Internal(
                "event queue is closed".to_string(),
            ));
        }
    }

    Ok(Json(EventAck {
        accepted: true,
        event_id,
    }))
}

pub(super) async fn get_settings(State(state): State<AppState>) -> Result<Json<EvalConfig>, ApiError> {
    let config = state
        .eval_config_cache
        .read()
        .map_err(|error| ApiError::Internal(format!("failed to read eval config cache: {error:?}")))?
        .clone();
    Ok(Json(config))
}

pub(super) async fn save_settings(
    State(state): State<AppState>,
    Json(mut config): Json<EvalConfig>,
) -> Result<Json<EvalConfig>, ApiError> {
    if config.sampling_rate == 0 {
        config.sampling_rate = 1;
    }
    config.timeout_ms = config.timeout_ms.clamp(500, 120_000);
    if config.provider.trim().is_empty() {
        return Err(ApiError::BadRequest("provider cannot be empty".to_string()));
    }
    if config.model.trim().is_empty() {
        return Err(ApiError::BadRequest("model cannot be empty".to_string()));
    }
    if config.base_url.trim().is_empty() {
        return Err(ApiError::BadRequest("base_url cannot be empty".to_string()));
    }

    let db = state
        .db
        .lock()
        .map_err(|error| ApiError::Internal(format!("failed to lock sqlite connection: {error:?}")))?;
    db::persist_eval_config(&db, &config)
        .map_err(|error| ApiError::Internal(format!("failed to persist eval config: {error:?}")))?;
    drop(db);
    let mut cached = state
        .eval_config_cache
        .write()
        .map_err(|error| ApiError::Internal(format!("failed to write eval config cache: {error:?}")))?;
    *cached = config.clone();
    Ok(Json(config))
}

pub(super) async fn get_events(
    State(state): State<AppState>,
    Query(query): Query<EventQuery>,
) -> Result<Json<EventQueryResponse>, ApiError> {
    if query.session_id.trim().is_empty() {
        return Err(ApiError::BadRequest(
            "session_id is required for event query".to_string(),
        ));
    }

    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(50).clamp(1, 200);
    let offset = (page - 1) * page_size;

    let db = state
        .db
        .lock()
        .map_err(|error| ApiError::Internal(format!("failed to lock sqlite connection: {error:?}")))?;

    let mut where_clauses = vec!["session_id = ?".to_string(), "is_archived = 0".to_string()];
    let mut params: Vec<SqlValue> = vec![SqlValue::Text(query.session_id.clone())];

    if let Some(from_ms) = query.from_ms {
        where_clauses.push("created_at_ms >= ?".to_string());
        params.push(SqlValue::Integer(from_ms));
    }

    if let Some(to_ms) = query.to_ms {
        where_clauses.push("created_at_ms <= ?".to_string());
        params.push(SqlValue::Integer(to_ms));
    }

    if let Some(event_type) = &query.event_type {
        where_clauses.push("event_type = ?".to_string());
        params.push(SqlValue::Text(event_type.clone()));
    }

    let where_sql = where_clauses.join(" AND ");
    let count_sql = format!("SELECT COUNT(1) FROM events WHERE {where_sql}");
    let total = db
        .query_row(&count_sql, params_from_iter(params.iter()), |row| row.get::<_, i64>(0))
        .map_err(|error| ApiError::Internal(format!("failed to query total events: {error:?}")))?
        as u64;

    let data_sql = format!(
        "SELECT event_id, session_id, event_type, payload, created_at_ms
         FROM events WHERE {where_sql}
         ORDER BY created_at_ms DESC
         LIMIT ? OFFSET ?"
    );

    let mut data_params = params.clone();
    data_params.push(SqlValue::Integer(page_size as i64));
    data_params.push(SqlValue::Integer(offset as i64));

    let mut statement = db
        .prepare(&data_sql)
        .map_err(|error| ApiError::Internal(format!("failed to prepare event query: {error:?}")))?;

    let mut rows = statement
        .query(params_from_iter(data_params.iter()))
        .map_err(|error| ApiError::Internal(format!("failed to execute event query: {error:?}")))?;

    let mut items = Vec::new();
    while let Some(row) = rows
        .next()
        .map_err(|error| ApiError::Internal(format!("failed to read event row: {error:?}")))?
    {
        let payload_str: String = row
            .get(3)
            .map_err(|error| ApiError::Internal(format!("failed to decode payload: {error:?}")))?;
        let mut payload = serde_json::from_str::<Value>(&payload_str).map_err(|error| {
            ApiError::Internal(format!("failed to parse persisted payload json: {error:?}"))
        })?;
        if let Some(payload_object) = payload.as_object_mut() {
            payload_object.remove("raw_stdin");
        }

        items.push(EventItem {
            event_id: row.get(0).map_err(|error| {
                ApiError::Internal(format!("failed to decode event_id from sqlite: {error:?}"))
            })?,
            session_id: row.get(1).map_err(|error| {
                ApiError::Internal(format!("failed to decode session_id from sqlite: {error:?}"))
            })?,
            event_type: row.get(2).map_err(|error| {
                ApiError::Internal(format!("failed to decode event_type from sqlite: {error:?}"))
            })?,
            payload,
            created_at_ms: row.get(4).map_err(|error| {
                ApiError::Internal(format!(
                    "failed to decode created_at_ms from sqlite: {error:?}"
                ))
            })?,
        });
    }

    Ok(Json(EventQueryResponse {
        items,
        total,
        page,
        page_size,
    }))
}

pub(super) async fn get_sessions(State(state): State<AppState>) -> Result<Json<Vec<SessionItem>>, ApiError> {
    let db = state
        .db
        .lock()
        .map_err(|error| ApiError::Internal(format!("failed to lock sqlite connection: {error:?}")))?;

    let sql = "
        SELECT
            s.session_id,
            s.project_name,
            s.agent_type,
            s.last_active_at_ms,
            COALESCE((
                SELECT ev.risk_level
                FROM evaluations ev
                WHERE ev.session_id = s.session_id
                ORDER BY ev.created_at_ms DESC
                LIMIT 1
            ), 'none') AS latest_risk_level,
            COALESCE((
                SELECT COUNT(1)
                FROM evaluations ev2
                WHERE ev2.session_id = s.session_id
            ), 0) AS evaluation_count,
            s.first_prompt,
            s.duration_minutes,
            s.input_tokens,
            s.output_tokens,
            s.goal,
            s.summary,
            s.outcome,
            s.source
        FROM sessions s
        WHERE EXISTS (
            SELECT 1 FROM events e
            WHERE e.session_id = s.session_id AND e.is_archived = 0
        ) OR s.source = 'discovery'
        ORDER BY s.last_active_at_ms DESC
    ";

    let mut statement = db
        .prepare(sql)
        .map_err(|error| ApiError::Internal(format!("failed to prepare sessions query: {error:?}")))?;
    let mut rows = statement
        .query([])
        .map_err(|error| ApiError::Internal(format!("failed to execute sessions query: {error:?}")))?;

    let mut items = Vec::new();
    while let Some(row) = rows
        .next()
        .map_err(|error| ApiError::Internal(format!("failed to read session row: {error:?}")))?
    {
        items.push(SessionItem {
            session_id: row
                .get(0)
                .map_err(|error| ApiError::Internal(format!("failed to decode session_id: {error:?}")))?,
            project_name: row
                .get(1)
                .map_err(|error| ApiError::Internal(format!("failed to decode project_name: {error:?}")))?,
            agent_type: row
                .get(2)
                .map_err(|error| ApiError::Internal(format!("failed to decode agent_type: {error:?}")))?,
            last_active_at_ms: row.get(3).map_err(|error| {
                ApiError::Internal(format!("failed to decode last_active_at_ms: {error:?}"))
            })?,
            latest_risk_level: row.get(4).map_err(|error| {
                ApiError::Internal(format!("failed to decode latest_risk_level: {error:?}"))
            })?,
            evaluation_count: row.get::<_, i64>(5).map_err(|error| {
                ApiError::Internal(format!("failed to decode evaluation_count: {error:?}"))
            })? as u64,
            first_prompt: row.get(6).map_err(|error| {
                ApiError::Internal(format!("failed to decode first_prompt: {error:?}"))
            })?,
            duration_minutes: row.get(7).map_err(|error| {
                ApiError::Internal(format!("failed to decode duration_minutes: {error:?}"))
            })?,
            input_tokens: row.get(8).map_err(|error| {
                ApiError::Internal(format!("failed to decode input_tokens: {error:?}"))
            })?,
            output_tokens: row.get(9).map_err(|error| {
                ApiError::Internal(format!("failed to decode output_tokens: {error:?}"))
            })?,
            goal: row.get(10).map_err(|error| {
                ApiError::Internal(format!("failed to decode goal: {error:?}"))
            })?,
            summary: row.get(11).map_err(|error| {
                ApiError::Internal(format!("failed to decode summary: {error:?}"))
            })?,
            outcome: row.get(12).map_err(|error| {
                ApiError::Internal(format!("failed to decode outcome: {error:?}"))
            })?,
            source: row.get(13).map_err(|error| {
                ApiError::Internal(format!("failed to decode source: {error:?}"))
            })?,
        });
    }

    Ok(Json(items))
}

pub(super) async fn discover_sessions(
    State(state): State<AppState>,
) -> Result<Json<discovery::DiscoverResult>, ApiError> {
    let discovered = discovery::scan_session_meta();
    let mut db = state
        .db
        .lock()
        .map_err(|error| ApiError::Internal(format!("failed to lock sqlite connection: {error:?}")))?;
    let result = discovery::import_discovered_sessions(&mut db, &discovered);
    Ok(Json(result))
}

pub(super) async fn get_transcript(
    State(state): State<AppState>,
    Query(query): Query<TranscriptQuery>,
) -> Result<Json<TranscriptItem>, ApiError> {
    if query.session_id.trim().is_empty() {
        return Err(ApiError::BadRequest(
            "session_id is required for transcript query".to_string(),
        ));
    }

    if query.before_line_no.is_none() {
        let sync_state = {
            let db = state.db.lock().ok();
            db.and_then(|db| transcript::load_transcript_sync_state(&db, &query.session_id).ok().flatten())
        };
        if let Some(sync_state) = sync_state {
            let path = sync_state.transcript_path.clone();
            match transcript::read_transcript_increment(&path, Some(&sync_state)) {
                Ok(read_result) if !read_result.lines.is_empty() => {
                    if let Ok(mut db) = state.db.lock() {
                        if let Err(error) = transcript::upsert_transcript_sync_state(
                            &mut db, &query.session_id, &path, &read_result,
                        ) {
                            transcript::record_transcript_error(
                                &state, &query.session_id, &path,
                                format!("auto-sync upsert failed: {error:?}"),
                                format!("{error:?}"),
                            );
                        }
                    }
                }
                Err(error) => {
                    transcript::record_transcript_error(
                        &state, &query.session_id, &path,
                        format!("auto-sync read failed: {error}"),
                        format!("{error:?}"),
                    );
                }
                _ => {}
            }
            if std::path::Path::new(&path).exists() {
                let _ = state.transcript_register_tx.try_send(transcript::TranscriptRegisterRequest {
                    session_id: query.session_id.clone(),
                    transcript_path: path,
                });
            }
        }
    }

    let db = state
        .db
        .lock()
        .map_err(|error| ApiError::Internal(format!("failed to lock sqlite connection: {error:?}")))?;

    let page_size = query.page_size.unwrap_or(200).clamp(1, 500) as usize;

    let metadata_result = db.query_row(
        "
        SELECT
            updated_at_ms,
            imported_offset_bytes,
            last_error_message,
            last_error_stack
        FROM session_transcripts
        WHERE session_id = ?1
        ",
        params![query.session_id.clone()],
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        },
    );

    let (updated_at_ms, imported_offset_bytes, last_error_message, last_error_stack) = match metadata_result
    {
        Ok(meta) => meta,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return Ok(Json(TranscriptItem {
                session_id: query.session_id,
                items: Vec::new(),
                has_more: false,
                next_before_line_no: None,
                updated_at_ms: 0,
                imported_offset_bytes: 0,
                last_error_message: None,
                last_error_stack: None,
            }));
        }
        Err(error) => {
            return Err(ApiError::Internal(format!(
                "failed to query transcript metadata from sqlite: {error:?}"
            )));
        }
    };

    let mut statement = if query.before_line_no.is_some() {
        db.prepare(
            "
            SELECT line_no, line_content
            FROM session_transcript_lines
            WHERE session_id = ?1
              AND line_no < ?2
            ORDER BY line_no DESC
            LIMIT ?3
            ",
        )
        .map_err(|error| ApiError::Internal(format!("failed to prepare transcript query: {error:?}")))?
    } else {
        db.prepare(
            "
            SELECT line_no, line_content
            FROM session_transcript_lines
            WHERE session_id = ?1
            ORDER BY line_no DESC
            LIMIT ?2
            ",
        )
        .map_err(|error| ApiError::Internal(format!("failed to prepare transcript query: {error:?}")))?
    };

    let limit = (page_size + 1) as i64;
    let mut rows = if let Some(before_line_no) = query.before_line_no {
        statement
            .query(params![query.session_id.clone(), before_line_no, limit])
            .map_err(|error| ApiError::Internal(format!("failed to execute transcript query: {error:?}")))?
    } else {
        statement
            .query(params![query.session_id.clone(), limit])
            .map_err(|error| ApiError::Internal(format!("failed to execute transcript query: {error:?}")))?
    };

    let mut items_desc: Vec<TranscriptLineItem> = Vec::new();
    while let Some(row) = rows
        .next()
        .map_err(|error| ApiError::Internal(format!("failed to read transcript row: {error:?}")))?
    {
        items_desc.push(TranscriptLineItem {
            line_no: row.get(0).map_err(|error| {
                ApiError::Internal(format!("failed to decode transcript line_no: {error:?}"))
            })?,
            line_content: row.get(1).map_err(|error| {
                ApiError::Internal(format!("failed to decode transcript line_content: {error:?}"))
            })?,
        });
    }

    let has_more = items_desc.len() > page_size;
    if has_more {
        items_desc.truncate(page_size);
    }
    items_desc.reverse();
    let next_before_line_no = if has_more {
        items_desc.first().map(|item| item.line_no)
    } else {
        None
    };

    Ok(Json(TranscriptItem {
        session_id: query.session_id,
        items: items_desc,
        has_more,
        next_before_line_no,
        updated_at_ms,
        imported_offset_bytes,
        last_error_message,
        last_error_stack,
    }))
}

pub(super) async fn get_evaluations(
    State(state): State<AppState>,
    Query(query): Query<EvaluationQuery>,
) -> Result<Json<EvaluationQueryResponse>, ApiError> {
    if query.session_id.trim().is_empty() {
        return Err(ApiError::BadRequest(
            "session_id is required for evaluation query".to_string(),
        ));
    }

    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(50).clamp(1, 200);
    let offset = (page - 1) * page_size;

    let db = state
        .db
        .lock()
        .map_err(|error| ApiError::Internal(format!("failed to lock sqlite connection: {error:?}")))?;

    let mut where_clauses = vec!["session_id = ?".to_string()];
    let mut params: Vec<SqlValue> = vec![SqlValue::Text(query.session_id.clone())];

    if let Some(from_ms) = query.from_ms {
        where_clauses.push("created_at_ms >= ?".to_string());
        params.push(SqlValue::Integer(from_ms));
    }

    if let Some(to_ms) = query.to_ms {
        where_clauses.push("created_at_ms <= ?".to_string());
        params.push(SqlValue::Integer(to_ms));
    }

    let where_sql = where_clauses.join(" AND ");
    let count_sql = format!("SELECT COUNT(1) FROM evaluations WHERE {where_sql}");
    let total = db
        .query_row(&count_sql, params_from_iter(params.iter()), |row| row.get::<_, i64>(0))
        .map_err(|error| ApiError::Internal(format!("failed to count evaluations: {error:?}")))?
        as u64;

    let data_sql = format!(
        "SELECT evaluation_id, session_id, event_id, provider, model, base_url, risk_level,
                risk_category, efficiency_level, suggestion, status, error_message,
                error_stack, retry_count, created_at_ms
         FROM evaluations
         WHERE {where_sql}
         ORDER BY created_at_ms DESC
         LIMIT ? OFFSET ?"
    );
    let mut data_params = params.clone();
    data_params.push(SqlValue::Integer(page_size as i64));
    data_params.push(SqlValue::Integer(offset as i64));

    let mut statement = db
        .prepare(&data_sql)
        .map_err(|error| ApiError::Internal(format!("failed to prepare evaluation query: {error:?}")))?;
    let mut rows = statement
        .query(params_from_iter(data_params.iter()))
        .map_err(|error| ApiError::Internal(format!("failed to execute evaluation query: {error:?}")))?;

    let mut items = Vec::new();
    while let Some(row) = rows
        .next()
        .map_err(|error| ApiError::Internal(format!("failed to read evaluation row: {error:?}")))?
    {
        items.push(EvaluationItem {
            evaluation_id: row.get(0).map_err(|error| {
                ApiError::Internal(format!("failed to decode evaluation_id: {error:?}"))
            })?,
            session_id: row
                .get(1)
                .map_err(|error| ApiError::Internal(format!("failed to decode session_id: {error:?}")))?,
            event_id: row
                .get(2)
                .map_err(|error| ApiError::Internal(format!("failed to decode event_id: {error:?}")))?,
            provider: row
                .get(3)
                .map_err(|error| ApiError::Internal(format!("failed to decode provider: {error:?}")))?,
            model: row
                .get(4)
                .map_err(|error| ApiError::Internal(format!("failed to decode model: {error:?}")))?,
            base_url: row.get(5).map_err(|error| {
                ApiError::Internal(format!("failed to decode base_url: {error:?}"))
            })?,
            risk_level: row
                .get(6)
                .map_err(|error| ApiError::Internal(format!("failed to decode risk_level: {error:?}")))?,
            risk_category: row.get(7).map_err(|error| {
                ApiError::Internal(format!("failed to decode risk_category: {error:?}"))
            })?,
            efficiency_level: row.get(8).map_err(|error| {
                ApiError::Internal(format!("failed to decode efficiency_level: {error:?}"))
            })?,
            suggestion: row
                .get(9)
                .map_err(|error| ApiError::Internal(format!("failed to decode suggestion: {error:?}")))?,
            status: row
                .get(10)
                .map_err(|error| ApiError::Internal(format!("failed to decode status: {error:?}")))?,
            error_message: row.get(11).map_err(|error| {
                ApiError::Internal(format!("failed to decode error_message: {error:?}"))
            })?,
            error_stack: row
                .get(12)
                .map_err(|error| ApiError::Internal(format!("failed to decode error_stack: {error:?}")))?,
            retry_count: row.get::<_, i64>(13).map_err(|error| {
                ApiError::Internal(format!("failed to decode retry_count: {error:?}"))
            })? as u32,
            created_at_ms: row.get(14).map_err(|error| {
                ApiError::Internal(format!("failed to decode created_at_ms: {error:?}"))
            })?,
        });
    }

    Ok(Json(EvaluationQueryResponse {
        items,
        total,
        page,
        page_size,
    }))
}

pub(super) async fn retry_evaluation(
    State(state): State<AppState>,
    Json(request): Json<RetryEvaluationRequest>,
) -> Result<Json<ApiErrorBody>, ApiError> {
    let db = state
        .db
        .lock()
        .map_err(|error| ApiError::Internal(format!("failed to lock sqlite connection: {error:?}")))?;

    let mut statement = db.prepare(
        "SELECT session_id, event_id, input_summary, retry_count
         FROM evaluations WHERE evaluation_id = ?1",
    )
    .map_err(|error| ApiError::Internal(format!("failed to prepare retry query: {error:?}")))?;

    let row = statement
        .query_row(params![request.evaluation_id.clone()], |row| {
            let session_id: String = row.get(0)?;
            let event_id: Option<String> = row.get(1)?;
            let input_summary: String = row.get(2)?;
            let retry_count: i64 = row.get(3)?;
            Ok((session_id, event_id, input_summary, retry_count))
        })
        .map_err(|error| {
            ApiError::BadRequest(format!(
                "cannot retry evaluation id={} error={error:?}",
                request.evaluation_id
            ))
        })?;

    drop(statement);
    drop(db);

    let input_summary: Value = serde_json::from_str(&row.2).map_err(|error| {
        ApiError::Internal(format!("failed to parse evaluation input summary: {error:?}"))
    })?;
    let event_type = input_summary
        .get("event_type")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown-event")
        .to_string();
    let payload = input_summary.get("payload").cloned().unwrap_or(Value::Null);

    match state.eval_tx.try_send(EvaluationJob {
        evaluation_id: request.evaluation_id,
        event_id: row.1,
        session_id: row.0,
        event_type,
        payload,
        retry_count: (row.3 as u32) + 1,
    }) {
        Ok(_) => {}
        Err(TrySendError::Full(_)) => {
            return Err(ApiError::QueueFull(
                "evaluation queue is full, please retry later".to_string(),
            ));
        }
        Err(TrySendError::Closed(_)) => {
            return Err(ApiError::Internal(
                "evaluation queue is closed, cannot enqueue retry".to_string(),
            ));
        }
    }

    Ok(Json(ApiErrorBody {
        accepted: true,
        error: String::new(),
        error_code: String::new(),
        retryable: false,
    }))
}

pub(super) async fn archive_session(
    State(state): State<AppState>,
    Json(request): Json<ArchiveSessionRequest>,
) -> Result<Json<ApiErrorBody>, ApiError> {
    if request.session_id.trim().is_empty() {
        return Err(ApiError::BadRequest("session_id cannot be empty".to_string()));
    }

    let archive_before_ms = now_timestamp_ms();
    let db = state
        .db
        .lock()
        .map_err(|error| ApiError::Internal(format!("failed to lock sqlite connection: {error:?}")))?;

    db.execute(
        "
        UPDATE events
        SET is_archived = 1
        WHERE session_id = ?1
          AND is_archived = 0
          AND created_at_ms <= ?2
        ",
        params![request.session_id, archive_before_ms],
    )
    .map_err(|error| ApiError::Internal(format!("failed to archive session events: {error:?}")))?;

    db.execute(
        "
        UPDATE session_transcripts
        SET transcript_path = ''
        WHERE session_id = ?1
        ",
        params![request.session_id],
    )
    .map_err(|error| ApiError::Internal(format!("failed to clear transcript path on archive: {error:?}")))?;

    Ok(Json(ApiErrorBody {
        accepted: true,
        error: String::new(),
        error_code: String::new(),
        retryable: false,
    }))
}

pub(super) async fn get_hooks() -> Result<Json<hooks::HooksResponse>, ApiError> {
    hooks::get_hooks()
        .map(Json)
        .map_err(ApiError::Internal)
}

pub(super) async fn save_hooks(
    Json(payload): Json<hooks::HooksResponse>,
) -> Result<Json<hooks::HooksResponse>, ApiError> {
    hooks::save_hooks(payload)
        .map(Json)
        .map_err(ApiError::Internal)
}

pub(super) async fn init_hooks() -> Result<Json<hooks::HooksInitResponse>, ApiError> {
    hooks::init_hooks()
        .map(Json)
        .map_err(ApiError::Internal)
}

pub(super) async fn sync_transcript(
    State(state): State<AppState>,
    Json(request): Json<SyncTranscriptRequest>,
) -> Result<Json<SyncTranscriptResponse>, ApiError> {
    if request.session_id.trim().is_empty() {
        return Err(ApiError::BadRequest("session_id cannot be empty".to_string()));
    }

    let existing_state = {
        let db = state
            .db
            .lock()
            .map_err(|error| ApiError::Internal(format!("failed to lock sqlite connection: {error:?}")))?;
        transcript::load_transcript_sync_state(&db, &request.session_id)
            .map_err(|error| ApiError::Internal(format!("failed to load transcript sync state: {error:?}")))?
    };

    let Some(sync_state) = existing_state else {
        return Ok(Json(SyncTranscriptResponse {
            accepted: true,
            session_id: request.session_id,
            lines_imported: 0,
            message: "no transcript record found for this session".to_string(),
        }));
    };

    let transcript_path = sync_state.transcript_path.clone();

    let read_result = match transcript::read_transcript_increment(&transcript_path, Some(&sync_state)) {
        Ok(result) => result,
        Err(error) => {
            transcript::record_transcript_error(
                &state,
                &request.session_id,
                &transcript_path,
                error.to_string(),
                format!("{error:?}"),
            );
            return Err(ApiError::Internal(format!(
                "failed to read transcript file: {error}"
            )));
        }
    };

    let lines_imported = read_result.lines.len();

    let mut db = state
        .db
        .lock()
        .map_err(|error| ApiError::Internal(format!("failed to lock sqlite connection: {error:?}")))?;

    transcript::upsert_transcript_sync_state(&mut db, &request.session_id, &transcript_path, &read_result)
        .map_err(|error| ApiError::Internal(format!("failed to upsert transcript sync state: {error:?}")))?;

    drop(db);

    let _ = state.transcript_register_tx.try_send(transcript::TranscriptRegisterRequest {
        session_id: request.session_id.clone(),
        transcript_path,
    });

    Ok(Json(SyncTranscriptResponse {
        accepted: true,
        session_id: request.session_id,
        lines_imported,
        message: format!("synced {lines_imported} new lines from transcript file"),
    }))
}
