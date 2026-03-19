# Technology Stack

**Analysis Date:** 2026-03-18

## Languages
**Primary:**
- TypeScript ~5.8.3 - Frontend (`src/`), Vite config
- Rust (edition 2021) - Backend, Tauri shell, local API (`src-tauri/`)

**Secondary:**
- HTML/CSS - Entry point, Tailwind styles

## Runtime
**Environment:**
- Node.js - Required for npm scripts, Vite dev server

**Package Manager:**
- npm 8+ (lockfileVersion 3)
- Lockfile: present (`package-lock.json`)

**Rust:**
- Cargo - Rust crate manager
- Lockfile: present (`src-tauri/Cargo.lock`)

## Frameworks
**Core:**
- React 19.1.0 - UI framework
- Tauri 2 - Desktop app shell (native window, Rust backend)
- Axum 0.8 - HTTP API server (Rust)

**UI:**
- shadcn 4.0.2 (radix-maia style) - Component library
- Radix UI 1.4.3 - Base primitives
- Base UI React 1.2.0 - Low-level components
- Tailwind CSS 4.2.1 - Utility-first styling
- tw-animate-css 1.4.0 - Animation utilities
- class-variance-authority 0.7.1 - Variant styling
- clsx 2.1.1, tailwind-merge 3.5.0 - Class merging

**Testing:**
- Rust built-in tests (cargo test) - Axum, collection, evaluation, app_config
- No frontend test framework detected

**Build/Dev:**
- Vite 7.0.4 - Bundler, dev server
- @vitejs/plugin-react 4.6.0 - React plugin
- @tailwindcss/vite 4.2.1 - Tailwind CSS plugin

## Key Dependencies
**Critical:**
- @tauri-apps/api 2, @tauri-apps/plugin-opener 2 - Tauri window, shell APIs
- rusqlite 0.37 (bundled) - SQLite storage for sessions, events, evaluations
- @fontsource-variable/geist, @fontsource-variable/inter - Typography

**Infrastructure:**
- tokio 1 - Async runtime (Rust)
- reqwest 0.12 (blocking, json, rustls-tls) - HTTP client for evaluation APIs, Node.js downloads
- serde, serde_json - Serialization
- tower-http 0.6 (cors) - CORS middleware
- uuid 1, chrono 0.4, dirs 6, linemux 0.3, tempfile 3, schemars 0.8 - Utilities

**Icons:**
- lucide-react 0.577.0 - Icon library

## Configuration
**Environment:**
- `VITE_API_BASE_URL` - Frontend API base (default: `http://127.0.0.1:8787`)
- `TAURI_DEV_HOST` - Vite dev host for Tauri remote dev
- `CLAUDE_CODE_LAUNCH_PORT` - Local API port (default: 8787)
- `CLAUDE_CODE_LAUNCH_CONFIG_PATH` - Override config path
- `CODE_AGENT_OVERSEER_ALLOWED_ORIGINS` - CORS allowed origins (comma-separated)

**Build:**
- `vite.config.ts` - Vite dev server (port 1420, strictPort)
- `tsconfig.json` - ES2020, ESNext, bundler mode, path alias `@/*` → `src/*`
- `tsconfig.node.json` - Vite config compilation
- `src-tauri/tauri.conf.json` - Tauri config, build, dev URL
- `components.json` - shadcn config (radix-maia, zinc, lucide)

**App Config:**
- `~/.config/claude-code-launch/config.json` (or `CLAUDE_CODE_LAUNCH_CONFIG_PATH`)
- Fields: `db_path`, `event_enabled`

## Platform Requirements
**Development:**
- Node.js - npm scripts
- Rust toolchain - Tauri build
- macOS / Windows / Linux - Tauri targets

**Production:**
- Desktop app bundle (Tauri native) - macOS, Windows, Linux
- Local HTTP API on 127.0.0.1:8787 (or configured port)
- SQLite database (file path from config)

---
*Stack analysis: 2026-03-18*
*Update after major dependency changes*
