#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"
mkdir -p runtime
SMOKE_PORT="${SMOKE_PORT:-18787}"
SMOKE_ENDPOINT="http://127.0.0.1:${SMOKE_PORT}"

SMOKE_CONFIG="$(mktemp -d)/config.json"
SMOKE_DB_PATH="${ROOT_DIR}/runtime/smoke.sqlite3"
cat > "${SMOKE_CONFIG}" <<CFGEOF
{ "db_path": "${SMOKE_DB_PATH}" }
CFGEOF

echo "[smoke] start local API server"
CODE_AGENT_OVERSEER_PORT="${SMOKE_PORT}" CODE_AGENT_OVERSEER_CONFIG_PATH="${SMOKE_CONFIG}" \
  cargo run --manifest-path src-tauri/Cargo.toml --bin local_api >runtime/smoke-server.log 2>&1 &
SERVER_PID=$!

cleanup() {
  kill "$SERVER_PID" >/dev/null 2>&1 || true
  rm -f "${SMOKE_CONFIG}" 2>/dev/null || true
}
trap cleanup EXIT

sleep 2
for _ in $(seq 1 20); do
  if curl -sf "${SMOKE_ENDPOINT}/health" >/dev/null; then
    break
  fi
  sleep 1
done
if ! curl -sf "${SMOKE_ENDPOINT}/health" >/dev/null; then
  echo "[smoke] local API server not ready: ${SMOKE_ENDPOINT}/health" >&2
  exit 1
fi

echo "[smoke] scenario A: success report"
CLAUDE_SESSION_ID="smoke-session-a" \
CLAUDE_PROJECT_NAME="smoke-project" \
CLAUDE_HOOK_EVENT_NAME="tool.call" \
CODE_AGENT_HOOK_SYNC=1 \
CODE_AGENT_OVERSEER_ENDPOINT="${SMOKE_ENDPOINT}/events" \
python3 hooks/report_event.py <<< '{"tool":"list-files"}'

echo "[smoke] scenario B: retry then success"
CLAUDE_SESSION_ID="smoke-session-b" \
CLAUDE_PROJECT_NAME="smoke-project" \
CLAUDE_HOOK_EVENT_NAME="tool.call" \
CODE_AGENT_HOOK_SYNC=1 \
CODE_AGENT_OVERSEER_ENDPOINTS="http://127.0.0.1:8799/events,${SMOKE_ENDPOINT}/events" \
CODE_AGENT_HOOK_RETRY_ATTEMPTS=4 \
python3 hooks/report_event.py <<< '{"tool":"read-file"}'

echo "[smoke] scenario C: cursor nested stdin_json"
env -u CLAUDE_SESSION_ID -u CLAUDE_HOOK_EVENT_NAME -u CLAUDE_PROJECT_NAME \
  CODE_AGENT_HOOK_SYNC=1 \
  CODE_AGENT_OVERSEER_ENDPOINT="${SMOKE_ENDPOINT}/events" \
  python3 hooks/report_event.py <<'EOF'
{"cwd":"/Users/ding/workspace/github/code-agent-overseer","stdin_json":{"conversation_id":"smoke-conversation-c","hook_event_name":"postToolUse","cursor_version":"2.5.25","workspace_roots":["/Users/ding/workspace/github/code-agent-overseer"],"tool_name":"Read","tool_input":{"file_path":"/Users/ding/workspace/github/code-agent-overseer/README.md"}}}
EOF

echo "[smoke] scenario E: cursor strict workspace fallback"
env -u CLAUDE_SESSION_ID -u CLAUDE_HOOK_EVENT_NAME -u CLAUDE_PROJECT_NAME \
  CODE_AGENT_HOOK_SYNC=1 \
  CODE_AGENT_OVERSEER_ENDPOINT="${SMOKE_ENDPOINT}/events" \
  python3 hooks/report_event.py <<'EOF'
{"cwd":"/Users/ding/workspace/github/code-agent-overseer","stdin_json":{"conversation_id":"smoke-conversation-e","hook_event_name":"postToolUse","cursor_version":"2.5.25","tool_name":"Read","tool_input":{"file_path":"/Users/ding/workspace/github/code-agent-overseer/README.md"}}}
EOF

echo "[smoke] scenario D: fallback unknown values"
env -u CLAUDE_SESSION_ID -u CLAUDE_HOOK_EVENT_NAME -u CLAUDE_PROJECT_NAME \
  CODE_AGENT_HOOK_SYNC=1 \
  CODE_AGENT_OVERSEER_ENDPOINT="${SMOKE_ENDPOINT}/events" \
  python3 hooks/report_event.py <<< '{}'

echo "[smoke] scenario F: async non-blocking return"
START_NS="$(python3 -c 'import time; print(time.time_ns())')"
env -u CODE_AGENT_HOOK_SYNC \
  CODE_AGENT_OVERSEER_ENDPOINT="http://127.0.0.1:8799/events" \
  CODE_AGENT_HOOK_RETRY_ATTEMPTS=4 \
  CODE_AGENT_HOOK_BACKOFF_SECONDS=0.5 \
  CODE_AGENT_HOOK_TIMEOUT_SECONDS=2 \
  python3 hooks/report_event.py <<< '{"tool":"non-blocking-check"}'
END_NS="$(python3 -c 'import time; print(time.time_ns())')"
ELAPSED_MS="$(((END_NS - START_NS) / 1000000))"
if [ "${ELAPSED_MS}" -gt 700 ]; then
  echo "[smoke] async mode should return quickly, elapsed=${ELAPSED_MS}ms" >&2
  exit 1
fi

echo "[smoke] verify dashboard query APIs"
SESSION_COUNT="$(curl -s "${SMOKE_ENDPOINT}/sessions" | python3 -c 'import json,sys; data=json.load(sys.stdin); print(len(data))')"
if [ "${SESSION_COUNT}" -lt 1 ]; then
  echo "[smoke] sessions API returned empty list" >&2
  exit 1
fi

EVALUATION_TOTAL="$(curl -s "${SMOKE_ENDPOINT}/evaluations?session_id=smoke-session-b&page=1&page_size=10" | python3 -c 'import json,sys; data=json.load(sys.stdin); print(data.get("total", 0))')"
if [ "${EVALUATION_TOTAL}" -lt 1 ]; then
  echo "[smoke] evaluations API did not return expected records" >&2
  exit 1
fi

echo "[smoke] verify provider switch settings"
curl -s -X POST "${SMOKE_ENDPOINT}/settings" \
  -H "Content-Type: application/json" \
  -d '{"enabled":true,"sampling_rate":1,"provider":"ollama","model":"llama3","base_url":"http://127.0.0.1:11434/api","api_key":"","timeout_ms":5000}' >/dev/null

PROVIDER_VALUE="$(curl -s "${SMOKE_ENDPOINT}/settings" | python3 -c 'import json,sys; data=json.load(sys.stdin); print(data.get("provider",""))')"
if [ "${PROVIDER_VALUE}" != "ollama" ]; then
  echo "[smoke] provider switch not persisted" >&2
  exit 1
fi

echo "[smoke] verify cursor event mapping"
CURSOR_EVENT_CHECK="$(curl -s "${SMOKE_ENDPOINT}/events?session_id=smoke-conversation-c&page=1&page_size=10" | python3 -c 'import json,sys; data=json.load(sys.stdin); items=data.get("items", []); ok=bool(items) and any(item.get("event_type")=="PostToolUse" for item in items); print("ok" if ok else "fail")')"
if [ "${CURSOR_EVENT_CHECK}" != "ok" ]; then
  echo "[smoke] cursor mapping failed: expected session_id=smoke-conversation-c event_type=PostToolUse" >&2
  exit 1
fi

CURSOR_PROJECT_CHECK="$(curl -s "${SMOKE_ENDPOINT}/sessions" | python3 -c 'import json,sys; data=json.load(sys.stdin); target=next((s for s in data if s.get("session_id")=="smoke-conversation-c"), None); ok=bool(target) and target.get("project_name")=="code-agent-overseer"; print("ok" if ok else "fail")')"
if [ "${CURSOR_PROJECT_CHECK}" != "ok" ]; then
  echo "[smoke] cursor project mapping failed: expected session_id=smoke-conversation-c project_name=code-agent-overseer" >&2
  exit 1
fi

echo "[smoke] verify cursor strict workspace fallback"
CURSOR_WORKSPACE_FALLBACK_CHECK="$(curl -s "${SMOKE_ENDPOINT}/sessions" | python3 -c 'import json,sys; data=json.load(sys.stdin); target=next((s for s in data if s.get("session_id")=="smoke-conversation-e"), None); ok=bool(target) and target.get("project_name")=="unknown-project"; print("ok" if ok else "fail")')"
if [ "${CURSOR_WORKSPACE_FALLBACK_CHECK}" != "ok" ]; then
  echo "[smoke] cursor strict fallback failed: expected session_id=smoke-conversation-e project_name=unknown-project" >&2
  exit 1
fi

echo "[smoke] verify fallback mapping"
FALLBACK_EVENT_CHECK="$(curl -s "${SMOKE_ENDPOINT}/events?session_id=unknown-session&page=1&page_size=10" | python3 -c 'import json,sys; data=json.load(sys.stdin); items=data.get("items", []); ok=bool(items) and items[0].get("event_type")=="UnknownEvent"; print("ok" if ok else "fail")')"
if [ "${FALLBACK_EVENT_CHECK}" != "ok" ]; then
  echo "[smoke] fallback mapping failed: expected session_id=unknown-session event_type=UnknownEvent" >&2
  exit 1
fi

echo "[smoke] verify frontend build"
npm run build >/dev/null

echo "[smoke] done; log file: runtime/smoke-server.log"
