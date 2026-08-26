pub mod commands;
pub mod models;
pub mod services;

use commands::copy::AppCopyState;
use commands::{cancel_current_copy, compute_file_hash, scan_folders, start_copy};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppCopyState::default())
        .invoke_handler(tauri::generate_handler![
            scan_folders,
            compute_file_hash,
            start_copy,
            cancel_current_copy
        ])
        .run(tauri::generate_context!())
        .expect("Erro ao executar aplicativo Tauri");
}
