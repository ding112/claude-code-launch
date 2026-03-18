# Codebase Concerns

**Analysis Date:** 2026-03-18

## Tech Debt

**Hooks JSON manipulation — `.unwrap()` panics:**
- Issue: `src-tauri/src/collection/hooks.rs` uses `.unwrap()` on `Value::as_array_mut()` and `get_mut("hooks")` (lines ~168, 174, 176, 178)
- Why: Assumes Cursor settings JSON has expected structure; shortcut to avoid verbose error handling
- Impact: Panic if user-edited settings file has malformed `hooks` structure or missing blocks
- Fix approach: Use `ok_or_else` / `map_err` and return `Result`, or validate structure before mutation

**Hardcoded Node.js version and mirror:**
- Issue: Hardcoded `NODE_MSI_URL` (v22.14.0) and `NPM_MIRROR` (npmmirror.com) in `src-tauri/src/services/node_install_service.rs`
- Why: Fixed version for reproducibility; Chinese mirror for faster installs
- Impact: Node.js version becomes outdated; mirror may change or become unavailable
- Fix approach: Make version configurable via app config; allow mirror override or fallback to official registry

**npm_cmd.unwrap() after install:**
- Issue: `src-tauri/src/services/node_install_service.rs` line ~72: `let npm = npm_cmd.unwrap()` after `install_nodejs_msi` succeeds
- Why: Assumes install always puts npm in PATH or default dir
- Impact: Panic if msiexec succeeds but PATH refresh fails or npm not in expected location
- Fix approach: Return `Err` from `ensure_npm_available` when `resolve_npm_cmd()` returns `None`

**Single SQLite connection with Mutex:**
- Issue: `AppState` uses `Arc<Mutex<Connection>>` for all DB access in `src-tauri/src/collection.rs`
- Why: `rusqlite::Connection` is `!Send`; simplest thread-safe approach
- Impact: All DB operations serialize through one lock; under heavy load, contention causes latency
- Fix approach: Consider connection pool (e.g. `r2d2` + `rusqlite`) or separate read/write connections

**Inconsistent error handling in frontend hooks:**
- Issue: `src/hooks/useEvalSettings.ts` and `src/hooks/useEvaluations.ts` call API functions without try/catch; `assertOk` throws on non-2xx
- Why: API layer throws; hooks were not designed to handle failures
- Impact: Unhandled rejection; pages show default/empty state with no user feedback when API is down
- Fix approach: Wrap in try/catch, set error state, show toast or inline message

## Known Bugs

**Settings page load failure:**
- Symptoms: Settings page shows default values (empty api_key) when backend is unavailable; no error message
- Trigger: Start app without backend; open Settings tab
- Root cause: `src/hooks/useEvalSettings.ts` `loadSettings` does not catch `fetchSettings` rejection

**Evaluations fetch failure:**
- Symptoms: Evaluations list empty or stale when API returns 4xx/5xx; no user feedback
- Trigger: Backend error or network issue while viewing session evaluations
- Root cause: `src/hooks/useEvaluations.ts` `loadEvaluations` does not catch `fetchEvaluations` rejection

**Events load — error only in console:**
- Symptoms: Events list empty; error only in console
- Trigger: `fetchEvents` fails (session not found, API error)
- Root cause: `src/hooks/useEvents.ts` `loadEvents` catches but only `console.error`; no user-visible feedback

## Security Considerations

**Eval API key storage:**
- Risk: API key stored in SQLite `settings` table; if DB file is accessible, key is exposed
- Current mitigation: DB typically in user home dir; app runs locally
- Recommendations: Consider encrypting sensitive fields at rest; document that DB should not be shared

**Event payload sanitization:**
- Risk: Event payloads may contain tokens, secrets, passwords
- Current mitigation: `sanitize_json_value` in `src-tauri/src/collection.rs` redacts keys matching `token`, `secret`, `password`, etc.
- Recommendations: Keep `is_sensitive_key` list updated; consider adding audit for new sensitive patterns

**CORS configuration:**
- Risk: Overly permissive CORS could allow malicious sites to call API
- Current mitigation: CORS allows `localhost:1420`, `127.0.0.1:1420`, `tauri://localhost`; env vars can override
- Recommendations: OK for local desktop app; if API ever exposed externally, restrict origins strictly

## Performance Bottlenecks

**SQLite connection contention:**
- Problem: All DB reads/writes go through single `Mutex<Connection>`
- Cause: `rusqlite::Connection` is not `Send`; shared state requires one connection
- Improvement path: Use `r2d2` + `rusqlite` for connection pool; or batch writes where possible

**Evaluation worker blocking:**
- Problem: Evaluation HTTP calls use `reqwest::blocking` in `src-tauri/src/evaluation.rs`
- Cause: Sync API for external LLM calls
- Improvement path: Already wrapped in `spawn_blocking` in `eval_queue.rs`; acceptable. Consider async reqwest if worker count grows

## Fragile Areas

**Hooks init/add logic (`src-tauri/src/collection/hooks.rs`):**
- Why fragile: Mutates nested JSON; assumes `blocks` is array, `hooks` exists; `.unwrap()` on `get_mut`
- Safe modification: Add validation before mutation; return `Result` instead of panicking

**Cursor path encoding (`src-tauri/src/collection/cursor_discovery.rs`):**
- Why fragile: `extract_project_name` relies on Cursor's internal path encoding; hardcoded markers (`IdeaProjects-`, `workspace-github-`, etc.)
- Safe modification: Add tests for known path formats; document assumptions; consider config override

**Transcript sync (`src-tauri/src/collection/transcript.rs`):**
- Why fragile: `read_transcript_increment` reads file by path; file may be locked, truncated, or moved by Cursor
- Safe modification: Handle `io::Error`; retry on transient errors

## Dependencies at Risk

**Vite 7 / React 19:**
- Risk: Cutting-edge versions; possible ecosystem compatibility issues
- Migration plan: Pin versions; monitor for regressions

**rusqlite bundled:**
- Risk: Bundled SQLite adds binary size; may lag upstream SQLite security fixes
- Migration plan: Monitor rusqlite releases; consider system SQLite if available

## Test Coverage Gaps

**Frontend hooks:**
- What's not tested: `useEvalSettings`, `useEvaluations`, `useEvents`, `useSessions`, `useHooksConfig`, `useTranscript`
- Risk: Regressions in error handling, loading states, or API integration
- Priority: Medium

**Frontend UI:**
- What's not tested: `SessionsPage`, `SettingsPage`, `SetupPage`; no unit or E2E tests
- Risk: Layout breakage, form validation, navigation
- Priority: Medium

**Full install flow:**
- What's not tested: Setup wizard end-to-end, Cursor discovery, transcript sync; covered by manual checklist only
- Risk: Manual testing may miss regressions
- Priority: High for critical paths

**Python hooks:**
- What's not tested: `hooks/report_event.py`, `hooks/init_hooks.py`; no automated tests
- Risk: JSON parsing, stdin handling, HTTP failures
- Priority: Low

---
*Concerns audit: 2026-03-18*
*Update as issues are fixed or new ones discovered*
