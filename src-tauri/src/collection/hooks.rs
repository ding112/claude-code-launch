use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::path::PathBuf;

const TARGET_EVENTS: &[&str] = &[
    "PermissionRequest",
    "Notification",
    "Stop",
    "SubagentStop",
    "SessionStart",
    "SessionEnd",
    "PreToolUse",
    "PostToolUse",
    "UserPromptSubmit",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct HooksResponse {
    pub events: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct HooksInitResponse {
    pub events: HashMap<String, Value>,
    pub added_count: u32,
}

fn user_claude_settings_path() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| "~".to_string());
    PathBuf::from(home).join(".claude").join("settings.json")
}

fn load_settings_file(path: &PathBuf) -> Result<Value, String> {
    if !path.exists() {
        return Ok(Value::Object(Map::new()));
    }
    let raw = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(Value::Object(Map::new()));
    }
    serde_json::from_str(trimmed)
        .map_err(|e| format!("failed to parse JSON from {}: {e}", path.display()))
}

fn backup_file(path: &PathBuf) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    let timestamp = chrono_timestamp();
    let backup_name = format!("{}.bak.{timestamp}", path.file_name().unwrap_or_default().to_string_lossy());
    let backup_path = path.with_file_name(backup_name);
    std::fs::copy(path, &backup_path)
        .map_err(|e| format!("failed to create backup at {}: {e}", backup_path.display()))?;
    Ok(())
}

fn chrono_timestamp() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let hours = (secs / 3600) % 24;
    let minutes = (secs / 60) % 60;
    let seconds = secs % 60;
    let days = secs / 86400;
    let year = 1970 + days / 365;
    format!("{year}{days:03}-{hours:02}{minutes:02}{seconds:02}")
}

fn write_settings_file(path: &PathBuf, value: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create parent dirs for {}: {e}", path.display()))?;
    }
    let serialized = serde_json::to_string_pretty(value)
        .map_err(|e| format!("failed to serialize settings JSON: {e}"))?;
    std::fs::write(path, format!("{serialized}\n"))
        .map_err(|e| format!("failed to write {}: {e}", path.display()))
}

fn report_event_command() -> String {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()));

    let candidates = [
        exe_dir
            .as_ref()
            .map(|d| d.join("../../../hooks/report_event.py")),
        exe_dir
            .as_ref()
            .map(|d| d.join("../../hooks/report_event.py")),
        Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../hooks/report_event.py")),
    ];

    for candidate in candidates.iter().flatten() {
        if let Ok(canonical) = candidate.canonicalize() {
            return format!("python3 {}", canonical.display());
        }
    }

    let fallback = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../hooks/report_event.py");
    format!("python3 {}", fallback.display())
}

pub(super) fn get_hooks() -> Result<HooksResponse, String> {
    let path = user_claude_settings_path();
    let settings = load_settings_file(&path)?;
    let events = match settings.get("hooks") {
        Some(Value::Object(hooks_map)) => {
            hooks_map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
        }
        _ => HashMap::new(),
    };
    Ok(HooksResponse { events })
}

pub(super) fn save_hooks(payload: HooksResponse) -> Result<HooksResponse, String> {
    let path = user_claude_settings_path();
    let mut settings = load_settings_file(&path)?;

    let hooks_value = serde_json::to_value(&payload.events)
        .map_err(|e| format!("failed to serialize hooks events: {e}"))?;

    if let Some(obj) = settings.as_object_mut() {
        obj.insert("hooks".to_string(), hooks_value);
    } else {
        return Err("settings root is not a JSON object".to_string());
    }

    backup_file(&path)?;
    write_settings_file(&path, &settings)?;

    Ok(payload)
}

pub(super) fn init_hooks() -> Result<HooksInitResponse, String> {
    let path = user_claude_settings_path();
    let mut settings = load_settings_file(&path)?;
    let command = report_event_command();

    let hooks = settings
        .as_object_mut()
        .ok_or("settings root is not a JSON object")?
        .entry("hooks")
        .or_insert_with(|| Value::Object(Map::new()));

    let hooks_obj = hooks
        .as_object_mut()
        .ok_or("settings.hooks is not a JSON object")?;

    let mut added_count: u32 = 0;

    for &event_name in TARGET_EVENTS {
        let blocks = hooks_obj
            .entry(event_name)
            .or_insert_with(|| Value::Array(Vec::new()));

        let blocks_arr = blocks
            .as_array_mut()
            .ok_or_else(|| format!("hooks.{event_name} is not a JSON array"))?;

        let target_block = blocks_arr.iter_mut().find(|block| {
            block.get("matcher").and_then(Value::as_str) == Some("*")
                && block.get("hooks").map(Value::is_array).unwrap_or(false)
        });

        let hook_items = match target_block {
            Some(block) => block
                .get_mut("hooks")
                .and_then(Value::as_array_mut)
                .unwrap(),
            None => {
                let new_block = serde_json::json!({"matcher": "*", "hooks": []});
                blocks_arr.push(new_block);
                blocks_arr
                    .last_mut()
                    .unwrap()
                    .get_mut("hooks")
                    .unwrap()
                    .as_array_mut()
                    .unwrap()
            }
        };

        let already_exists = hook_items.iter().any(|item| {
            item.get("type").and_then(Value::as_str) == Some("command")
                && item.get("command").and_then(Value::as_str) == Some(&command)
        });

        if !already_exists {
            hook_items.push(serde_json::json!({
                "type": "command",
                "command": command,
            }));
            added_count += 1;
        }
    }

    backup_file(&path)?;
    write_settings_file(&path, &settings)?;

    let events = match settings.get("hooks") {
        Some(Value::Object(hooks_map)) => {
            hooks_map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
        }
        _ => HashMap::new(),
    };

    Ok(HooksInitResponse {
        events,
        added_count,
    })
}
