# Coding Conventions

**Analysis Date:** 2026-03-18

## Naming Patterns

**Files:**
- PascalCase for React components and pages (`App.tsx`, `SessionsPage.tsx`, `SetupPage.tsx`, `Pager.tsx`, `TabSwitcher.tsx`)
- camelCase for hooks (`useSessions.ts`, `useTranscript.ts`, `useEvents.ts`, `usePrereqs.ts`)
- lowercase for utilities and API (`api.ts`, `utils.ts`, `constants.ts`, `types.ts`)
- kebab-case for UI primitives in `components/ui/` (`button.tsx`, `card.tsx`, `collapsible.tsx`)
- snake_case for Rust modules (`eval_queue.rs`, `cursor_discovery.rs`, `node_install_service.rs`)

**Functions:**
- camelCase for all TypeScript functions (`fetchSessions`, `loadTranscript`, `formatTimestamp`, `riskClass`)
- `handle*` prefix for event handlers (`handleTranscriptScroll`, `handleTimeRangeChange`)
- No special prefix for async functions
- `assertOk`, `fetch*`, `save*`, `load*`, `run*` for API/action functions
- snake_case for Rust functions (`check_prereqs`, `install_deps`, `verify_install`)

**Variables:**
- camelCase for TypeScript variables (`selectedSessionId`, `transcriptLoading`, `groupedSessions`)
- UPPER_SNAKE_CASE for constants (`API_BASE`, `TRANSCRIPT_PAGE_SIZE`, `RISK_STYLES`, `KNOWN_EVENTS`)
- No underscore prefix for private members
- snake_case for Rust variables

**Types:**
- PascalCase for interfaces and type aliases (`SessionItem`, `EventResponse`, `PrereqResult`)
- No `I` prefix (`SessionItem`, not `ISessionItem`)
- snake_case for API response fields (`session_id`, `project_name`, `last_active_at_ms`)

## Code Style

**Formatting:**
- No Prettier, ESLint, or Biome config in project root
- TypeScript strict mode enabled (`tsconfig.json`: strict, noUnusedLocals, noUnusedParameters)
- Double quotes for strings
- Semicolons used
- 2 space indentation

**Linting:**
- No dedicated lint tool; TypeScript compiler provides strict checks
- No `npm run lint` or format scripts in `package.json`
- Rust: standard `cargo clippy` conventions

## Import Organization

**Order:**
1. React and external packages (`react`, `@tauri-apps/api`, `lucide-react`)
2. Internal modules via `@/` alias (`@/components/ui/button`, `@/lib/utils`)
3. Relative imports (`../types`, `../api`, `./utils`)
4. Type imports (`import type { ... }`) — often inline with regular imports

**Path Aliases:**
- `@/` maps to `src/` (tsconfig paths, vite resolve.alias)
- `components.json` defines: `@/components`, `@/lib/utils`, `@/components/ui`, `@/lib`, `@/hooks`

## Error Handling

**Frontend Patterns:**
- `assertOk` helper in `api.ts`: throws on non-ok Response, returns Response otherwise
- try/catch in hooks; set error state (`setTranscriptError`, `setSessionMessage`) for user feedback
- catch blocks sometimes empty or minimal; one instance uses `console.error` (`useEvents.ts`)
- Optional returns: `fetchHooks`, `initHooksApi` return null on non-ok; `saveSettingsApi`, `archiveSession` return boolean

**Backend Patterns:**
- `Result<T, E>` with custom `ApiError` enum for HTTP error responses
- `Result<T, String>` for Tauri commands
- `.unwrap()` in some places (hooks.rs JSON manipulation) — tech debt
- `spawn_blocking` for blocking operations

**Error Types:**
- Throw on API failures via `assertOk`
- Return null or false for expected failures (e.g., hooks not configured)
- User-facing messages in Chinese (e.g., "Transcript 加载失败，请稍后重试。")

## Logging

**Framework:**
- No dedicated logger; minimal use of console
- Single `console.error` in `useEvents.ts` for failed event load
- Rust backend: `println!` for startup info, `eprintln!` for errors

**Patterns:**
- Logging is sparse; errors surfaced via UI state
- No structured logging or log levels

## Comments

**When to Comment:**
- Section headers in `types.ts` (`// ── Setup (wizard) types ──`, `// ── Monitoring (overseer) types ──`)
- Vite config comments for Tauri-specific options
- `@ts-expect-error` with reason (`vite.config.ts`: process is a nodejs global)
- Minimal inline comments; code is self-documenting

## Function Design

**Size:**
- Functions generally under 50 lines; complex logic in `TranscriptView.tsx` `parseLines`
- Hooks return objects with many properties; logic split across `useMemo`, `useCallback`, `useEffect`

**Parameters:**
- Options objects for API calls with optional fields (`opts?: { page?, pageSize?, eventType?, fromMs?, toMs? }`)
- Destructuring in parameter list common (e.g., `Pager` props destructured)

## Module Design

**Exports:**
- Default exports for React components and pages (`App`, `SessionsPage`, `SetupPage`, `Pager`)
- Named exports for hooks (`useSessions`, `useTranscript`, `useEvents`)
- Named exports for API functions and utilities (`fetchSessions`, `formatTimestamp`, `riskClass`)
- Type exports from `types.ts`; re-exported via `import type`

**Barrel Files:**
- No `index.ts` barrel files in `src/`
- Direct imports from specific files
- Rust uses `mod.rs` for module declarations

---
*Convention analysis: 2026-03-18*
*Update when patterns change*
