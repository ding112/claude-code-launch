use super::*;

pub(super) fn enqueue_evaluation_for_event(
    state: &AppState,
    event: &IncomingEvent,
) -> Result<(), ApiError> {
    let config = state
        .eval_config_cache
        .read()
        .map_err(|error| ApiError::Internal(format!("failed to read eval config cache: {error:?}")))?
        .clone();

    if !config.enabled {
        return Ok(());
    }

    let sampling_rate = config.sampling_rate.max(1);
    let count = state.eval_counter.fetch_add(1, Ordering::Relaxed) + 1;
    if count % sampling_rate as u64 != 0 {
        return Ok(());
    }

    let job = EvaluationJob {
        evaluation_id: format!("eval-{}", Uuid::new_v4()),
        event_id: Some(event.event_id.clone()),
        session_id: event.session_id.clone(),
        event_type: event.event_type.clone(),
        payload: event.payload.clone(),
        retry_count: 0,
    };

    match state.eval_tx.try_send(job) {
        Ok(_) => {}
        Err(TrySendError::Full(_)) => {
            return Err(ApiError::QueueFull(format!(
                "evaluation queue is full for event {}, please retry later",
                event.event_id
            )));
        }
        Err(TrySendError::Closed(_)) => {
            return Err(ApiError::Internal(format!(
                "evaluation queue is closed for event {}",
                event.event_id
            )));
        }
    }

    Ok(())
}

pub(super) fn spawn_evaluation_worker(
    db: Arc<Mutex<Connection>>,
    eval_config_cache: Arc<RwLock<EvalConfig>>,
    mut eval_rx: mpsc::Receiver<EvaluationJob>,
) {
    tokio::spawn(async move {
        while let Some(job) = eval_rx.recv().await {
            let db_clone = db.clone();
            let eval_config_cache_clone = eval_config_cache.clone();
            let job_clone = job.clone();
            let result = tokio::task::spawn_blocking(move || {
                process_evaluation_job(&db_clone, &eval_config_cache_clone, &job_clone)
            })
            .await
            .map_err(|error| format!("evaluation worker join error: {error:?}"))
            .and_then(|inner| inner);

            if let Err(error) = result {
                eprintln!(
                    "level=error event=evaluation_job_failed evaluation_id={} session_id={} error={error:?}",
                    job.evaluation_id, job.session_id
                );
            }
        }
    });
}

fn process_evaluation_job(
    db: &Arc<Mutex<Connection>>,
    eval_config_cache: &Arc<RwLock<EvalConfig>>,
    job: &EvaluationJob,
) -> Result<(), String> {
    let config = eval_config_cache
        .read()
        .map_err(|error| format!("failed to read eval config cache in worker: {error:?}"))?
        .clone();

    let created_at_ms = now_timestamp_ms();
    let input_summary = serde_json::json!({
        "event_type": job.event_type,
        "payload": job.payload,
    });
    let input_summary_text = serde_json::to_string(&input_summary)
        .map_err(|error| format!("failed to serialize input summary: {error:?}"))?;

    let input = EvaluationInput {
        event_id: job
            .event_id
            .clone()
            .unwrap_or_else(|| format!("retry-{}", job.evaluation_id)),
        event_type: job.event_type.clone(),
        payload: job.payload.clone(),
    };

    let evaluation_result = evaluation::evaluate(&input, &config);

    let db_guard = db
        .lock()
        .map_err(|error| format!("failed to lock sqlite connection for evaluation: {error:?}"))?;

    match evaluation_result {
        Ok(result) => {
            db_guard
                .execute(
                    "
                    INSERT INTO evaluations(
                        evaluation_id, session_id, event_id, provider, model, base_url, risk_level,
                        risk_category, efficiency_level, suggestion, status,
                        error_message, error_stack, input_summary, retry_count,
                        created_at_ms, updated_at_ms
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'success', NULL, NULL, ?11, ?12, ?13, ?14)
                    ON CONFLICT(evaluation_id) DO UPDATE SET
                        provider = excluded.provider,
                        model = excluded.model,
                        base_url = excluded.base_url,
                        risk_level = excluded.risk_level,
                        risk_category = excluded.risk_category,
                        efficiency_level = excluded.efficiency_level,
                        suggestion = excluded.suggestion,
                        status = excluded.status,
                        error_message = excluded.error_message,
                        error_stack = excluded.error_stack,
                        input_summary = excluded.input_summary,
                        retry_count = excluded.retry_count,
                        updated_at_ms = excluded.updated_at_ms
                    ",
                    params![
                        job.evaluation_id,
                        job.session_id,
                        job.event_id,
                        config.provider,
                        config.model,
                        config.base_url,
                        result.risk_level,
                        result.risk_category,
                        result.efficiency_level,
                        result.suggestion,
                        input_summary_text,
                        job.retry_count as i64,
                        created_at_ms,
                        created_at_ms
                    ],
                )
                .map_err(|error| format!("failed to write successful evaluation: {error:?}"))?;
        }
        Err(error_message) => {
            db_guard
                .execute(
                    "
                    INSERT INTO evaluations(
                        evaluation_id, session_id, event_id, provider, model, base_url, risk_level,
                        risk_category, efficiency_level, suggestion, status,
                        error_message, error_stack, input_summary, retry_count,
                        created_at_ms, updated_at_ms
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'unknown', 'unknown', 'unknown', '', 'failed', ?7, ?8, ?9, ?10, ?11, ?12)
                    ON CONFLICT(evaluation_id) DO UPDATE SET
                        provider = excluded.provider,
                        model = excluded.model,
                        base_url = excluded.base_url,
                        status = excluded.status,
                        error_message = excluded.error_message,
                        error_stack = excluded.error_stack,
                        input_summary = excluded.input_summary,
                        retry_count = excluded.retry_count,
                        updated_at_ms = excluded.updated_at_ms
                    ",
                    params![
                        job.evaluation_id,
                        job.session_id,
                        job.event_id,
                        config.provider,
                        config.model,
                        config.base_url,
                        error_message,
                        error_message,
                        input_summary_text,
                        job.retry_count as i64,
                        created_at_ms,
                        created_at_ms
                    ],
                )
                .map_err(|error| format!("failed to write failed evaluation: {error:?}"))?;
        }
    }
    Ok(())
}
