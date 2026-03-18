# External Integrations

**Analysis Date:** 2026-03-18

## APIs & External Services
- **Local HTTP API** (port 8787) - Internal REST API served by Tauri app or standalone `local_api` binary. Frontend consumes `/sessions`, `/events`, `/evaluations`, `/transcripts`, `/settings`, `/hooks`, `/cursor/ai-tracking/*`, `/app-config`.

- **Evaluation API providers** (configurable via settings):
  - OpenAI API (`https://api.openai.com/v1`) - Chat completions endpoint
  - Anthropic API (`https://api.anthropic.com/v1`) - Messages endpoint
  - Ollama (`http://127.0.0.1:11434/api`) - Local chat endpoint

- **Node.js** - `https://nodejs.org/dist/v22.14.0/node-v22.14.0-x64.msi` - Windows installer download when npm is missing

- **npm registry** - `https://registry.npmmirror.com` - Node.js install uses npm mirror (China region)

## Data Storage
- **SQLite** (rusqlite, bundled) - Primary storage for sessions, events, evaluations, transcripts, settings, hooks. Path from config (`~/.config/claude-code-launch/config.json` → `db_path`).

- **Local filesystem** - Cursor transcript files (`~/.cursor/projects/.../agent-transcripts/*.jsonl`), session discovery

## Authentication & Identity
- **None** - No OAuth or user auth. Local app, no auth required for local API.

- **API keys** - User configurable for evaluation providers (OpenAI, Anthropic). Stored in SQLite via `eval_config` table.

## Monitoring & Observability
- **None** - No Sentry, Datadog, or analytics. Logs to stdout/stderr.

## CI/CD & Deployment
- **None** - No `.github` workflows, no CI config found. Manual build via `npm run build` + `tauri build`.

## Environment Configuration
| Variable | Purpose |
|----------|---------|
| `VITE_API_BASE_URL` | Frontend API base URL (default: `http://127.0.0.1:8787`) |
| `TAURI_DEV_HOST` | Vite dev host for Tauri remote dev |
| `CLAUDE_CODE_LAUNCH_PORT` | Local API port (default: 8787) |
| `CLAUDE_CODE_LAUNCH_CONFIG_PATH` | Override config file path |
| `CODE_AGENT_OVERSEER_ALLOWED_ORIGINS` | CORS allowed origins (comma-separated) |

**No `.env` or `.env.example`** in repo. `.gitignore` excludes `*.local`.

## Webhooks & Callbacks
- **Incoming** - POST `/events` accepts events from external agents (e.g. `hooks/report_event.py`). Endpoint configurable via `CODE_AGENT_OVERSEER_ENDPOINT` (default `http://127.0.0.1:8787/events`).

- **Custom hooks** - User-defined hooks in settings (e.g. shell commands). Stored in SQLite, triggered via `/hooks/init` and internal logic.

## Local Integrations
- **Cursor IDE** - Reads `~/.cursor/projects/.../agent-transcripts/*.jsonl` for session discovery and AI tracking.
- **Tauri plugin-opener** - Opens URLs in system browser.

---
*Integration audit: 2026-03-18*
*Update when adding/removing external services*
