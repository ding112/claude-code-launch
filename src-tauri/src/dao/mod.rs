use std::{
    io,
    io::{BufRead, BufReader, Write},
    path::Path,
    process::{Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use crate::models::LogEvent;

/// Windows GUI 应用启动时，进程继承的 PATH 可能缺少用户级条目。
/// 从注册表读取最新的用户/系统 PATH，并探测 fnm/nvm 等版本管理器的 Node.js 路径，合并到当前进程。
#[cfg(target_os = "windows")]
pub fn refresh_path_from_registry() {
    use std::env;

    fn read_reg_path(key: &str) -> Vec<String> {
        let output = Command::new("reg")
            .args(["query", key, "/v", "Path"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output();

        let output = match output {
            Ok(o) if o.status.success() => o,
            _ => return Vec::new(),
        };

        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            let trimmed = line.trim();
            if let Some(pos) = trimmed.find("REG_SZ").or_else(|| trimmed.find("REG_EXPAND_SZ")) {
                let after_type = &trimmed[pos..];
                if let Some(val_start) = after_type.find("    ") {
                    let val = after_type[val_start..].trim();
                    if !val.is_empty() {
                        return val.split(';').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                    }
                }
            }
        }
        Vec::new()
    }

    let sys_paths = read_reg_path(r"HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment");
    let user_paths = read_reg_path(r"HKCU\Environment");
    let node_manager_paths = detect_node_manager_paths();

    let current = env::var("PATH").unwrap_or_default();
    let current_set: std::collections::HashSet<String> = current
        .split(';')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    let mut new_entries = Vec::new();
    for p in sys_paths.iter().chain(user_paths.iter()).chain(node_manager_paths.iter()) {
        if !current_set.contains(&p.to_lowercase()) {
            new_entries.push(p.clone());
        }
    }

    if !new_entries.is_empty() {
        let merged = format!("{};{}", current, new_entries.join(";"));
        env::set_var("PATH", &merged);
    }
}

/// 探测 fnm / nvm-windows 管理的 Node.js 路径
#[cfg(target_os = "windows")]
fn detect_node_manager_paths() -> Vec<String> {
    use std::path::PathBuf;

    let mut paths = Vec::new();

    // 1. fnm: %APPDATA%\fnm\aliases\default (包含 node.exe, npm.cmd)
    if let Ok(appdata) = std::env::var("APPDATA") {
        let fnm_default = PathBuf::from(&appdata).join("fnm").join("aliases").join("default");
        if fnm_default.join("npm.cmd").exists() {
            paths.push(fnm_default.to_string_lossy().to_string());
        }
    }

    // 2. nvm-windows: %NVM_HOME% -> %NVM_SYMLINK% (通常是 C:\Program Files\nodejs)
    if let Ok(nvm_symlink) = std::env::var("NVM_SYMLINK") {
        let p = PathBuf::from(&nvm_symlink);
        if p.join("npm.cmd").exists() {
            paths.push(p.to_string_lossy().to_string());
        }
    }

    // 3. 标准 Node.js 安装路径
    if let Ok(pf) = std::env::var("ProgramFiles") {
        let nodejs = PathBuf::from(&pf).join("nodejs");
        if nodejs.join("npm.cmd").exists() {
            paths.push(nodejs.to_string_lossy().to_string());
        }
    }

    // 4. npm 全局安装目录 (%APPDATA%\npm)
    if let Ok(appdata) = std::env::var("APPDATA") {
        let npm_global = PathBuf::from(&appdata).join("npm");
        if npm_global.is_dir() {
            paths.push(npm_global.to_string_lossy().to_string());
        }
    }

    paths
}

#[cfg(not(target_os = "windows"))]
pub fn refresh_path_from_registry() {}

#[derive(Debug, Clone)]
pub struct CommandRunResult {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

pub fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub fn command_exists(cmd: &str) -> bool {
    let checker = if cfg!(target_os = "windows") {
        ("where", vec![cmd])
    } else {
        ("which", vec![cmd])
    };

    Command::new(checker.0)
        .args(checker.1)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

pub fn run_command_with_streaming_logs_timeout<F>(
    step: &str,
    level: &str,
    program: &str,
    args: &[&str],
    timeout_seconds: Option<u64>,
    emit_log: F,
) -> Result<CommandRunResult, String>
where
    F: Fn(LogEvent),
{
    let command_line = format!("{} {}", program, args.join(" "));
    emit_log(LogEvent {
        step: step.to_string(),
        level: level.to_string(),
        message: format!("执行命令: {}", command_line),
        raw: Some(command_line.clone()),
        timestamp: now_millis(),
    });

    let mut child = spawn_command(program, args)
        .map_err(|err| format!("启动命令失败: {}", err))?;

    let stdout = child.stdout.take().ok_or("无法读取 stdout")?;
    let stderr = child.stderr.take().ok_or("无法读取 stderr")?;

    let (tx, rx) = mpsc::channel::<(bool, String)>();
    let tx_out = tx.clone();
    let tx_err = tx.clone();

    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            let _ = tx_out.send((false, line));
        }
    });

    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().map_while(Result::ok) {
            let _ = tx_err.send((true, line));
        }
    });

    drop(tx);

    let mut stdout_lines = Vec::new();
    let mut stderr_lines = Vec::new();
    let started_at = Instant::now();

    loop {
        if let Some(seconds) = timeout_seconds {
            if started_at.elapsed() >= Duration::from_secs(seconds) {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("命令执行超时({}s): {}", seconds, command_line));
            }
        }

        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok((is_stderr, line)) => {
                if is_stderr {
                    stderr_lines.push(line.clone());
                    emit_stream_log(step, true, line, &emit_log);
                } else {
                    stdout_lines.push(line.clone());
                    emit_stream_log(step, false, line, &emit_log);
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if let Ok(Some(_)) = child.try_wait() {
                    while let Ok((is_stderr, line)) = rx.try_recv() {
                        if is_stderr {
                            stderr_lines.push(line.clone());
                            emit_stream_log(step, true, line, &emit_log);
                        } else {
                            stdout_lines.push(line.clone());
                            emit_stream_log(step, false, line, &emit_log);
                        }
                    }
                    break;
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    let status = child.wait().map_err(|err| format!("等待命令结束失败: {}", err))?;

    Ok(CommandRunResult {
        success: status.success(),
        exit_code: status.code(),
        stdout: stdout_lines.join("\n"),
        stderr: stderr_lines.join("\n"),
    })
}

fn spawn_command(program: &str, args: &[&str]) -> io::Result<std::process::Child> {
    let direct = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    if !cfg!(target_os = "windows") {
        return direct;
    }

    match direct {
        Ok(child) => Ok(child),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Command::new("cmd")
            .arg("/C")
            .arg(program)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn(),
        Err(err) => Err(err),
    }
}

fn emit_stream_log<F>(step: &str, is_stderr: bool, line: String, emit_log: &F)
where
    F: Fn(LogEvent),
{
    emit_log(LogEvent {
        step: step.to_string(),
        level: if is_stderr {
            "error".to_string()
        } else {
            "info".to_string()
        },
        message: sanitize_stream_line(&line),
        raw: Some(line),
        timestamp: now_millis(),
    });
}

fn sanitize_stream_line(input: &str) -> String {
    let no_carriage = input
        .split('\n')
        .map(|line| line.rsplit('\r').next().unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n");
    strip_ansi_and_control(&no_carriage)
}

fn strip_ansi_and_control(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            match chars.peek().copied() {
                // CSI: ESC [ ... final-byte
                Some('[') => {
                    let _ = chars.next();
                    while let Some(next) = chars.next() {
                        if ('@'..='~').contains(&next) {
                            break;
                        }
                    }
                }
                // OSC: ESC ] ... BEL/ST
                Some(']') => {
                    let _ = chars.next();
                    while let Some(next) = chars.next() {
                        if next == '\u{07}' {
                            break;
                        }
                        if next == '\u{1b}' && chars.peek().copied() == Some('\\') {
                            let _ = chars.next();
                            break;
                        }
                    }
                }
                _ => {}
            }
            continue;
        }

        if ch == '\n' || ch == '\t' || !ch.is_control() {
            output.push(ch);
        }
    }

    output
}

pub fn download_file<F>(url: &str, dest: &Path, step: &str, emit_log: &F) -> Result<(), String>
where
    F: Fn(LogEvent),
{
    emit_log(LogEvent {
        step: step.to_string(),
        level: "info".to_string(),
        message: format!("开始下载: {}", url),
        raw: None,
        timestamp: now_millis(),
    });

    let response = reqwest::blocking::get(url).map_err(|e| format!("HTTP 请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP 状态码: {}", response.status()));
    }

    let total_size = response.content_length();
    if let Some(size) = total_size {
        emit_log(LogEvent {
            step: step.to_string(),
            level: "info".to_string(),
            message: format!("文件大小: {:.1} MB", size as f64 / 1_048_576.0),
            raw: None,
            timestamp: now_millis(),
        });
    }

    let bytes = response.bytes().map_err(|e| format!("读取响应体失败: {}", e))?;

    let mut file =
        std::fs::File::create(dest).map_err(|e| format!("创建文件失败: {}", e))?;
    file.write_all(&bytes)
        .map_err(|e| format!("写入文件失败: {}", e))?;

    emit_log(LogEvent {
        step: step.to_string(),
        level: "info".to_string(),
        message: format!("下载完成: {}", dest.display()),
        raw: None,
        timestamp: now_millis(),
    });

    Ok(())
}
