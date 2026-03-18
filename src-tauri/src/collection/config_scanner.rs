use serde::Serialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

#[derive(Debug, Clone, Serialize)]
pub(super) struct ConfigItem {
    pub id: String,
    pub source: String,
    pub name: String,
    pub category: String,
    pub scope: String,
    pub project_path: Option<String>,
    pub file_path: String,
    pub status: String,
    pub size_bytes: Option<u64>,
    pub last_modified_ms: Option<i64>,
}

#[derive(Debug, Serialize)]
pub(super) struct ConfigsResponse {
    pub items: Vec<ConfigItem>,
    pub project_count: usize,
}

#[derive(Debug, Serialize)]
pub(super) struct ConfigContentResponse {
    pub id: String,
    pub name: String,
    pub file_path: String,
    pub content: String,
    pub content_type: String,
}

const MAX_CONTENT_BYTES: u64 = 1_048_576; // 1 MB

pub(super) fn scan_all_configs(project_paths: &[String]) -> Vec<ConfigItem> {
    let mut items = Vec::new();
    items.extend(scan_claude_global_configs());
    items.extend(scan_cursor_global_configs());
    for project_path in project_paths {
        items.extend(scan_claude_project_configs(project_path));
        items.extend(scan_cursor_project_configs(project_path));
    }
    items
}

pub(super) fn discover_project_paths() -> Vec<String> {
    let mut paths = HashSet::new();

    if let Some(dir) = claude_projects_dir() {
        collect_decoded_paths(&dir, &mut paths);
    }
    if let Some(dir) = cursor_projects_dir() {
        collect_decoded_paths(&dir, &mut paths);
    }

    paths.into_iter().collect()
}

pub(super) fn read_config_content(item: &ConfigItem) -> Result<ConfigContentResponse, String> {
    if item.status == "missing" {
        return Err(format!("config file does not exist: {}", item.file_path));
    }

    let path = Path::new(&item.file_path);
    let meta = std::fs::metadata(path)
        .map_err(|e| format!("failed to read metadata for {}: {e}", item.file_path))?;

    if meta.len() > MAX_CONTENT_BYTES {
        return Err(format!(
            "config file too large ({} bytes, max {})",
            meta.len(),
            MAX_CONTENT_BYTES
        ));
    }

    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read {}: {e}", item.file_path))?;

    let content_type = detect_content_type(&item.file_path);

    Ok(ConfigContentResponse {
        id: item.id.clone(),
        name: item.name.clone(),
        file_path: item.file_path.clone(),
        content,
        content_type,
    })
}

// ── Path helpers ──

fn claude_home() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude"))
}

fn claude_projects_dir() -> Option<PathBuf> {
    claude_home().map(|h| h.join("projects"))
}

fn cursor_home() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".cursor"))
}

fn cursor_projects_dir() -> Option<PathBuf> {
    cursor_home().map(|h| h.join("projects"))
}

fn agents_home() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".agents"))
}

// ── Project path decoding ──

fn collect_decoded_paths(dir: &Path, paths: &mut HashSet<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let decoded = format!("/{}", name.replace('-', "/"));
        if Path::new(&decoded).is_dir() {
            paths.insert(decoded);
        }
    }
}

// ── Claude global configs ──

fn scan_claude_global_configs() -> Vec<ConfigItem> {
    let Some(home) = claude_home() else {
        return Vec::new();
    };
    let mut items = Vec::new();

    items.push(probe_config(
        "claude", "global", "claude-md", "CLAUDE.md",
        &home.join("CLAUDE.md"), None,
    ));

    items.push(probe_config(
        "claude", "global", "settings", "settings.json",
        &home.join("settings.json"), None,
    ));

    if let Ok(entries) = std::fs::read_dir(home.join("commands")) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("md") {
                let fname = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                items.push(probe_config(
                    "claude", "global", "commands", &fname,
                    &path, None,
                ));
            }
        }
    }

    let settings_path = home.join("settings.json");
    if has_hooks_in_settings(&settings_path) {
        items.push(ConfigItem {
            id: make_id("claude", "global", "hooks", None, None),
            source: "claude".to_string(),
            name: "hooks (in settings.json)".to_string(),
            category: "hooks".to_string(),
            scope: "global".to_string(),
            project_path: None,
            file_path: settings_path.to_string_lossy().to_string(),
            status: "active".to_string(),
            size_bytes: None,
            last_modified_ms: None,
        });
    }

    items
}

// ── Cursor global configs ──

fn scan_cursor_global_configs() -> Vec<ConfigItem> {
    let Some(home) = cursor_home() else {
        return Vec::new();
    };
    let mut items = Vec::new();

    items.push(probe_config(
        "cursor", "global", "mcp", "mcp.json",
        &home.join("mcp.json"), None,
    ));

    collect_skills(&home.join("skills"), "cursor", "global", None, &mut items);

    if let Some(agents) = agents_home() {
        collect_skills(&agents.join("skills"), "cursor", "global", None, &mut items);
    }

    items
}

// ── Claude project configs ──

fn scan_claude_project_configs(project_path: &str) -> Vec<ConfigItem> {
    let base = Path::new(project_path);
    if !base.is_dir() {
        return Vec::new();
    }
    let mut items = Vec::new();
    let pp = Some(project_path.to_string());

    let root_claude_md = base.join("CLAUDE.md");
    let dot_claude_md = base.join(".claude").join("CLAUDE.md");
    if root_claude_md.is_file() {
        items.push(probe_config("claude", "project", "claude-md", "CLAUDE.md", &root_claude_md, pp.clone()));
    } else {
        items.push(probe_config("claude", "project", "claude-md", "CLAUDE.md", &dot_claude_md, pp.clone()));
    }

    items.push(probe_config(
        "claude", "project", "settings", "settings.json",
        &base.join(".claude").join("settings.json"), pp.clone(),
    ));

    let commands_dir = base.join(".claude").join("commands");
    if let Ok(entries) = std::fs::read_dir(&commands_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("md") {
                let fname = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                items.push(probe_config("claude", "project", "commands", &fname, &path, pp.clone()));
            }
        }
    }

    items
}

// ── Cursor project configs ──

fn scan_cursor_project_configs(project_path: &str) -> Vec<ConfigItem> {
    let base = Path::new(project_path);
    if !base.is_dir() {
        return Vec::new();
    }
    let mut items = Vec::new();
    let pp = Some(project_path.to_string());

    let cursorrules = base.join(".cursorrules");
    if cursorrules.is_file() {
        items.push(probe_config("cursor", "project", "rules", ".cursorrules", &cursorrules, pp.clone()));
    }

    let rules_dir = base.join(".cursor").join("rules");
    if let Ok(entries) = std::fs::read_dir(&rules_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("mdc") {
                let fname = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                items.push(probe_config("cursor", "project", "rules", &fname, &path, pp.clone()));
            }
        }
    }

    items.push(probe_config(
        "cursor", "project", "mcp", "mcp.json",
        &base.join(".cursor").join("mcp.json"), pp.clone(),
    ));

    collect_skills(&base.join(".cursor").join("skills"), "cursor", "project", pp.clone(), &mut items);
    collect_skills(&base.join(".agents").join("skills"), "cursor", "project", pp.clone(), &mut items);

    items
}

// ── Shared helpers ──

fn probe_config(
    source: &str, scope: &str, category: &str, name: &str,
    path: &Path, project_path: Option<String>,
) -> ConfigItem {
    let (status, size_bytes, last_modified_ms) = if path.is_file() {
        match std::fs::metadata(path) {
            Ok(meta) => {
                let size = meta.len();
                let mtime = meta.modified().ok().and_then(|t| {
                    t.duration_since(UNIX_EPOCH).ok().map(|d| d.as_millis() as i64)
                });
                if size == 0 {
                    ("missing".to_string(), Some(size), mtime)
                } else {
                    ("active".to_string(), Some(size), mtime)
                }
            }
            Err(_) => ("missing".to_string(), None, None),
        }
    } else {
        ("missing".to_string(), None, None)
    };

    let file_name = if category == "commands" || category == "rules" || category == "skills" {
        Some(name.to_string())
    } else {
        None
    };

    ConfigItem {
        id: make_id(source, scope, category, project_path.as_deref(), file_name.as_deref()),
        source: source.to_string(),
        name: name.to_string(),
        category: category.to_string(),
        scope: scope.to_string(),
        project_path,
        file_path: path.to_string_lossy().to_string(),
        status,
        size_bytes,
        last_modified_ms,
    }
}

fn collect_skills(
    skills_dir: &Path, source: &str, scope: &str,
    project_path: Option<String>, items: &mut Vec<ConfigItem>,
) {
    let Ok(entries) = std::fs::read_dir(skills_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let skill_path = entry.path().join("SKILL.md");
        if skill_path.is_file() {
            let dir_name = entry.file_name().to_string_lossy().to_string();
            let display_name = format!("{dir_name}/SKILL.md");
            items.push(probe_config(source, scope, "skills", &display_name, &skill_path, project_path.clone()));
        }
    }
}

fn has_hooks_in_settings(path: &Path) -> bool {
    let Ok(raw) = std::fs::read_to_string(path) else {
        return false;
    };
    let Ok(val) = serde_json::from_str::<serde_json::Value>(raw.trim()) else {
        return false;
    };
    matches!(val.get("hooks"), Some(serde_json::Value::Object(m)) if !m.is_empty())
}

fn make_id(source: &str, scope: &str, category: &str, project_path: Option<&str>, file_name: Option<&str>) -> String {
    let mut id = format!("{source}-{scope}-{category}");
    if let Some(pp) = project_path {
        let hash = &simple_hash(pp);
        id.push_str(&format!("-{hash}"));
    }
    if let Some(fname) = file_name {
        let clean: String = fname
            .replace('.', "-")
            .replace('/', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect();
        id.push_str(&format!("-{clean}"));
    }
    id
}

fn simple_hash(input: &str) -> String {
    let mut hash: u64 = 5381;
    for byte in input.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
    format!("{:06x}", hash & 0xFFFFFF)
}

fn detect_content_type(file_path: &str) -> String {
    if file_path.ends_with(".json") {
        "json".to_string()
    } else if file_path.ends_with(".md") || file_path.ends_with(".mdc") {
        "markdown".to_string()
    } else {
        "text".to_string()
    }
}
