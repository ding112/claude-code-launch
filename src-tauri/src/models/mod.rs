use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEvent {
    pub step: String,
    pub level: String,
    pub message: String,
    pub raw: Option<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrereqItem {
    pub name: String,
    pub available: bool,
    pub severity: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrereqResult {
    pub platform: String,
    pub claude_installed: bool,
    pub claude_version: Option<String>,
    pub items: Vec<PrereqItem>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallAttempt {
    pub method: String,
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub error_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallResult {
    pub status: String,
    pub selected_method: Option<String>,
    pub summary: String,
    pub attempts: Vec<InstallAttempt>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyResult {
    pub success: bool,
    pub version_output: String,
    pub doctor_output: String,
    pub error_summary: Option<String>,
}
