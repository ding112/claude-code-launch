# Architecture

**Analysis Date:** 2026-03-18

## Pattern Overview

**Overall:** Tauri 2 Desktop App with Embedded HTTP Server (Hybrid IPC + REST)

**Key Characteristics:**
- Dual communication: Tauri `invoke()` for setup wizard, REST API for monitoring/data
- Embedded Axum HTTP server on localhost:8787
- SQLite as single data store
- Background workers for event processing, evaluation queue, transcript polling
- Standalone `local_api` binary for headless mode

## Layers

**Presentation Layer (Frontend):**
- Purpose: React UI for setup wizard, session monitoring, settings
- Contains: Pages, components, hooks, API client
- Location: `src/pages/`, `src/components/`, `src/hooks/`, `src/api.ts`
- Depends on: Local HTTP API (REST), Tauri IPC (setup commands)
- Used by: End user

**API Client Layer:**
- Purpose: Typed fetch wrappers for backend REST endpoints
- Contains: `fetchSessions`, `fetchEvents`, `fetchEvaluations`, `fetchTranscript`, etc.
- Location: `src/api.ts`
- Depends on: Backend HTTP API
- Used by: React hooks

**Tauri Command Layer:**
- Purpose: Native OS operations (prereq checks, npm install, verification)
- Contains: Tauri `invoke()` commands for setup wizard
- Location: `src-tauri/src/commands/mod.rs`
- Depends on: Services layer
- Used by: Frontend setup hooks (`usePrereqs`, `useInstall`, `useVerify`)

**HTTP Server Layer:**
- Purpose: REST API endpoints for monitoring data
- Contains: Axum route handlers for sessions, events, evaluations, transcripts, settings, hooks
- Location: `src-tauri/src/collection/handlers.rs`, `src-tauri/src/collection.rs`
- Depends on: Data access layer, AppState
- Used by: Frontend API client, external agents

**Data Access Layer:**
- Purpose: SQLite database operations
- Contains: Schema creation, CRUD operations, queries
- Location: `src-tauri/src/collection/db.rs`, `src-tauri/src/dao/mod.rs`
- Depends on: rusqlite, SQLite
- Used by: HTTP handlers, background workers

**Services Layer:**
- Purpose: Business logic for setup, installation, verification
- Contains: Prereq checking, npm installation, Node.js download, verification
- Location: `src-tauri/src/services/`
- Depends on: OS filesystem, network
- Used by: Tauri commands

**Background Workers:**
- Purpose: Async processing of events, evaluations, transcript polling
- Contains: Event worker, eval queue, transcript poller, session discovery
- Location: `src-tauri/src/collection/eval_queue.rs`, `src-tauri/src/collection/transcript_poller.rs`, `src-tauri/src/collection/discovery.rs`
- Depends on: Data access layer, external APIs
- Used by: Spawned at app startup

## Data Flow

**App Bootstrap:**
1. `main.rs` → Tauri app builder
2. `lib.rs` → registers Tauri commands, sets up app state
3. Spawns background workers (event, eval queue, transcript poller, discovery)
4. Starts Axum HTTP server on port 8787
5. Frontend loads, checks setup status

**Setup Flow (Tauri IPC):**
1. Frontend `SetupPage` calls `usePrereqs` → Tauri `invoke("check_prereqs")`
2. `commands/mod.rs` → `services/prereq_service.rs` checks npm, Claude Code
3. If missing: `useInstall` → `invoke("install_deps")` → `services/install_service.rs`
4. `useVerify` → `invoke("verify_install")` → `services/verify_service.rs`
5. Setup complete → navigate to monitoring

**Monitoring Flow (REST):**
1. `SessionsPage` → `useSessions` → `fetchSessions()` → GET `/sessions`
2. `handlers.rs` → queries SQLite → returns JSON
3. Select session → `useTranscript` → `fetchTranscript()` → GET `/transcripts/{id}`
4. Events tab → `useEvents` → `fetchEvents()` → GET `/events`
5. Evaluations → `useEvaluations` → `fetchEvaluations()` → GET `/evaluations`

**Event Ingestion:**
1. External agent POSTs to `/events`
2. `handlers.rs` validates and sanitizes payload
3. Stores in SQLite events table
4. Background eval queue picks up for LLM evaluation (if configured)

**State Management:**
- Frontend: React hooks with `useState`/`useEffect`, no global store
- Backend: `AppState` with `Arc<Mutex<Connection>>` for SQLite access
- Persistent state: SQLite database
- Config: `~/.config/claude-code-launch/config.json`

## Key Abstractions

**AppState:**
- Purpose: Shared application state across all Axum handlers
- Location: `src-tauri/src/collection.rs`
- Pattern: `Arc<Mutex<Connection>>` wrapped in Axum state extractor

**Evaluation Provider:**
- Purpose: Abstraction over LLM APIs (OpenAI, Anthropic, Ollama)
- Location: `src-tauri/src/evaluation.rs`
- Pattern: Provider enum with trait-like dispatch for chat completions

**Collection Module:**
- Purpose: Core monitoring data collection and serving
- Location: `src-tauri/src/collection/` (handlers, db, discovery, hooks, transcript, eval_queue)
- Pattern: Module with submodules for each concern

**React Hooks:**
- Purpose: Encapsulate API calls and state management per feature
- Location: `src/hooks/use*.ts`
- Pattern: Custom hooks returning `{ data, loading, error, load }` objects

## Entry Points

**Tauri Binary (`claude-code-launch`):**
- Location: `src-tauri/src/main.rs` → `src-tauri/src/lib.rs`
- Triggers: User launches desktop app
- Responsibilities: Initialize Tauri, register commands, start HTTP server, spawn workers

**Standalone API (`local_api`):**
- Location: `src-tauri/src/bin/local_api.rs`
- Triggers: Run as standalone binary without Tauri GUI
- Responsibilities: Start HTTP server only, no GUI

**Frontend Entry:**
- Location: `src/main.tsx` → `src/App.tsx`
- Triggers: Tauri webview loads
- Responsibilities: React root, routing between Setup and Sessions pages

## Error Handling

**Strategy:** Mixed — Rust uses `Result<T, E>` with custom `ApiError`; Frontend uses `assertOk` throw pattern

**Backend Patterns:**
- `ApiError` enum in handlers for HTTP error responses
- `Result<T, String>` for Tauri commands
- `spawn_blocking` with fallback for blocking operations
- `sanitize_json_value` for payload security

**Frontend Patterns:**
- `assertOk(response)` in `api.ts` — throws on non-2xx responses
- Hooks catch errors inconsistently (some try/catch, some unhandled)
- Error state via `useState` in hooks

## Cross-Cutting Concerns

**Logging:**
- Backend: `println!` / `eprintln!` (no structured logging framework)
- Frontend: Minimal `console.error` usage

**Validation:**
- Backend: Manual validation in handlers
- Frontend: Form validation in settings page

**CORS:**
- `tower-http` CORS middleware on Axum server
- Allows localhost origins, configurable via env var

**Serialization:**
- serde/serde_json for all Rust JSON handling
- Frontend uses native `fetch` + `response.json()`

---
*Architecture analysis: 2026-03-18*
*Update when major patterns change*
