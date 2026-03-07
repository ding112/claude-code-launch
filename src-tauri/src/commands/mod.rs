use tauri::{AppHandle, Emitter};

use crate::{
    dao::now_millis,
    models::{InstallResult, LogEvent, PrereqResult, VerifyResult},
    services::{install_service, prereq_service, verify_service},
};

#[tauri::command]
pub async fn check_prereqs() -> PrereqResult {
    tauri::async_runtime::spawn_blocking(prereq_service::check_prereqs)
        .await
        .unwrap_or_else(|_| PrereqResult {
            platform: std::env::consts::OS.to_string(),
            claude_installed: false,
            claude_version: None,
            items: vec![],
        })
}

#[tauri::command]
pub async fn run_install(app: AppHandle) -> InstallResult {
    tauri::async_runtime::spawn_blocking(move || {
        install_service::run_install(|event| {
            let _ = app.emit("launch-log", event);
        })
    })
    .await
    .unwrap_or_else(|err| InstallResult {
        status: "failed".to_string(),
        selected_method: None,
        summary: format!("安装任务异常中断: {}", err),
        attempts: Vec::new(),
    })
}

#[tauri::command]
pub async fn run_verify(app: AppHandle) -> VerifyResult {
    tauri::async_runtime::spawn_blocking(move || {
        verify_service::run_verify(|event| {
            let _ = app.emit("launch-log", event);
        })
    })
    .await
    .unwrap_or_else(|err| VerifyResult {
        success: false,
        version_output: String::new(),
        doctor_output: String::new(),
        error_summary: Some(format!("验证任务异常中断: {}", err)),
    })
}

#[tauri::command]
pub fn append_log(app: AppHandle, step: String, level: String, message: String) {
    let event = LogEvent {
        step,
        level,
        message: message.clone(),
        raw: Some(message),
        timestamp: now_millis(),
    };
    let _ = app.emit("launch-log", event);
}
