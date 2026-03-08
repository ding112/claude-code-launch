#!/usr/bin/env python3
import json
import os
import sys
import threading
import time
import traceback
import urllib.error
import urllib.request
import uuid
from datetime import datetime, timezone
import re


def now_ms() -> int:
    return int(datetime.now(tz=timezone.utc).timestamp() * 1000)


def read_stdin_utf8() -> str:
    raw_bytes = sys.stdin.buffer.read()
    return raw_bytes.decode("utf-8-sig").strip()


def parse_stdin_json(raw_stdin: str) -> dict:
    if not raw_stdin:
        return {}
    try:
        parsed = json.loads(raw_stdin)
        if isinstance(parsed, dict):
            return parsed
    except json.JSONDecodeError:
        pass
    return {}


def derive_project_name(stdin_data: dict) -> str:
    nested = extract_nested_stdin_json(stdin_data)

    if is_cursor_stdin(stdin_data):
        workspace_roots = first_non_empty_string_list(
            nested.get("workspace_roots"),
            stdin_data.get("workspace_roots"),
        )
        if workspace_roots:
            normalized = os.path.normpath(workspace_roots[0])
            basename = os.path.basename(normalized)
            if basename:
                return basename
        return "unknown-project"

    project_name = first_non_empty_string(
        nested.get("project_name"),
        stdin_data.get("project_name"),
    )
    if project_name:
        return project_name

    cwd = first_non_empty_string(
        nested.get("cwd"),
        stdin_data.get("cwd"),
        os.getcwd(),
    )
    if cwd:
        normalized = os.path.normpath(cwd)
        basename = os.path.basename(normalized)
        if basename:
            return basename

    env_project_name = os.getenv("CLAUDE_PROJECT_NAME", "").strip()
    if env_project_name:
        return env_project_name
    return "unknown-project"


def extract_nested_stdin_json(stdin_data: dict) -> dict:
    nested = stdin_data.get("stdin_json")
    if isinstance(nested, dict):
        return nested
    return {}


def first_non_empty_string(*values: object) -> str:
    for value in values:
        if isinstance(value, str) and value.strip():
            return value.strip()
    return ""


def first_non_empty_string_list(*values: object) -> list[str]:
    for value in values:
        if not isinstance(value, list):
            continue

        cleaned: list[str] = []
        for item in value:
            if isinstance(item, str) and item.strip():
                cleaned.append(item.strip())
        if cleaned:
            return cleaned
    return []


def parse_bool_env(name: str, default: bool = False) -> bool:
    raw = os.getenv(name)
    if raw is None:
        return default
    normalized = raw.strip().lower()
    if not normalized:
        return default
    return normalized in {"1", "true", "yes", "on"}


def is_cursor_stdin(stdin_data: dict) -> bool:
    nested = extract_nested_stdin_json(stdin_data)
    return bool(
        first_non_empty_string(
            nested.get("cursor_version"),
            stdin_data.get("cursor_version"),
        )
    )


def normalize_event_type(raw_event_type: str) -> str:
    if not raw_event_type:
        return "UnknownEvent"

    cleaned = raw_event_type.strip()
    if not cleaned:
        return "UnknownEvent"

    # 支持 camelCase/PascalCase/snake_case/kebab-case/dot.case 等格式。
    normalized = re.sub(r"([a-z0-9])([A-Z])", r"\1 \2", cleaned)
    normalized = re.sub(r"[^A-Za-z0-9]+", " ", normalized)
    parts = [part for part in normalized.split() if part]
    if not parts:
        return "UnknownEvent"
    return "".join(part[0].upper() + part[1:] for part in parts)


def build_payload() -> dict:
    raw_stdin = read_stdin_utf8()
    stdin_data = parse_stdin_json(raw_stdin)
    nested = extract_nested_stdin_json(stdin_data)

    session_id = (
        first_non_empty_string(
            nested.get("conversation_id"),
            nested.get("session_id"),
            stdin_data.get("conversation_id"),
            stdin_data.get("session_id"),
            os.getenv("CLAUDE_SESSION_ID", ""),
        )
        or "unknown-session"
    )
    original_event_type = (
        first_non_empty_string(
            nested.get("hook_event_name"),
            stdin_data.get("hook_event_name"),
            os.getenv("CLAUDE_HOOK_EVENT_NAME", ""),
        )
        or "unknown-event"
    )
    event_type = normalize_event_type(original_event_type)
    project_name = derive_project_name(stdin_data)
    event_id = os.getenv("CODE_AGENT_EVENT_ID", str(uuid.uuid4()))

    event_payload = {
        "raw_stdin": raw_stdin,
        "stdin_json": stdin_data,
        "original_hook_event_name": original_event_type,
        "cwd": os.getcwd(),
        "token": os.getenv("ANTHROPIC_API_KEY", ""),
        "timestamp": now_ms(),
    }

    return {
        "event_id": event_id,
        "session_id": session_id,
        "project_name": project_name,
        "event_type": event_type,
        "payload": event_payload,
        "created_at_ms": now_ms(),
    }


def endpoints() -> list[str]:
    endpoints_raw = os.getenv("CODE_AGENT_OVERSEER_ENDPOINTS")
    if endpoints_raw:
        parsed = [item.strip() for item in endpoints_raw.split(",") if item.strip()]
        if parsed:
            return parsed
    default_url = os.getenv("CODE_AGENT_OVERSEER_ENDPOINT", "http://127.0.0.1:8787/events")
    return [default_url]


def post_json(url: str, payload: dict, timeout_seconds: float) -> tuple[int, str]:
    data = json.dumps(payload).encode("utf-8")
    request = urllib.request.Request(
        url=url,
        data=data,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    with urllib.request.urlopen(request, timeout=timeout_seconds) as response:
        return response.status, response.read().decode("utf-8")


def report_with_retry(payload: dict) -> bool:
    target_endpoints = endpoints()
    max_attempts = int(os.getenv("CODE_AGENT_HOOK_RETRY_ATTEMPTS", "4"))
    backoff_seconds = float(os.getenv("CODE_AGENT_HOOK_BACKOFF_SECONDS", "0.3"))
    timeout_seconds = float(os.getenv("CODE_AGENT_HOOK_TIMEOUT_SECONDS", "3"))
    verbose = parse_bool_env("CODE_AGENT_HOOK_VERBOSE", False)

    for attempt in range(max_attempts):
        endpoint = target_endpoints[attempt % len(target_endpoints)]
        try:
            status, body = post_json(endpoint, payload, timeout_seconds)
            if 200 <= status < 300:
                if verbose:
                    print(
                        f"[hook] report success attempt={attempt + 1} endpoint={endpoint} status={status}",
                        file=sys.stderr,
                    )
                return True
            if verbose:
                print(
                    f"[hook] non-2xx response attempt={attempt + 1} endpoint={endpoint} status={status}",
                    file=sys.stderr,
                )
        except Exception:
            if verbose:
                print(
                    f"[hook] error attempt={attempt + 1} endpoint={endpoint}",
                    file=sys.stderr,
                )
                traceback.print_exc(file=sys.stderr)

        if attempt < max_attempts - 1:
            delay = backoff_seconds * (2 ** attempt)
            time.sleep(delay)

    if verbose:
        print("[hook] report failed after all retry attempts", file=sys.stderr)
    return False


def report_with_background_thread(payload: dict) -> None:
    timeout_seconds = float(os.getenv("CODE_AGENT_HOOK_TIMEOUT_SECONDS", "3"))
    max_attempts = int(os.getenv("CODE_AGENT_HOOK_RETRY_ATTEMPTS", "4"))
    join_timeout = timeout_seconds * max_attempts + 2.0

    def worker() -> None:
        try:
            report_with_retry(payload)
        except Exception:
            pass

    thread = threading.Thread(target=worker, name="hook-report-worker", daemon=True)
    thread.start()
    thread.join(timeout=join_timeout)


def main() -> int:
    payload = build_payload()
    if parse_bool_env("CODE_AGENT_HOOK_SYNC", False):
        report_with_retry(payload)
        return 0

    report_with_background_thread(payload)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
