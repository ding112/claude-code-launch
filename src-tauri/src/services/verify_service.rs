use crate::{
    dao::{command_exists, run_command_with_streaming_logs_timeout},
    models::{LogEvent, VerifyResult},
};

pub fn run_verify<F>(emit_log: F) -> VerifyResult
where
    F: Fn(LogEvent),
{
    if !command_exists("claude") {
        return VerifyResult {
            success: false,
            version_output: String::new(),
            doctor_output: String::new(),
            error_summary: Some("未找到 claude 命令，请先完成安装并检查 PATH".to_string()),
        };
    }

    let version = run_command_with_streaming_logs_timeout(
        "verify",
        "info",
        "claude",
        &["--version"],
        Some(30),
        &emit_log,
    );
    let mut success = true;
    let mut error_summary = None;

    let version_output = match &version {
        Ok(run) => {
            if !run.success {
                success = false;
                let code = run
                    .exit_code
                    .map(|x| x.to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                error_summary = Some(format!("claude --version 执行失败，退出码: {}", code));
            }
            format_output(run.stdout.clone(), run.stderr.clone())
        }
        Err(err) => {
            success = false;
            error_summary = Some(format!(
                "claude --version {}",
                classify_verify_error(err.to_string())
            ));
            String::new()
        }
    };

    VerifyResult {
        success,
        version_output,
        doctor_output: String::new(),
        error_summary,
    }
}

fn format_output(stdout: String, stderr: String) -> String {
    if stderr.trim().is_empty() {
        stdout
    } else if stdout.trim().is_empty() {
        stderr
    } else {
        format!("{}\n{}", stdout, stderr)
    }
}

fn classify_verify_error(err: String) -> String {
    if err.contains("命令执行超时") {
        format!("执行超时: {}", err)
    } else if err.contains("启动命令失败") {
        format!("启动失败: {}", err)
    } else {
        format!("执行异常: {}", err)
    }
}
