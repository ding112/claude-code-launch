# Codebase Structure

**Analysis Date:** 2026-03-18

## Directory Layout

```
claude-code-launch/
├── src/                    # Frontend (React + TypeScript)
│   ├── components/         # Reusable UI components
│   │   └── ui/            # shadcn/Radix primitives
│   ├── hooks/             # Custom React hooks
│   ├── pages/             # Page-level components
│   ├── lib/               # Utility library (cn helper)
│   ├── App.tsx            # Root component, routing
│   ├── main.tsx           # React entry point
│   ├── api.ts             # Backend API client
│   ├── types.ts           # TypeScript type definitions
│   ├── constants.ts       # App constants
│   ├── utils.ts           # Utility functions
│   └── TranscriptView.tsx # Transcript display component
├── src-tauri/             # Backend (Rust + Tauri)
│   ├── src/
│   │   ├── collection/    # Core monitoring module
│   │   ├── services/      # Setup/install services
│   │   ├── commands/      # Tauri IPC commands
│   │   ├── dao/           # Data access objects
│   │   ├── models/        # Data models
│   │   ├── bin/           # Standalone binaries
│   │   ├── lib.rs         # Tauri app setup
│   │   ├── main.rs        # Tauri entry point
│   │   ├── collection.rs  # Collection module root
│   │   ├── evaluation.rs  # LLM evaluation logic
│   │   ├── app_config.rs  # App configuration
│   │   └── overseer_models.rs # API models
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri configuration
├── hooks/                 # Python helper scripts
│   ├── init_hooks.py      # Hook initialization
│   ├── report_event.py    # Event reporting
│   └── smoke_test.sh      # Smoke test script
├── docs/                  # Documentation
│   ├── architecture-design.md
│   └── domain-model.md
├── package.json           # Node.js dependencies
├── vite.config.ts         # Vite build config
├── tsconfig.json          # TypeScript config
├── components.json        # shadcn config
└── .gitignore             # Git exclusions
```

## Directory Purposes

**`src/`**
- Purpose: Frontend React application
- Contains: TSX components, hooks, API client, types, styles
- Key files: `App.tsx` (routing), `api.ts` (backend communication), `types.ts` (shared types)

**`src/components/`**
- Purpose: Reusable UI components
- Contains: `Pager.tsx`, `TabSwitcher.tsx`, `LogPanel.tsx`, `AddCommandInput.tsx`
- Subdirectories: `ui/` — shadcn primitives (button, card, select, tabs, etc.)

**`src/hooks/`**
- Purpose: Custom React hooks encapsulating API calls and state
- Contains: `useSessions.ts`, `useTranscript.ts`, `useEvents.ts`, `useEvaluations.ts`, `useEvalSettings.ts`, `useHooksConfig.ts`, `usePrereqs.ts`, `useInstall.ts`, `useVerify.ts`

**`src/pages/`**
- Purpose: Page-level components (top-level views)
- Contains: `SessionsPage.tsx` (main monitoring), `SettingsPage.tsx` (config), `SetupPage.tsx` (wizard)

**`src-tauri/src/`**
- Purpose: Rust backend — Tauri app, HTTP server, data layer
- Key files: `lib.rs` (app setup), `main.rs` (entry), `collection.rs` (monitoring module root), `evaluation.rs` (LLM eval)

**`src-tauri/src/collection/`**
- Purpose: Core monitoring data collection and serving
- Contains: `handlers.rs` (Axum routes), `db.rs` (SQLite schema/queries), `discovery.rs` (session discovery), `cursor_discovery.rs` (Cursor IDE integration), `transcript.rs` (transcript parsing), `transcript_poller.rs` (background polling), `eval_queue.rs` (evaluation worker), `hooks.rs` (custom hooks), `cursor_ai_tracking.rs` (AI tracking)

**`src-tauri/src/services/`**
- Purpose: Setup wizard business logic
- Contains: `prereq_service.rs` (check prerequisites), `install_service.rs` (install deps), `node_install_service.rs` (Node.js download), `verify_service.rs` (verify installation)

**`src-tauri/src/bin/`**
- Purpose: Standalone binary entry points
- Contains: `local_api.rs` — headless HTTP server without Tauri GUI

**`hooks/`**
- Purpose: Python helper scripts for external agent integration
- Contains: `init_hooks.py` (initialize Cursor hooks), `report_event.py` (report events to API), `smoke_test.sh` (basic testing)

**`docs/`**
- Purpose: Project documentation
- Contains: `architecture-design.md`, `domain-model.md`

## Key File Locations

**Entry Points:**
- `src/main.tsx` — React app entry, renders `<App />`
- `src/App.tsx` — Root component, tab routing (Setup / Sessions / Settings)
- `src-tauri/src/main.rs` — Tauri binary entry
- `src-tauri/src/lib.rs` — Tauri app builder, command registration, server startup
- `src-tauri/src/bin/local_api.rs` — Standalone API server entry

**Configuration:**
- `vite.config.ts` — Vite dev server (port 1420)
- `tsconfig.json` — TypeScript config, path aliases (`@/*` → `src/*`)
- `src-tauri/tauri.conf.json` — Tauri app config, window settings
- `src-tauri/Cargo.toml` — Rust dependencies
- `components.json` — shadcn component config
- `package.json` — Node.js scripts and dependencies

**Core Logic:**
- `src/api.ts` — All frontend API calls
- `src/types.ts` — Shared TypeScript types
- `src-tauri/src/collection.rs` — Collection module root, AppState
- `src-tauri/src/collection/handlers.rs` — All REST API route handlers
- `src-tauri/src/collection/db.rs` — SQLite schema and queries
- `src-tauri/src/evaluation.rs` — LLM evaluation logic (OpenAI, Anthropic, Ollama)

## Naming Conventions

**Files:**
- PascalCase for React components and pages: `SessionsPage.tsx`, `Pager.tsx`, `TranscriptView.tsx`
- camelCase for hooks: `useSessions.ts`, `useTranscript.ts`
- lowercase for utilities: `api.ts`, `utils.ts`, `constants.ts`, `types.ts`
- kebab-case for shadcn UI primitives: `button.tsx`, `scroll-area.tsx`
- snake_case for Rust modules: `eval_queue.rs`, `cursor_discovery.rs`

**Directories:**
- lowercase for all directories: `hooks/`, `pages/`, `components/`, `services/`
- Plural for collections: `hooks/`, `pages/`, `services/`, `models/`

**Special Patterns:**
- `use*.ts` prefix for React hooks
- `*_service.rs` suffix for Rust service modules
- `mod.rs` for Rust module declarations

## Where to Add New Code

**New Frontend Page:**
- Component: `src/pages/NewPage.tsx`
- Hook: `src/hooks/useNewFeature.ts`
- API calls: Add to `src/api.ts`
- Types: Add to `src/types.ts`
- Route: Add tab in `src/App.tsx`

**New Backend API Endpoint:**
- Handler: Add to `src-tauri/src/collection/handlers.rs`
- Route: Register in `src-tauri/src/collection.rs` (Axum router)
- DB queries: Add to `src-tauri/src/collection/db.rs`

**New UI Component:**
- Reusable: `src/components/NewComponent.tsx`
- shadcn primitive: `src/components/ui/new-component.tsx`

**New Tauri Command:**
- Command: `src-tauri/src/commands/mod.rs`
- Service: `src-tauri/src/services/new_service.rs`
- Register: `src-tauri/src/lib.rs` (invoke_handler)

**New Background Worker:**
- Worker: `src-tauri/src/collection/new_worker.rs`
- Spawn: `src-tauri/src/collection.rs` (in server startup)

## Special Directories

**`src/components/ui/`**
- Purpose: shadcn/Radix UI primitives
- Source: Generated by `npx shadcn add <component>`
- Committed: Yes

**`src-tauri/target/`**
- Purpose: Rust build output
- Source: Cargo build artifacts
- Committed: No (gitignored)

**`node_modules/`**
- Purpose: npm dependencies
- Source: `npm install`
- Committed: No (gitignored)

---
*Structure analysis: 2026-03-18*
*Update when directory structure changes*
