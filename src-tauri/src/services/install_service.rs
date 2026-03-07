use crate::{
    dao::{now_millis, run_command_with_streaming_logs_timeout},
    models::{InstallAttempt, InstallResult, LogEvent},
    services::node_install_service,
};

pub fn run_install<F>(emit_log: F) -> InstallResult
where
    F: Fn(LogEvent),
{
    let mut attempts = Vec::new();

    emit_log(LogEvent {
        step: "install".to_string(),
        level: "info".to_string(),
        message: "开始安装策略: npm".to_string(),
        raw: None,
        timestamp: now_millis(),
    });

    if let Some(npm_cmd) = node_install_service::ensure_npm_available(&emit_log) {
        let args = ["install", "-g", "@anthropic-ai/claude-code"];
        let result = run_command_with_streaming_logs_timeout(
            "install",
            "info",
            npm_cmd.as_str(),
            &args,
            Some(600),
            &emit_log,
        );
        attempts.push(to_attempt("npm", result));
        if attempts.last().is_some_and(|x| x.success) {
            return InstallResult {
                status: "success".to_string(),
                selected_method: Some("npm".to_string()),
                summary: "已通过 npm 全局安装完成安装。".to_string(),
                attempts,
            };
        }
    } else {
        attempts.push(InstallAttempt {
            method: "npm".to_string(),
            success: false,
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            error_summary: Some("npm 不可用且 Node.js 自动安装失败".to_string()),
        });
    }

    InstallResult {
        status: "failed".to_string(),
        selected_method: None,
        summary: "npm 安装失败，请复制日志进行排查。".to_string(),
        attempts,
    }
}

fn to_attempt(method: &str, result: Result<crate::dao::CommandRunResult, String>) -> InstallAttempt {
    match result {
        Ok(run) => InstallAttempt {
            method: method.to_string(),
            success: run.success,
            exit_code: run.exit_code,
            stdout: run.stdout,
            stderr: run.stderr,
            error_summary: if run.success {
                None
            } else {
                let code = run
                    .exit_code
                    .map(|x| x.to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                Some(format!("命令执行失败，退出码: {}", code))
            },
        },
        Err(err) => InstallAttempt {
            method: method.to_string(),
            success: false,
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            error_summary: Some(classify_install_error(err)),
        },
    }
}

fn classify_install_error(err: String) -> String {
    if err.contains("命令执行超时") {
        format!("命令执行超时: {}", err)
    } else if err.contains("启动命令失败") {
        format!("命令启动失败: {}", err)
    } else {
        err
    }
}
