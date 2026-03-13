use crate::evaluation::{self, EvalConfig, EvaluationInput};
use axum::extract::{Query, State};
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderValue, Method};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use rusqlite::types::Value as SqlValue;
use rusqlite::{params, params_from_iter, Connection};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TrySendError;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

#[path = "collection/db.rs"]
mod db;
#[path = "collection/eval_queue.rs"]
mod eval_queue;
#[path = "collection/handlers.rs"]
mod handlers;
#[path = "collection/hooks.rs"]
mod hooks;
#[path = "collection/transcript.rs"]
mod transcript;
#[path = "collection/transcript_poller.rs"]
mod transcript_poller;

#[derive(Clone)]
pub struct AppState {
    db: Arc<Mutex<Connection>>,
    eval_tx: mpsc::Sender<EvaluationJob>,
    event_tx: mpsc::Sender<IncomingEvent>,
    #[allow(dead_code)]
    eval_counter: Arc<AtomicU64>,
    eval_config_cache: Arc<RwLock<EvalConfig>>,
    transcript_register_tx: mpsc::Sender<transcript::TranscriptRegisterRequest>,
}

impl AppState {
    pub fn new(db_path: &str) -> Result<Self, String> {
        let db = Connection::open(db_path)
            .map_err(|error| format!("failed to open sqlite db at {db_path}: {error:?}"))?;
        db::init_schema(&db).map_err(|error| format!("failed to init sqlite schema: {error:?}"))?;
        let eval_config = db::load_eval_config(&db)
            .map_err(|error| format!("failed to load eval config from sqlite: {error:?}"))?;
        let db = Arc::new(Mutex::new(db));
        let (eval_tx, eval_rx) = mpsc::channel(128);
        let (event_tx, event_rx) = mpsc::channel::<IncomingEvent>(512);
        let (transcript_register_tx, transcript_register_rx) =
            mpsc::channel::<transcript::TranscriptRegisterRequest>(256);
        let eval_counter = Arc::new(AtomicU64::new(0));
        let eval_config_cache = Arc::new(RwLock::new(eval_config));
        eval_queue::spawn_evaluation_worker(db.clone(), eval_config_cache.clone(), eval_rx);
        transcript_poller::spawn_transcript_watcher(db.clone(), transcript_register_rx);
        spawn_event_worker(
            db.clone(),
            eval_tx.clone(),
            eval_counter.clone(),
            eval_config_cache.clone(),
            transcript_register_tx.clone(),
            event_rx,
        );

        Ok(Self {
            db,
            eval_tx,
            event_tx,
            eval_counter,
            eval_config_cache,
            transcript_register_tx,
        })
    }

    #[cfg(test)]
    fn new_in_memory() -> Result<Self, String> {
        let db = Connection::open_in_memory()
            .map_err(|error| format!("failed to open sqlite in-memory db: {error:?}"))?;
        db::init_schema(&db).map_err(|error| format!("failed to init sqlite schema: {error:?}"))?;
        let eval_config = db::load_eval_config(&db)
            .map_err(|error| format!("failed to load eval config from sqlite: {error:?}"))?;
        let db = Arc::new(Mutex::new(db));
        let (eval_tx, eval_rx) = mpsc::channel(128);
        let (event_tx, event_rx) = mpsc::channel::<IncomingEvent>(512);
        let (transcript_register_tx, transcript_register_rx) =
            mpsc::channel::<transcript::TranscriptRegisterRequest>(256);
        let eval_counter = Arc::new(AtomicU64::new(0));
        let eval_config_cache = Arc::new(RwLock::new(eval_config));
        eval_queue::spawn_evaluation_worker(db.clone(), eval_config_cache.clone(), eval_rx);
        transcript_poller::spawn_transcript_watcher(db.clone(), transcript_register_rx);
        spawn_event_worker(
            db.clone(),
            eval_tx.clone(),
            eval_counter.clone(),
            eval_config_cache.clone(),
            transcript_register_tx.clone(),
            event_rx,
        );

        Ok(Self {
            db,
            eval_tx,
            event_tx,
            eval_counter,
            eval_config_cache,
            transcript_register_tx,
        })
    }
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    accepted: bool,
    status: &'static str,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IncomingEvent {
    pub event_id: String,
    pub session_id: String,
    pub project_name: String,
    pub event_type: String,
    pub payload: Value,
    pub created_at_ms: i64,
}

#[derive(Debug, Serialize)]
pub struct EventAck {
    pub accepted: bool,
    pub duplicate: bool,
    pub event_id: String,
}

#[derive(Debug, Serialize)]
pub struct EventItem {
    pub event_id: String,
    pub session_id: String,
    pub event_type: String,
    pub payload: Value,
    pub created_at_ms: i64,
}

#[derive(Debug, Deserialize)]
pub struct EventQuery {
    pub session_id: String,
    pub from_ms: Option<i64>,
    pub to_ms: Option<i64>,
    pub event_type: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct TranscriptQuery {
    session_id: String,
    before_line_no: Option<i64>,
    page_size: Option<u32>,
}

#[derive(Debug, Serialize)]
struct TranscriptLineItem {
    line_no: i64,
    line_content: String,
}

#[derive(Debug, Serialize)]
struct TranscriptItem {
    session_id: String,
    items: Vec<TranscriptLineItem>,
    has_more: bool,
    next_before_line_no: Option<i64>,
    updated_at_ms: i64,
    imported_offset_bytes: i64,
    last_error_message: Option<String>,
    last_error_stack: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ArchiveSessionRequest {
    session_id: String,
}

#[derive(Debug, Deserialize)]
struct SyncTranscriptRequest {
    session_id: String,
}

#[derive(Debug, Serialize)]
struct SyncTranscriptResponse {
    accepted: bool,
    session_id: String,
    lines_imported: usize,
    message: String,
}

#[derive(Debug, Serialize)]
pub struct EventQueryResponse {
    pub items: Vec<EventItem>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Serialize)]
pub struct SessionItem {
    pub session_id: String,
    pub project_name: String,
    pub agent_type: String,
    pub last_active_at_ms: i64,
    pub latest_risk_level: String,
    pub evaluation_count: u64,
}

#[derive(Debug, Serialize)]
pub struct ApiErrorBody {
    pub accepted: bool,
    pub error: String,
    pub error_code: String,
    pub retryable: bool,
}

#[derive(Debug)]
enum ApiError {
    BadRequest(String),
    Internal(String),
    QueueFull(String),
}

#[derive(Debug, Clone)]
struct EvaluationJob {
    evaluation_id: String,
    event_id: Option<String>,
    session_id: String,
    event_type: String,
    payload: Value,
    retry_count: u32,
}

const TRANSCRIPT_SYNC_MAX_BYTES: usize = 1024 * 1024;

#[derive(Debug, Deserialize, Serialize)]
struct RetryEvaluationRequest {
    evaluation_id: String,
}

#[derive(Debug, Serialize)]
struct EvaluationItem {
    evaluation_id: String,
    session_id: String,
    event_id: Option<String>,
    provider: String,
    model: String,
    base_url: String,
    risk_level: String,
    risk_category: String,
    efficiency_level: String,
    suggestion: String,
    status: String,
    error_message: Option<String>,
    error_stack: Option<String>,
    retry_count: u32,
    created_at_ms: i64,
}

#[derive(Debug, Deserialize)]
struct EvaluationQuery {
    session_id: String,
    from_ms: Option<i64>,
    to_ms: Option<i64>,
    page: Option<u32>,
    page_size: Option<u32>,
}

#[derive(Debug, Serialize)]
struct EvaluationQueryResponse {
    items: Vec<EvaluationItem>,
    total: u64,
    page: u32,
    page_size: u32,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message, error_code, retryable) = match self {
            Self::BadRequest(message) => (
                StatusCode::BAD_REQUEST,
                message,
                "bad_request".to_string(),
                false,
            ),
            Self::Internal(message) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                message,
                "internal_error".to_string(),
                false,
            ),
            Self::QueueFull(message) => (
                StatusCode::TOO_MANY_REQUESTS,
                message,
                "evaluation_queue_full".to_string(),
                true,
            ),
        };

        (
            status,
            Json(ApiErrorBody {
                accepted: false,
                error: message,
                error_code,
                retryable,
            }),
        )
            .into_response()
    }
}

#[derive(Clone)]
struct EventWorkerContext {
    db: Arc<Mutex<Connection>>,
    eval_tx: mpsc::Sender<EvaluationJob>,
    eval_counter: Arc<AtomicU64>,
    eval_config_cache: Arc<RwLock<EvalConfig>>,
    transcript_register_tx: mpsc::Sender<transcript::TranscriptRegisterRequest>,
}

fn spawn_event_worker(
    db: Arc<Mutex<Connection>>,
    eval_tx: mpsc::Sender<EvaluationJob>,
    eval_counter: Arc<AtomicU64>,
    eval_config_cache: Arc<RwLock<EvalConfig>>,
    transcript_register_tx: mpsc::Sender<transcript::TranscriptRegisterRequest>,
    mut event_rx: mpsc::Receiver<IncomingEvent>,
) {
    tokio::spawn(async move {
        let ctx = EventWorkerContext {
            db,
            eval_tx,
            eval_counter,
            eval_config_cache,
            transcript_register_tx,
        };
        while let Some(event) = event_rx.recv().await {
            let ctx = ctx.clone();
            tokio::task::spawn_blocking(move || {
                process_incoming_event(&ctx, &event);
            })
            .await
            .ok();
        }
    });
}

fn process_incoming_event(ctx: &EventWorkerContext, event: &IncomingEvent) {
    let inserted = {
        let db = match ctx.db.lock() {
            Ok(db) => db,
            Err(error) => {
                eprintln!(
                    "level=error event=event_worker_lock_failed event_id={} error={error:?}",
                    event.event_id
                );
                return;
            }
        };
        match db::persist_event(&db, event) {
            Ok(inserted) => inserted,
            Err(error) => {
                eprintln!(
                    "level=error event=event_persist_failed event_id={} error={error:?}",
                    event.event_id
                );
                return;
            }
        }
    };

    transcript::sync_transcript_after_event(&ctx.db, &ctx.transcript_register_tx, event);

    if inserted {
        eval_queue::enqueue_for_worker(
            &ctx.eval_config_cache,
            &ctx.eval_counter,
            &ctx.eval_tx,
            event,
        );
    }
}

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(parse_allowed_origins())
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([CONTENT_TYPE]);

    Router::new()
        .route("/health", get(handlers::health))
        .route("/sessions", get(handlers::get_sessions))
        .route("/transcripts", get(handlers::get_transcript))
        .route("/transcripts/sync", post(handlers::sync_transcript))
        .route("/sessions/archive", post(handlers::archive_session))
        .route("/events", post(handlers::post_event).get(handlers::get_events))
        .route("/settings", get(handlers::get_settings).post(handlers::save_settings))
        .route("/evaluations", get(handlers::get_evaluations))
        .route("/evaluations/retry", post(handlers::retry_evaluation))
        .route("/hooks", get(handlers::get_hooks).post(handlers::save_hooks))
        .route("/hooks/init", post(handlers::init_hooks))
        .layer(cors)
        .with_state(state)
}

fn parse_allowed_origins() -> Vec<HeaderValue> {
    if let Ok(raw) = env::var("CODE_AGENT_OVERSEER_ALLOWED_ORIGINS") {
        let parsed: Vec<HeaderValue> = raw
            .split(',')
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .filter_map(|item| HeaderValue::from_str(item).ok())
            .collect();
        if !parsed.is_empty() {
            return parsed;
        }
    }

    vec![
        HeaderValue::from_static("http://localhost:1420"),
        HeaderValue::from_static("http://127.0.0.1:1420"),
        HeaderValue::from_static("tauri://localhost"),
    ]
}

pub async fn serve(addr: SocketAddr, db_path: String) -> Result<(), std::io::Error> {
    let state = AppState::new(&db_path).map_err(std::io::Error::other)?;
    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await
}

fn validate_event(event: &IncomingEvent) -> Result<(), ApiError> {
    if event.event_id.trim().is_empty() {
        return Err(ApiError::BadRequest("event_id cannot be empty".to_string()));
    }
    if event.session_id.trim().is_empty() {
        return Err(ApiError::BadRequest("session_id cannot be empty".to_string()));
    }
    if event.project_name.trim().is_empty() {
        return Err(ApiError::BadRequest("project_name cannot be empty".to_string()));
    }
    if event.event_type.trim().is_empty() {
        return Err(ApiError::BadRequest("event_type cannot be empty".to_string()));
    }
    Ok(())
}

fn is_sensitive_key(key_lower: &str) -> bool {
    key_lower.contains("token")
        || key_lower.contains("secret")
        || key_lower.contains("password")
        || key_lower.contains("credential")
        || key_lower.contains("private")
        || key_lower.contains("cert")
        || key_lower.contains("cookie")
        || key_lower == "authorization"
        || (key_lower.contains("key") && key_lower != "keyboard")
}

pub(crate) fn sanitize_json_value(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, nested) in map.iter_mut() {
                if is_sensitive_key(&key.to_ascii_lowercase()) {
                    *nested = Value::String("[REDACTED]".to_string());
                    continue;
                }
                sanitize_json_value(nested);
            }
        }
        Value::Array(items) => {
            for nested in items.iter_mut() {
                sanitize_json_value(nested);
            }
        }
        _ => {}
    }
}

fn now_timestamp_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Method, Request};
    use http_body_util::BodyExt;
    use serde_json::json;
    use std::fs::{self, OpenOptions};
    use std::io::Write;
    use std::path::PathBuf;
    use std::time::Duration;
    use tower::ServiceExt;
    use tokio::sync::mpsc;
    use tokio::time::sleep;

    #[tokio::test]
    async fn event_insert_is_idempotent() {
        let state = AppState::new_in_memory().expect("in-memory sqlite init should succeed");
        let app = build_router(state.clone());

        let event = json!({
            "event_id": "evt-1",
            "session_id": "session-1",
            "project_name": "project-a",
            "event_type": "tool.call",
            "payload": {"apiKey": "super-secret"},
            "created_at_ms": 1000
        });

        let first = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/events")
                    .header("content-type", "application/json")
                    .body(Body::from(event.to_string()))
                    .expect("post request should build"),
            )
            .await
            .expect("first post should complete");
        assert_eq!(first.status(), StatusCode::OK);

        wait_for_event_persisted(&state, "evt-1").await;

        let second = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/events")
                    .header("content-type", "application/json")
                    .body(Body::from(event.to_string()))
                    .expect("duplicate request should build"),
            )
            .await
            .expect("duplicate post should complete");
        assert_eq!(second.status(), StatusCode::OK);

        wait_for_stable_event_count(&state, "session-1", 1).await;

        let db = state.db.lock().expect("db lock should succeed");
        let count: i64 = db
            .query_row(
                "SELECT COUNT(1) FROM events WHERE event_id = 'evt-1'",
                [],
                |row| row.get(0),
            )
            .expect("count query should succeed");
        assert_eq!(count, 1, "duplicate event_id should not create a second row");
    }

    #[tokio::test]
    async fn event_query_supports_pagination() {
        let state = AppState::new_in_memory().expect("in-memory sqlite init should succeed");
        let app = build_router(state.clone());

        for index in 0..3 {
            let event = json!({
                "event_id": format!("evt-{index}"),
                "session_id": "session-2",
                "project_name": "project-b",
                "event_type": "tool.call",
                "payload": {"value": index},
                "created_at_ms": 2000 + index
            });

            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method(Method::POST)
                        .uri("/events")
                        .header("content-type", "application/json")
                        .body(Body::from(event.to_string()))
                        .expect("post request should build"),
                )
                .await
                .expect("event post should complete");
            assert_eq!(response.status(), StatusCode::OK);
        }

        wait_for_event_count(&state, "session-2", 3).await;

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/events?session_id=session-2&page=1&page_size=2")
                    .body(Body::empty())
                    .expect("query request should build"),
            )
            .await
            .expect("query request should complete");
        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("query response body should be readable")
            .to_bytes();
        let json: Value = serde_json::from_slice(&body).expect("query response should be json");
        let items = json["items"]
            .as_array()
            .expect("items should be represented as array");
        assert_eq!(items.len(), 2);
        assert_eq!(json["total"], 3);
    }

    #[tokio::test]
    async fn event_query_hides_raw_stdin_from_payload() {
        let state = AppState::new_in_memory().expect("in-memory sqlite init should succeed");
        let app = build_router(state.clone());

        let event = json!({
            "event_id": "evt-raw-stdin",
            "session_id": "session-raw-stdin",
            "project_name": "project-raw",
            "event_type": "tool.call",
            "payload": {
                "raw_stdin": "{\"secret\":\"value\"}",
                "stdin_json": {"safe": true},
                "tool_name": "shell"
            },
            "created_at_ms": 4000
        });

        let post_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/events")
                    .header("content-type", "application/json")
                    .body(Body::from(event.to_string()))
                    .expect("post request should build"),
            )
            .await
            .expect("event post should complete");
        assert_eq!(post_response.status(), StatusCode::OK);

        wait_for_event_persisted(&state, "evt-raw-stdin").await;

        let query_response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/events?session_id=session-raw-stdin&page=1&page_size=20")
                    .body(Body::empty())
                    .expect("query request should build"),
            )
            .await
            .expect("query request should complete");
        assert_eq!(query_response.status(), StatusCode::OK);

        let body = query_response
            .into_body()
            .collect()
            .await
            .expect("query response body should be readable")
            .to_bytes();
        let json: Value = serde_json::from_slice(&body).expect("query response should be json");
        let item_payload = &json["items"][0]["payload"];
        assert!(
            item_payload.get("raw_stdin").is_none(),
            "raw_stdin should be removed from /events response payload"
        );
        assert_eq!(item_payload["stdin_json"]["safe"], true);
    }

    #[tokio::test]
    async fn event_payload_validation_rejects_empty_required_fields() {
        let state = AppState::new_in_memory().expect("in-memory sqlite init should succeed");
        let app = build_router(state);

        let invalid_event = json!({
            "event_id": "",
            "session_id": "session-3",
            "project_name": "project-c",
            "event_type": "tool.call",
            "payload": {},
            "created_at_ms": 3000
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/events")
                    .header("content-type", "application/json")
                    .body(Body::from(invalid_event.to_string()))
                    .expect("invalid request should build"),
            )
            .await
            .expect("invalid request should complete");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn archive_session_hides_old_events_and_keeps_new_events_visible() {
        let state = AppState::new_in_memory().expect("in-memory sqlite init should succeed");
        let app = build_router(state.clone());

        let old_event_1 = json!({
            "event_id": "evt-archive-old-1",
            "session_id": "session-archive",
            "project_name": "project-archive",
            "event_type": "tool.call",
            "payload": {"value": 1},
            "created_at_ms": 5000
        });
        let old_event_2 = json!({
            "event_id": "evt-archive-old-2",
            "session_id": "session-archive",
            "project_name": "project-archive",
            "event_type": "tool.call",
            "payload": {"value": 2},
            "created_at_ms": 5001
        });

        for event in [old_event_1, old_event_2] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method(Method::POST)
                        .uri("/events")
                        .header("content-type", "application/json")
                        .body(Body::from(event.to_string()))
                        .expect("post request should build"),
                )
                .await
                .expect("event post should complete");
            assert_eq!(response.status(), StatusCode::OK);
        }

        wait_for_event_count(&state, "session-archive", 2).await;

        let archive_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/sessions/archive")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({ "session_id": "session-archive" }).to_string(),
                    ))
                    .expect("archive request should build"),
            )
            .await
            .expect("archive request should complete");
        assert_eq!(archive_response.status(), StatusCode::OK);

        let events_after_archive = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/events?session_id=session-archive&page=1&page_size=20")
                    .body(Body::empty())
                    .expect("events query request should build"),
            )
            .await
            .expect("events query should complete");
        assert_eq!(events_after_archive.status(), StatusCode::OK);
        let events_after_archive_json: Value = serde_json::from_slice(
            &events_after_archive
                .into_body()
                .collect()
                .await
                .expect("events query body should be readable")
                .to_bytes(),
        )
        .expect("events query response should be json");
        assert_eq!(
            events_after_archive_json["total"], 0,
            "archived session should hide old events from /events"
        );

        let sessions_after_archive = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/sessions")
                    .body(Body::empty())
                    .expect("sessions query request should build"),
            )
            .await
            .expect("sessions query should complete");
        assert_eq!(sessions_after_archive.status(), StatusCode::OK);
        let sessions_after_archive_json: Value = serde_json::from_slice(
            &sessions_after_archive
                .into_body()
                .collect()
                .await
                .expect("sessions query body should be readable")
                .to_bytes(),
        )
        .expect("sessions query response should be json");
        assert!(
            sessions_after_archive_json
                .as_array()
                .expect("sessions response should be an array")
                .is_empty(),
            "session should be hidden when all its events are archived"
        );

        let new_event = json!({
            "event_id": "evt-archive-new-1",
            "session_id": "session-archive",
            "project_name": "project-archive",
            "event_type": "tool.call",
            "payload": {"value": 3},
            "created_at_ms": 7000
        });
        let new_event_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/events")
                    .header("content-type", "application/json")
                    .body(Body::from(new_event.to_string()))
                    .expect("post request should build"),
            )
            .await
            .expect("new event post should complete");
        assert_eq!(new_event_response.status(), StatusCode::OK);

        wait_for_event_persisted(&state, "evt-archive-new-1").await;

        let events_after_new_message = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/events?session_id=session-archive&page=1&page_size=20")
                    .body(Body::empty())
                    .expect("events query request should build"),
            )
            .await
            .expect("events query should complete");
        assert_eq!(events_after_new_message.status(), StatusCode::OK);
        let events_after_new_message_json: Value = serde_json::from_slice(
            &events_after_new_message
                .into_body()
                .collect()
                .await
                .expect("events query body should be readable")
                .to_bytes(),
        )
        .expect("events query response should be json");
        assert_eq!(events_after_new_message_json["total"], 1);
        assert_eq!(
            events_after_new_message_json["items"][0]["event_id"],
            "evt-archive-new-1"
        );

        let sessions_after_new_message = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/sessions")
                    .body(Body::empty())
                    .expect("sessions query request should build"),
            )
            .await
            .expect("sessions query should complete");
        assert_eq!(sessions_after_new_message.status(), StatusCode::OK);
        let sessions_after_new_message_json: Value = serde_json::from_slice(
            &sessions_after_new_message
                .into_body()
                .collect()
                .await
                .expect("sessions query body should be readable")
                .to_bytes(),
        )
        .expect("sessions query response should be json");
        assert_eq!(
            sessions_after_new_message_json
                .as_array()
                .expect("sessions response should be an array")
                .len(),
            1,
            "session should reappear after a new unarchived event is inserted"
        );
    }

    #[tokio::test]
    async fn evaluation_honors_switch_and_sampling() {
        let state = AppState::new_in_memory().expect("in-memory sqlite init should succeed");
        let app = build_router(state.clone());

        let disable = EvalConfig {
            enabled: false,
            sampling_rate: 1,
            provider: "openai".to_string(),
            model: "gpt-4o-mini".to_string(),
            base_url: "http://127.0.0.1:18081".to_string(),
            api_key: Some("test-key".to_string()),
            timeout_ms: 1_500,
        };
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/settings")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_string(&disable).expect("settings json should serialize"),
                    ))
                    .expect("settings request should build"),
            )
            .await
            .expect("settings request should complete");
        assert_eq!(response.status(), StatusCode::OK);

        post_test_event(&app, "evt-switch-off", "session-switch", json!({"cmd":"ls"})).await;
        wait_for_event_persisted(&state, "evt-switch-off").await;
        assert_eq!(count_evaluations(&state, "session-switch"), 0);

        let enable = EvalConfig {
            enabled: true,
            sampling_rate: 2,
            provider: "openai".to_string(),
            model: "gpt-4o-mini".to_string(),
            base_url: "http://127.0.0.1:18081".to_string(),
            api_key: Some("test-key".to_string()),
            timeout_ms: 1_500,
        };
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/settings")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_string(&enable).expect("settings json should serialize"),
                    ))
                    .expect("settings request should build"),
            )
            .await
            .expect("settings request should complete");
        assert_eq!(response.status(), StatusCode::OK);

        post_test_event(&app, "evt-sampling-1", "session-switch", json!({"cmd":"pwd"})).await;
        wait_for_event_persisted(&state, "evt-sampling-1").await;
        assert_eq!(count_evaluations(&state, "session-switch"), 0);

        post_test_event(&app, "evt-sampling-2", "session-switch", json!({"cmd":"whoami"})).await;
        wait_for_eval_count(&state, "session-switch", 1).await;
        assert!(count_evaluations(&state, "session-switch") >= 1);
    }

    #[tokio::test]
    async fn evaluation_failure_records_and_retry_succeeds() {
        let state = AppState::new_in_memory().expect("in-memory sqlite init should succeed");
        let app = build_router(state.clone());

        post_test_event(
            &app,
            "evt-retry-1",
            "session-retry",
            json!({"force_error_once": true}),
        )
        .await;

        wait_for_eval_status(&state, "session-retry", "failed").await;
        let evaluation_id =
            latest_evaluation_id(&state, "session-retry").expect("failed evaluation should exist");

        let retry_request = json!({ "evaluation_id": evaluation_id });
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/evaluations/retry")
                    .header("content-type", "application/json")
                    .body(Body::from(retry_request.to_string()))
                    .expect("retry request should build"),
            )
            .await
            .expect("retry request should complete");
        assert_eq!(response.status(), StatusCode::OK);

        wait_for_eval_status(&state, "session-retry", "success").await;
        assert_eq!(
            latest_retry_count(&state, "session-retry").unwrap_or(0),
            1,
            "retry count should increment to 1 after successful retry"
        );
    }

    #[tokio::test]
    async fn settings_supports_provider_connection_parameters() {
        let state = AppState::new_in_memory().expect("in-memory sqlite init should succeed");
        let app = build_router(state);

        let config = EvalConfig {
            enabled: true,
            sampling_rate: 3,
            provider: "anthropic".to_string(),
            model: "claude-3-7-sonnet".to_string(),
            base_url: "https://api.anthropic.com/v1".to_string(),
            api_key: Some("sk-test-key".to_string()),
            timeout_ms: 3_000,
        };

        let save_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/settings")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_string(&config).expect("settings payload should serialize"),
                    ))
                    .expect("settings save request should build"),
            )
            .await
            .expect("settings save request should complete");
        assert_eq!(save_response.status(), StatusCode::OK);

        let load_response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/settings")
                    .body(Body::empty())
                    .expect("settings load request should build"),
            )
            .await
            .expect("settings load request should complete");
        assert_eq!(load_response.status(), StatusCode::OK);

        let body = load_response
            .into_body()
            .collect()
            .await
            .expect("settings body should be readable")
            .to_bytes();
        let loaded: EvalConfig =
            serde_json::from_slice(&body).expect("settings response should parse");
        assert_eq!(loaded.provider, "anthropic");
        assert_eq!(loaded.base_url, "https://api.anthropic.com/v1");
        assert_eq!(loaded.timeout_ms, 3_000);
        assert_eq!(loaded.api_key.as_deref(), Some("sk-test-key"));
    }

    #[tokio::test]
    async fn transcript_sync_imports_and_appends_incrementally() {
        let state = AppState::new_in_memory().expect("in-memory sqlite init should succeed");
        let app = build_router(state.clone());
        let transcript_path = temp_transcript_path("append");

        fs::write(&transcript_path, "{\"msg\":\"one\"}\n")
            .expect("should write initial transcript content");
        post_test_event(
            &app,
            "evt-transcript-append-1",
            "session-transcript-append",
            json!({
                "stdin_json": {
                    "transcript_path": transcript_path.to_string_lossy()
                }
            }),
        )
        .await;

        wait_for_event_persisted(&state, "evt-transcript-append-1").await;

        let first = transcript_state(&state, "session-transcript-append")
            .expect("transcript state should exist after first event");
        assert_eq!(first.0, "{\"msg\":\"one\"}\n");

        let mut file = OpenOptions::new()
            .append(true)
            .open(&transcript_path)
            .expect("should reopen transcript for appending");
        file.write_all(b"{\"msg\":\"two\"}\n")
            .expect("should append transcript content");

        post_test_event(
            &app,
            "evt-transcript-append-2",
            "session-transcript-append",
            json!({
                "stdin_json": {
                    "transcript_path": transcript_path.to_string_lossy()
                }
            }),
        )
        .await;

        wait_for_event_persisted(&state, "evt-transcript-append-2").await;

        let second = transcript_state(&state, "session-transcript-append")
            .expect("transcript state should exist after second event");
        assert_eq!(second.0, "{\"msg\":\"one\"}\n{\"msg\":\"two\"}\n");
        assert!(second.1 > first.1, "offset should move forward after append");

        post_test_event(
            &app,
            "evt-transcript-append-3",
            "session-transcript-append",
            json!({
                "stdin_json": {
                    "transcript_path": transcript_path.to_string_lossy()
                }
            }),
        )
        .await;

        wait_for_event_persisted(&state, "evt-transcript-append-3").await;

        let third = transcript_state(&state, "session-transcript-append")
            .expect("transcript state should still exist");
        assert_eq!(
            third.0, second.0,
            "no new file bytes should keep transcript content unchanged"
        );

        let _ = fs::remove_file(transcript_path);
    }

    #[tokio::test]
    async fn transcript_sync_merges_pending_fragment_before_inserting_lines() {
        let state = AppState::new_in_memory().expect("in-memory sqlite init should succeed");
        let app = build_router(state.clone());
        let transcript_path = temp_transcript_path("pending-fragment");

        fs::write(&transcript_path, "{\"msg\":\"first\"}")
            .expect("should write transcript without trailing newline");
        post_test_event(
            &app,
            "evt-transcript-fragment-1",
            "session-transcript-fragment",
            json!({
                "stdin_json": {
                    "transcript_path": transcript_path.to_string_lossy()
                }
            }),
        )
        .await;

        wait_for_event_persisted(&state, "evt-transcript-fragment-1").await;

        let first = transcript_state(&state, "session-transcript-fragment")
            .expect("transcript state should exist after first event");
        assert_eq!(
            first.0, "",
            "incomplete line should not be inserted before newline arrives"
        );

        let mut file = OpenOptions::new()
            .append(true)
            .open(&transcript_path)
            .expect("should reopen transcript for appending");
        file.write_all(b"\n{\"msg\":\"second\"}\n")
            .expect("should append completed lines");

        post_test_event(
            &app,
            "evt-transcript-fragment-2",
            "session-transcript-fragment",
            json!({
                "stdin_json": {
                    "transcript_path": transcript_path.to_string_lossy()
                }
            }),
        )
        .await;

        wait_for_event_persisted(&state, "evt-transcript-fragment-2").await;

        let second = transcript_state(&state, "session-transcript-fragment")
            .expect("transcript state should exist after second event");
        assert_eq!(second.0, "{\"msg\":\"first\"}\n{\"msg\":\"second\"}\n");
        assert!(second.1 > first.1, "offset should move forward after append");

        let _ = fs::remove_file(transcript_path);
    }

    #[tokio::test]
    async fn transcript_sync_resets_when_file_truncated() {
        let state = AppState::new_in_memory().expect("in-memory sqlite init should succeed");
        let app = build_router(state.clone());
        let transcript_path = temp_transcript_path("truncate");

        fs::write(&transcript_path, "{\"msg\":\"before\"}\n{\"msg\":\"keep\"}\n")
            .expect("should write initial transcript content");
        post_test_event(
            &app,
            "evt-transcript-truncate-1",
            "session-transcript-truncate",
            json!({
                "stdin_json": {
                    "transcript_path": transcript_path.to_string_lossy()
                }
            }),
        )
        .await;

        wait_for_event_persisted(&state, "evt-transcript-truncate-1").await;

        fs::write(&transcript_path, "{\"msg\":\"after\"}\n")
            .expect("should truncate and rewrite transcript content");
        post_test_event(
            &app,
            "evt-transcript-truncate-2",
            "session-transcript-truncate",
            json!({
                "stdin_json": {
                    "transcript_path": transcript_path.to_string_lossy()
                }
            }),
        )
        .await;

        wait_for_event_persisted(&state, "evt-transcript-truncate-2").await;

        let state_after = transcript_state(&state, "session-transcript-truncate")
            .expect("transcript state should exist after truncate event");
        assert_eq!(
            state_after.0, "{\"msg\":\"after\"}\n",
            "truncate should reset and replace persisted transcript content"
        );
        let _ = fs::remove_file(transcript_path);
    }

    #[tokio::test]
    async fn transcript_sync_records_error_for_invalid_path() {
        let state = AppState::new_in_memory().expect("in-memory sqlite init should succeed");
        let app = build_router(state.clone());
        let transcript_path = format!(
            "{}/missing-{}.jsonl",
            std::env::temp_dir().to_string_lossy(),
            Uuid::new_v4()
        );

        post_test_event(
            &app,
            "evt-transcript-error-1",
            "session-transcript-error",
            json!({
                "stdin_json": {
                    "transcript_path": transcript_path
                }
            }),
        )
        .await;

        wait_for_event_persisted(&state, "evt-transcript-error-1").await;

        let error = transcript_error(&state, "session-transcript-error")
            .expect("transcript sync error should be recorded");
        assert!(
            error.0.contains("No such file")
                || error.0.contains("not found")
                || error.0.contains("cannot find")
                || error.0.contains("os error 2"),
            "error message should include file-not-found details, got={}",
            error.0
        );
        assert!(
            !error.1.is_empty(),
            "error stack/debug detail should be persisted"
        );
    }

    #[tokio::test]
    async fn transcript_query_returns_content_for_existing_session() {
        let state = AppState::new_in_memory().expect("in-memory sqlite init should succeed");
        let app = build_router(state.clone());
        let transcript_path = temp_transcript_path("query-existing");

        fs::write(&transcript_path, "{\"msg\":\"query\"}\n")
            .expect("should write transcript content for query test");
        post_test_event(
            &app,
            "evt-transcript-query-1",
            "session-transcript-query",
            json!({
                "stdin_json": {
                    "transcript_path": transcript_path.to_string_lossy()
                }
            }),
        )
        .await;

        wait_for_event_persisted(&state, "evt-transcript-query-1").await;

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/transcripts?session_id=session-transcript-query")
                    .body(Body::empty())
                    .expect("transcript query request should build"),
            )
            .await
            .expect("transcript query request should complete");
        assert_eq!(response.status(), StatusCode::OK);
        let body = response
            .into_body()
            .collect()
            .await
            .expect("transcript query body should be readable")
            .to_bytes();
        let json: Value =
            serde_json::from_slice(&body).expect("transcript query response should be json");
        assert_eq!(json["session_id"], "session-transcript-query");
        assert_eq!(json["items"].as_array().map(Vec::len), Some(1));
        assert_eq!(json["items"][0]["line_no"], 1);
        assert_eq!(json["items"][0]["line_content"], "{\"msg\":\"query\"}\n");
        assert_eq!(json["has_more"], false);
        assert_eq!(json["next_before_line_no"], Value::Null);
        assert_eq!(json["imported_offset_bytes"], "{\"msg\":\"query\"}\n".len() as i64);

        let _ = fs::remove_file(transcript_path);
    }

    #[tokio::test]
    async fn transcript_query_returns_empty_payload_when_not_found() {
        let state = AppState::new_in_memory().expect("in-memory sqlite init should succeed");
        let app = build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/transcripts?session_id=session-without-transcript")
                    .body(Body::empty())
                    .expect("transcript query request should build"),
            )
            .await
            .expect("transcript query request should complete");
        assert_eq!(response.status(), StatusCode::OK);
        let body = response
            .into_body()
            .collect()
            .await
            .expect("transcript query body should be readable")
            .to_bytes();
        let json: Value =
            serde_json::from_slice(&body).expect("transcript query response should be json");
        assert_eq!(json["items"].as_array().map(Vec::len), Some(0));
        assert_eq!(json["has_more"], false);
        assert_eq!(json["next_before_line_no"], Value::Null);
        assert_eq!(json["updated_at_ms"], 0);
        assert_eq!(json["imported_offset_bytes"], 0);
    }

    #[tokio::test]
    async fn transcript_query_supports_cursor_pagination_for_older_lines() {
        let state = AppState::new_in_memory().expect("in-memory sqlite init should succeed");
        let app = build_router(state.clone());
        let transcript_path = temp_transcript_path("query-pagination");

        fs::write(
            &transcript_path,
            "{\"msg\":\"one\"}\n{\"msg\":\"two\"}\n{\"msg\":\"three\"}\n",
        )
        .expect("should write transcript content for pagination test");
        post_test_event(
            &app,
            "evt-transcript-query-page-1",
            "session-transcript-query-page",
            json!({
                "stdin_json": {
                    "transcript_path": transcript_path.to_string_lossy()
                }
            }),
        )
        .await;

        wait_for_event_persisted(&state, "evt-transcript-query-page-1").await;

        let first_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/transcripts?session_id=session-transcript-query-page&page_size=2")
                    .body(Body::empty())
                    .expect("first transcript query should build"),
            )
            .await
            .expect("first transcript query should complete");
        assert_eq!(first_response.status(), StatusCode::OK);
        let first_body = first_response
            .into_body()
            .collect()
            .await
            .expect("first transcript query body should be readable")
            .to_bytes();
        let first_json: Value =
            serde_json::from_slice(&first_body).expect("first transcript query response should be json");
        assert_eq!(first_json["items"].as_array().map(Vec::len), Some(2));
        assert_eq!(first_json["items"][0]["line_no"], 2);
        assert_eq!(first_json["items"][1]["line_no"], 3);
        assert_eq!(first_json["has_more"], true);
        assert_eq!(first_json["next_before_line_no"], 2);

        let second_response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(
                        "/transcripts?session_id=session-transcript-query-page&page_size=2&before_line_no=2",
                    )
                    .body(Body::empty())
                    .expect("second transcript query should build"),
            )
            .await
            .expect("second transcript query should complete");
        assert_eq!(second_response.status(), StatusCode::OK);
        let second_body = second_response
            .into_body()
            .collect()
            .await
            .expect("second transcript query body should be readable")
            .to_bytes();
        let second_json: Value = serde_json::from_slice(&second_body)
            .expect("second transcript query response should be json");
        assert_eq!(second_json["items"].as_array().map(Vec::len), Some(1));
        assert_eq!(second_json["items"][0]["line_no"], 1);
        assert_eq!(second_json["has_more"], false);
        assert_eq!(second_json["next_before_line_no"], Value::Null);

        let _ = fs::remove_file(transcript_path);
    }

    #[tokio::test]
    async fn transcript_query_rejects_empty_session_id() {
        let state = AppState::new_in_memory().expect("in-memory sqlite init should succeed");
        let app = build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/transcripts?session_id=")
                    .body(Body::empty())
                    .expect("transcript query request should build"),
            )
            .await
            .expect("transcript query request should complete");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = response
            .into_body()
            .collect()
            .await
            .expect("bad request body should be readable")
            .to_bytes();
        let json: Value = serde_json::from_slice(&body).expect("bad request response should be json");
        assert_eq!(json["error_code"], "bad_request");
        assert_eq!(json["retryable"], false);
    }

    #[tokio::test]
    async fn post_event_returns_429_when_event_queue_is_full() {
        let db = Connection::open_in_memory().expect("in-memory sqlite init should succeed");
        db::init_schema(&db).expect("schema should initialize");
        let db = Arc::new(Mutex::new(db));
        let (eval_tx, _eval_rx) = mpsc::channel(128);
        let (event_tx, _event_rx) = mpsc::channel::<IncomingEvent>(1);
        let (transcript_register_tx, _transcript_register_rx) =
            mpsc::channel::<transcript::TranscriptRegisterRequest>(16);

        let state = AppState {
            db,
            eval_tx,
            event_tx,
            eval_counter: Arc::new(AtomicU64::new(0)),
            eval_config_cache: Arc::new(RwLock::new(EvalConfig::default())),
            transcript_register_tx,
        };
        let app = build_router(state);

        post_test_event(&app, "evt-queue-full-1", "session-queue-full", json!({})).await;

        let second_event = json!({
            "event_id": "evt-queue-full-2",
            "session_id": "session-queue-full",
            "project_name": "test-project",
            "event_type": "tool.call",
            "payload": {},
            "created_at_ms": now_timestamp_ms()
        });
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/events")
                    .header("content-type", "application/json")
                    .body(Body::from(second_event.to_string()))
                    .expect("post request should build"),
            )
            .await
            .expect("post request should complete");
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        let body = response
            .into_body()
            .collect()
            .await
            .expect("queue full response body should be readable")
            .to_bytes();
        let json: Value = serde_json::from_slice(&body).expect("queue full response should be json");
        assert_eq!(json["error_code"], "evaluation_queue_full");
        assert_eq!(json["retryable"], true);
    }

    async fn wait_for_event_persisted(state: &AppState, event_id: &str) {
        for _ in 0..80 {
            let found = {
                let db = state.db.lock().expect("db lock should succeed");
                db.query_row(
                    "SELECT COUNT(1) FROM events WHERE event_id = ?1",
                    params![event_id],
                    |row| row.get::<_, i64>(0),
                )
                .unwrap_or(0)
            };
            if found > 0 {
                return;
            }
            sleep(Duration::from_millis(25)).await;
        }
    }

    async fn wait_for_event_count(state: &AppState, session_id: &str, expected_min: u64) {
        for _ in 0..80 {
            let count = {
                let db = state.db.lock().expect("db lock should succeed");
                db.query_row(
                    "SELECT COUNT(1) FROM events WHERE session_id = ?1",
                    params![session_id],
                    |row| row.get::<_, i64>(0),
                )
                .unwrap_or(0) as u64
            };
            if count >= expected_min {
                return;
            }
            sleep(Duration::from_millis(25)).await;
        }
    }

    async fn wait_for_stable_event_count(state: &AppState, session_id: &str, expected: u64) {
        for _ in 0..80 {
            let count = {
                let db = state.db.lock().expect("db lock should succeed");
                db.query_row(
                    "SELECT COUNT(1) FROM events WHERE session_id = ?1",
                    params![session_id],
                    |row| row.get::<_, i64>(0),
                )
                .unwrap_or(0) as u64
            };
            if count == expected {
                return;
            }
            sleep(Duration::from_millis(25)).await;
        }
    }

    async fn post_test_event(app: &Router, event_id: &str, session_id: &str, payload: Value) {
        let event = json!({
            "event_id": event_id,
            "session_id": session_id,
            "project_name": "test-project",
            "event_type": "tool.call",
            "payload": payload,
            "created_at_ms": now_timestamp_ms()
        });
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/events")
                    .header("content-type", "application/json")
                    .body(Body::from(event.to_string()))
                    .expect("post request should build"),
            )
            .await
            .expect("post request should complete");
        assert_eq!(response.status(), StatusCode::OK);
    }

    async fn wait_for_eval_count(state: &AppState, session_id: &str, expected_min: u64) {
        for _ in 0..40 {
            if count_evaluations(state, session_id) >= expected_min {
                return;
            }
            sleep(Duration::from_millis(50)).await;
        }
    }

    async fn wait_for_eval_status(state: &AppState, session_id: &str, status: &str) {
        for _ in 0..40 {
            if latest_eval_status(state, session_id).as_deref() == Some(status) {
                return;
            }
            sleep(Duration::from_millis(50)).await;
        }
    }

    fn count_evaluations(state: &AppState, session_id: &str) -> u64 {
        let db = state
            .db
            .lock()
            .expect("sqlite lock should be available for count");
        db.query_row(
            "SELECT COUNT(1) FROM evaluations WHERE session_id = ?1",
            params![session_id],
            |row| row.get::<_, i64>(0),
        )
        .expect("count query should succeed") as u64
    }

    fn latest_evaluation_id(state: &AppState, session_id: &str) -> Option<String> {
        let db = state
            .db
            .lock()
            .expect("sqlite lock should be available for latest id");
        db.query_row(
            "SELECT evaluation_id FROM evaluations WHERE session_id = ?1 ORDER BY created_at_ms DESC LIMIT 1",
            params![session_id],
            |row| row.get(0),
        )
        .ok()
    }

    fn latest_eval_status(state: &AppState, session_id: &str) -> Option<String> {
        let db = state
            .db
            .lock()
            .expect("sqlite lock should be available for status");
        db.query_row(
            "SELECT status FROM evaluations WHERE session_id = ?1 ORDER BY created_at_ms DESC LIMIT 1",
            params![session_id],
            |row| row.get(0),
        )
        .ok()
    }

    fn latest_retry_count(state: &AppState, session_id: &str) -> Option<u32> {
        let db = state
            .db
            .lock()
            .expect("sqlite lock should be available for retry count");
        db.query_row(
            "SELECT retry_count FROM evaluations WHERE session_id = ?1 ORDER BY created_at_ms DESC LIMIT 1",
            params![session_id],
            |row| row.get::<_, i64>(0),
        )
        .ok()
        .map(|value| value as u32)
    }

    fn temp_transcript_path(suffix: &str) -> PathBuf {
        let base = dirs::home_dir()
            .map(|h| h.join(".cache").join("claude-code-launch-test"))
            .unwrap_or_else(|| std::env::temp_dir().join("claude-code-launch-test"));
        fs::create_dir_all(&base).expect("should create test temp dir under home");
        base.join(format!("transcript-{suffix}-{}.jsonl", Uuid::new_v4()))
    }

    fn transcript_state(state: &AppState, session_id: &str) -> Option<(String, i64)> {
        let db = state
            .db
            .lock()
            .expect("sqlite lock should be available for transcript state");
        let offset = db
            .query_row(
                "SELECT imported_offset_bytes FROM session_transcripts WHERE session_id = ?1",
                params![session_id],
                |row| row.get::<_, i64>(0),
            )
            .ok()?;
        let mut statement = db
            .prepare(
                "
                SELECT line_content
                FROM session_transcript_lines
                WHERE session_id = ?1
                ORDER BY line_no ASC
                ",
            )
            .ok()?;
        let rows = statement
            .query_map(params![session_id], |row| row.get::<_, String>(0))
            .ok()?;
        let mut content = String::new();
        for row in rows {
            content.push_str(&row.ok()?);
        }
        Some((content, offset))
    }

    fn transcript_error(state: &AppState, session_id: &str) -> Option<(String, String)> {
        let db = state
            .db
            .lock()
            .expect("sqlite lock should be available for transcript error");
        db.query_row(
            "SELECT last_error_message, last_error_stack FROM session_transcripts WHERE session_id = ?1",
            params![session_id],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?.unwrap_or_default(),
                    row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                ))
            },
        )
        .ok()
    }
}
