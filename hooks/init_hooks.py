#!/usr/bin/env python3
import argparse
import copy
import json
import sys
from datetime import datetime
from pathlib import Path
from typing import Any


TARGET_EVENTS = [
    "PermissionRequest",
    "Notification",
    "Stop",
    "SubagentStop",
    "SessionStart",
    "SessionEnd",
    "PreToolUse",
    "PostToolUse",
    "UserPromptSubmit",
]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Initialize Claude hooks by merging target events into settings.json.",
    )
    parser.add_argument(
        "--target",
        choices=["repo", "user"],
        default="repo",
        help="Target settings scope. Default: repo",
    )
    parser.add_argument(
        "--config",
        type=str,
        default="",
        help="Optional explicit settings.json path. Overrides --target.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Preview changes without writing files.",
    )
    return parser.parse_args()


def repo_root() -> Path:
    return Path(__file__).resolve().parents[1]


def resolve_config_path(args: argparse.Namespace) -> Path:
    if args.config.strip():
        return Path(args.config).expanduser().resolve()
    if args.target == "user":
        return (Path.home() / ".claude" / "settings.json").resolve()
    return (repo_root() / ".claude" / "settings.json").resolve()


def load_settings(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    raw = path.read_text(encoding="utf-8")
    if not raw.strip():
        return {}
    parsed = json.loads(raw)
    if not isinstance(parsed, dict):
        raise ValueError(f"settings root must be object: {path}")
    return parsed


def ensure_hooks_container(settings: dict[str, Any]) -> dict[str, Any]:
    hooks = settings.get("hooks")
    if hooks is None:
        hooks = {}
        settings["hooks"] = hooks
    if not isinstance(hooks, dict):
        raise ValueError("settings.hooks must be a JSON object")
    return hooks


def ensure_event_blocks(hooks: dict[str, Any], event_name: str) -> list[dict[str, Any]]:
    blocks = hooks.get(event_name)
    if blocks is None:
        blocks = []
        hooks[event_name] = blocks
    if not isinstance(blocks, list):
        raise ValueError(f"hooks.{event_name} must be a JSON array")
    normalized: list[dict[str, Any]] = []
    for block in blocks:
        if isinstance(block, dict):
            normalized.append(block)
    hooks[event_name] = normalized
    return normalized


def ensure_command_hook(blocks: list[dict[str, Any]], command: str) -> bool:
    target_block: dict[str, Any] | None = None
    for block in blocks:
        if block.get("matcher") == "*" and isinstance(block.get("hooks"), list):
            target_block = block
            break

    if target_block is None:
        target_block = {"matcher": "*", "hooks": []}
        blocks.append(target_block)

    hook_items = target_block.get("hooks")
    if not isinstance(hook_items, list):
        hook_items = []
        target_block["hooks"] = hook_items

    for item in hook_items:
        if (
            isinstance(item, dict)
            and item.get("type") == "command"
            and item.get("command") == command
        ):
            return False

    hook_items.append({"type": "command", "command": command})
    return True


def merge_target_events(settings: dict[str, Any], command: str) -> tuple[dict[str, Any], int]:
    merged = copy.deepcopy(settings)
    hooks = ensure_hooks_container(merged)
    added_count = 0
    for event in TARGET_EVENTS:
        blocks = ensure_event_blocks(hooks, event)
        if ensure_command_hook(blocks, command):
            added_count += 1
    return merged, added_count


def backup_file(path: Path) -> Path:
    timestamp = datetime.now().strftime("%Y%m%d-%H%M%S-%f")
    backup_path = path.with_name(f"{path.name}.bak.{timestamp}")
    suffix = 1
    while backup_path.exists():
        backup_path = path.with_name(f"{path.name}.bak.{timestamp}-{suffix}")
        suffix += 1
    backup_path.write_text(path.read_text(encoding="utf-8"), encoding="utf-8")
    return backup_path


def write_settings(path: Path, data: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    serialized = json.dumps(data, ensure_ascii=False, indent=2) + "\n"
    path.write_text(serialized, encoding="utf-8")


def main() -> int:
    args = parse_args()
    config_path = resolve_config_path(args)
    report_script = (repo_root() / "hooks" / "report_event.py").resolve()
    if sys.platform == "win32":
        python_cmd = sys.executable.replace("\\", "/")
        script_path = str(report_script).replace("\\", "/")
    else:
        python_cmd = "python3"
        script_path = str(report_script)
    command = f"{python_cmd} {script_path}"

    original = load_settings(config_path)
    merged, added_count = merge_target_events(original, command)

    print(f"[init-hooks] target settings: {config_path}")
    print(f"[init-hooks] command: {command}")
    print(f"[init-hooks] events ensured: {', '.join(TARGET_EVENTS)}")
    print(f"[init-hooks] new command entries added: {added_count}")

    if args.dry_run:
        print("[init-hooks] dry-run enabled, no file changes")
        return 0

    if config_path.exists():
        backup_path = backup_file(config_path)
        print(f"[init-hooks] backup created: {backup_path}")
    else:
        print("[init-hooks] target file not found, will create a new one")

    write_settings(config_path, merged)
    print("[init-hooks] settings updated successfully")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
