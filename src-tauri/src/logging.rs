use tracing_appender::rolling;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

/// Initialize file-based logging with daily rotation.
/// Logs are written to `~/.config/claude-code-launch/logs/`.
/// Retains 7 days of log files (via `max_log_files`).
/// Also outputs to stderr for development convenience.
pub fn init_logging() {
    let log_dir = resolve_log_dir();
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        eprintln!("failed to create log directory {}: {e}", log_dir.display());
        return;
    }

    let file_appender = rolling::Builder::new()
        .rotation(rolling::Rotation::DAILY)
        .filename_prefix("app")
        .filename_suffix("log")
        .max_log_files(7)
        .build(&log_dir)
        .expect("failed to create log file appender");

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_writer(file_appender)
                .with_ansi(false)
                .with_target(true),
        )
        .with(
            fmt::layer()
                .with_writer(std::io::stderr)
                .with_target(true),
        )
        .init();
}

fn resolve_log_dir() -> std::path::PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| "/tmp".to_string());
    std::path::PathBuf::from(home)
        .join(".config")
        .join("claude-code-launch")
        .join("logs")
}
