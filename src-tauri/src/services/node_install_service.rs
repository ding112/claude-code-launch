use crate::{
    dao::{command_exists, download_file, now_millis, run_command_with_streaming_logs_timeout},
    models::LogEvent,
};

const NODE_MSI_URL: &str = "https://nodejs.org/dist/v22.14.0/node-v22.14.0-x64.msi";
const NPM_MIRROR: &str = "https://registry.npmmirror.com";
const NODEJS_DEFAULT_DIR: &str = r"C:\Program Files\nodejs";

/// 确保 npm 可用。不可用时自动下载 Node.js MSI 并静默安装。
/// 安装成功后设置国内镜像。返回可用的 npm 命令路径（"npm" 或绝对路径）。
pub fn ensure_npm_available<F>(emit_log: &F) -> Option<String>
where
    F: Fn(LogEvent),
{
    if command_exists("npm") {
        emit_log(LogEvent {
            step: "install".to_string(),
            level: "info".to_string(),
            message: "npm 已存在，跳过 Node.js 安装".to_string(),
            raw: None,
            timestamp: now_millis(),
        });
        set_npm_mirror("npm", emit_log);
        return Some("npm".to_string());
    }

    if !cfg!(target_os = "windows") {
        emit_log(LogEvent {
            step: "install".to_string(),
            level: "error".to_string(),
            message: "未检测到 npm，请先手动安装 Node.js（https://nodejs.org）".to_string(),
            raw: None,
            timestamp: now_millis(),
        });
        return None;
    }

    emit_log(LogEvent {
        step: "install".to_string(),
        level: "info".to_string(),
        message: "未检测到 npm，开始自动安装 Node.js ...".to_string(),
        raw: None,
        timestamp: now_millis(),
    });

    if let Err(e) = install_nodejs_msi(emit_log) {
        emit_log(LogEvent {
            step: "install".to_string(),
            level: "error".to_string(),
            message: format!("Node.js 安装失败: {}", e),
            raw: None,
            timestamp: now_millis(),
        });
        return None;
    }

    refresh_path(emit_log);

    let npm_cmd = resolve_npm_cmd();
    if npm_cmd.is_none() {
        emit_log(LogEvent {
            step: "install".to_string(),
            level: "error".to_string(),
            message: "Node.js 安装完成但未找到 npm，请检查安装路径".to_string(),
            raw: None,
            timestamp: now_millis(),
        });
        return None;
    }

    let npm = npm_cmd.unwrap();
    emit_log(LogEvent {
        step: "install".to_string(),
        level: "info".to_string(),
        message: format!("npm 已就绪: {}", npm),
        raw: None,
        timestamp: now_millis(),
    });

    set_npm_mirror(&npm, emit_log);
    Some(npm)
}

fn install_nodejs_msi<F>(emit_log: &F) -> Result<(), String>
where
    F: Fn(LogEvent),
{
    let tmp_dir = tempfile::tempdir().map_err(|e| format!("创建临时目录失败: {}", e))?;
    let msi_path = tmp_dir.path().join("node-setup.msi");

    download_file(NODE_MSI_URL, &msi_path, "install", emit_log)?;

    emit_log(LogEvent {
        step: "install".to_string(),
        level: "info".to_string(),
        message: "开始静默安装 Node.js（msiexec /i ... /qn）...".to_string(),
        raw: None,
        timestamp: now_millis(),
    });

    let msi_str = msi_path.to_string_lossy().to_string();
    let args = ["/i", msi_str.as_str(), "/qn", "/norestart"];
    let result =
        run_command_with_streaming_logs_timeout("install", "info", "msiexec", &args, Some(300), emit_log);

    match result {
        Ok(run) if run.success => {
            emit_log(LogEvent {
                step: "install".to_string(),
                level: "info".to_string(),
                message: "Node.js MSI 安装完成".to_string(),
                raw: None,
                timestamp: now_millis(),
            });
            Ok(())
        }
        Ok(run) => {
            let code = run.exit_code.map(|c| c.to_string()).unwrap_or_else(|| "unknown".to_string());
            Err(format!(
                "msiexec 退出码: {}，stderr: {}",
                code,
                run.stderr.chars().take(500).collect::<String>()
            ))
        }
        Err(e) => Err(e),
    }
}

/// MSI 安装后当前进程的 PATH 不会自动更新，
/// 从注册表读取系统 PATH 并合并到当前进程环境变量。
fn refresh_path<F>(emit_log: &F)
where
    F: Fn(LogEvent),
{
    emit_log(LogEvent {
        step: "install".to_string(),
        level: "info".to_string(),
        message: "刷新 PATH 环境变量 ...".to_string(),
        raw: None,
        timestamp: now_millis(),
    });

    let output = std::process::Command::new("reg")
        .args([
            "query",
            r"HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment",
            "/v",
            "Path",
        ])
        .output();

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        // reg query 输出格式: "    Path    REG_EXPAND_SZ    <value>"
        if let Some(line) = stdout.lines().find(|l| l.contains("REG_EXPAND_SZ") || l.contains("REG_SZ")) {
            let parts: Vec<&str> = line.splitn(3, "    ").collect();
            if let Some(sys_path) = parts.last() {
                let sys_path = sys_path.trim();
                let current = std::env::var("PATH").unwrap_or_default();
                let merged = format!("{};{}", sys_path, current);
                unsafe { std::env::set_var("PATH", &merged); }
                emit_log(LogEvent {
                    step: "install".to_string(),
                    level: "info".to_string(),
                    message: "PATH 已刷新".to_string(),
                    raw: None,
                    timestamp: now_millis(),
                });
                return;
            }
        }
    }

    // 注册表读取失败时，直接追加默认 Node.js 路径
    let current = std::env::var("PATH").unwrap_or_default();
    let merged = format!("{};{}", NODEJS_DEFAULT_DIR, current);
    unsafe { std::env::set_var("PATH", &merged); }
    emit_log(LogEvent {
        step: "install".to_string(),
        level: "warn".to_string(),
        message: format!("注册表读取失败，已将 {} 追加到 PATH", NODEJS_DEFAULT_DIR),
        raw: None,
        timestamp: now_millis(),
    });
}

/// 优先使用 PATH 中的 npm，找不到则尝试默认安装路径。
fn resolve_npm_cmd() -> Option<String> {
    if command_exists("npm") {
        return Some("npm".to_string());
    }

    let npm_cmd = std::path::Path::new(NODEJS_DEFAULT_DIR).join("npm.cmd");
    if npm_cmd.exists() {
        return Some(npm_cmd.to_string_lossy().to_string());
    }

    None
}

fn set_npm_mirror<F>(npm_cmd: &str, emit_log: &F)
where
    F: Fn(LogEvent),
{
    emit_log(LogEvent {
        step: "install".to_string(),
        level: "info".to_string(),
        message: format!("设置 npm 镜像: {}", NPM_MIRROR),
        raw: None,
        timestamp: now_millis(),
    });

    let args = ["config", "set", "registry", NPM_MIRROR];
    let result = run_command_with_streaming_logs_timeout("install", "info", npm_cmd, &args, Some(30), emit_log);

    match result {
        Ok(run) if run.success => {
            emit_log(LogEvent {
                step: "install".to_string(),
                level: "info".to_string(),
                message: "npm 镜像设置成功".to_string(),
                raw: None,
                timestamp: now_millis(),
            });
        }
        _ => {
            emit_log(LogEvent {
                step: "install".to_string(),
                level: "warn".to_string(),
                message: "npm 镜像设置失败，将使用默认源".to_string(),
                raw: None,
                timestamp: now_millis(),
            });
        }
    }
}
