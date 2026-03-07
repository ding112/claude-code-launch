mod commands;
mod dao;
mod models;
mod services;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dao::refresh_path_from_registry();

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
