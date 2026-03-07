use std::{env, process::{Command, Output}};

use crate::{
    dao::command_exists,
    models::{PrereqItem, PrereqResult},
};

pub fn check_prereqs() -> PrereqResult {
    let platform = env::consts::OS.to_string();
    let (claude_installed, claude_version) = detect_claude_version();
    let mut items = Vec::new();

    items.push(PrereqItem {
        name: "npm".to_string(),
        available: command_exists("npm"),
        severity: "blocker".to_string(),
        message: if cfg!(target_os = "windows") {
            "需要可用的 npm 以执行全局安装（不可用时将自动安装 Node.js）".to_string()
        } else {
            "需要可用的 npm 以执行全局安装，请先安装 Node.js".to_string()
        },
    });

    if cfg!(target_os = "windows") {
        items.push(PrereqItem {
            name: "git_for_windows".to_string(),
            available: command_exists("git"),
            severity: "warning".to_string(),
            message: "建议安装 Git for Windows，Claude Code 在 Windows 依赖 Git Bash".to_string(),
        });
    }

    PrereqResult {
        platform,
        claude_installed,
        claude_version,
        items,
    }
}

fn detect_claude_version() -> (bool, Option<String>) {
    match run_claude_version() {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if stdout.is_empty() {
                (true, Some("已安装（无版本输出）".to_string()))
            } else {
                (true, Some(stdout))
            }
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if stderr.is_empty() {
                (false, None)
            } else {
                (false, Some(format!("版本检查失败: {}", stderr)))
            }
        }
        Err(_) => (false, None),
    }
}

fn run_claude_version() -> std::io::Result<Output> {
    let direct = Command::new("claude").arg("--version").output();
    if !cfg!(target_os = "windows") {
        return direct;
    }

    match direct {
        Ok(output) => Ok(output),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            Command::new("cmd")
                .arg("/C")
                .arg("claude --version")
                .output()
        }
        Err(err) => Err(err),
    }
}
