use serde::Deserialize;
use std::env;
use std::path::{Path, PathBuf};

const DEFAULT_CONFIG_DIR: &str = ".config/claude-code-launch";
const DEFAULT_CONFIG_FILE: &str = "config.json";
const DEFAULT_DB_PATH: &str = "runtime/claude-code-launch.sqlite3";
const CONFIG_PATH_ENV: &str = "CLAUDE_CODE_LAUNCH_CONFIG_PATH";

#[derive(Debug, Deserialize)]
struct RawConfig {
    db_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub db_path: String,
}

fn expand_tilde(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        format!("{home}/{rest}")
    } else {
        path.to_string()
    }
}

pub fn resolve_config_path() -> PathBuf {
    if let Ok(explicit) = env::var(CONFIG_PATH_ENV) {
        return PathBuf::from(explicit);
    }
    let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    Path::new(&home)
        .join(DEFAULT_CONFIG_DIR)
        .join(DEFAULT_CONFIG_FILE)
}

pub fn load_app_config() -> Result<AppConfig, String> {
    load_from_path(&resolve_config_path())
}

fn default_config_json() -> String {
    format!("{{\n  \"db_path\": \"{DEFAULT_DB_PATH}\"\n}}\n")
}

fn load_from_path(config_path: &Path) -> Result<AppConfig, String> {
    if !config_path.exists() {
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                format!("无法创建配置目录 {}：{e}", parent.display())
            })?;
        }
        std::fs::write(config_path, default_config_json()).map_err(|e| {
            format!("无法写入默认配置文件 {}：{e}", config_path.display())
        })?;
        println!(
            "created default config at {}, db_path: {}",
            config_path.display(),
            DEFAULT_DB_PATH,
        );
        return Ok(AppConfig {
            db_path: DEFAULT_DB_PATH.to_string(),
        });
    }

    let content = std::fs::read_to_string(config_path).map_err(|error| {
        format!(
            "无法读取配置文件 {}。\n原因: {error}\n建议: 检查文件权限或删除后重新创建",
            config_path.display(),
        )
    })?;

    let raw: RawConfig = serde_json::from_str(&content).map_err(|error| {
        format!(
            "配置文件 {} JSON 解析失败。\n原因: {error}\n建议: 检查 JSON 格式，示例:\n{{\n  \"db_path\": \"/absolute/path/to/overseer.sqlite3\"\n}}",
            config_path.display(),
        )
    })?;

    let db_path = expand_tilde(&raw.db_path.unwrap_or_default());
    if db_path.trim().is_empty() {
        return Err(format!(
            "配置文件 {} 中 db_path 为空。\n建议: 设置 db_path 为有效的 SQLite 数据库路径，例如:\n{{\n  \"db_path\": \"/absolute/path/to/overseer.sqlite3\"\n}}",
            config_path.display(),
        ));
    }

    println!(
        "loaded config from {}: db_path={}",
        config_path.display(),
        db_path,
    );

    Ok(AppConfig { db_path })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(label: &str) -> PathBuf {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        std::env::temp_dir().join(format!("overseer-test-{label}-{nanos}"))
    }

    #[test]
    fn missing_config_file_creates_default_and_returns_it() {
        let dir = temp_dir("missing");
        let path = dir.join("config.json");
        assert!(!path.exists());

        let config = load_from_path(&path).expect("missing config should create default");
        assert_eq!(config.db_path, DEFAULT_DB_PATH);
        assert!(path.exists(), "config file should be created on disk");

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains(DEFAULT_DB_PATH));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn valid_config_file_returns_custom_db_path() {
        let dir = temp_dir("valid");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.json");
        fs::write(&path, r#"{ "db_path": "/tmp/my-overseer.sqlite3" }"#).unwrap();

        let config = load_from_path(&path).expect("valid config should parse");
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(config.db_path, "/tmp/my-overseer.sqlite3");
    }

    #[test]
    fn invalid_json_returns_error() {
        let dir = temp_dir("invalid");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.json");
        fs::write(&path, "NOT JSON").unwrap();

        let error = load_from_path(&path).expect_err("invalid json should fail");
        let _ = fs::remove_dir_all(&dir);
        assert!(error.contains("JSON 解析失败"), "got: {error}");
    }

    #[test]
    fn empty_db_path_returns_error() {
        let dir = temp_dir("empty");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.json");
        fs::write(&path, r#"{ "db_path": "" }"#).unwrap();

        let error = load_from_path(&path).expect_err("empty db_path should fail");
        let _ = fs::remove_dir_all(&dir);
        assert!(error.contains("db_path 为空"), "got: {error}");
    }

    #[test]
    fn tilde_in_db_path_is_expanded() {
        let dir = temp_dir("tilde");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.json");
        fs::write(&path, r#"{ "db_path": "~/Documents/overseer/overseer.sqlite3" }"#).unwrap();

        let config = load_from_path(&path).expect("tilde path should parse");
        let _ = fs::remove_dir_all(&dir);
        assert!(!config.db_path.contains('~'), "tilde should be expanded: {}", config.db_path);
        assert!(config.db_path.ends_with("/Documents/overseer/overseer.sqlite3"));
    }

    #[test]
    fn missing_db_path_field_returns_error() {
        let dir = temp_dir("no-field");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.json");
        fs::write(&path, r#"{}"#).unwrap();

        let error = load_from_path(&path).expect_err("missing db_path field should fail");
        let _ = fs::remove_dir_all(&dir);
        assert!(error.contains("db_path 为空"), "got: {error}");
    }
}
