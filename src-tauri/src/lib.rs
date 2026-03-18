use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

mod commands;
mod dao;
mod models;
mod services;

mod app_config;
mod collection;
mod evaluation;
mod logging;
mod overseer_models;

fn http_server_addr() -> SocketAddr {
    let port = env::var("CLAUDE_CODE_LAUNCH_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .filter(|port| *port > 0)
        .unwrap_or(8787);

    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port)
}

fn absolute_path(path: &str) -> String {
    let p = std::path::Path::new(path);
    if p.is_absolute() {
        path.to_string()
    } else {
        env::current_dir()
            .map(|cwd| cwd.join(p).display().to_string())
            .unwrap_or_else(|_| path.to_string())
    }
}

async fn run_http_server(addr: SocketAddr, db_path: String, event_enabled: bool) -> Result<(), std::io::Error> {
    println!("sqlite db path: {}", absolute_path(&db_path));
    println!("http server listening on http://{addr}");
    collection::serve(addr, db_path, event_enabled).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    logging::init_logging();
    dao::refresh_path_from_registry();

    let server_addr = http_server_addr();
    let app_cfg = app_config::load_app_config().unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(1);
    });
    let db_path = app_cfg.db_path;
    let event_enabled = app_cfg.event_enabled;
    if let Some(parent) = std::path::Path::new(&db_path).parent() {
        if let Err(error) = std::fs::create_dir_all(parent) {
            eprintln!("failed to create sqlite parent dir for {db_path}: {error:?}");
        }
    }

    tauri::async_runtime::spawn(async move {
        if let Err(error) = run_http_server(server_addr, db_path, event_enabled).await {
            eprintln!("failed to start http server: {error:?}");
        }
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::check_prereqs,
            commands::run_install,
            commands::run_verify,
            commands::append_log
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
