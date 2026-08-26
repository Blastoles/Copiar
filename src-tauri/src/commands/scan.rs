use std::path::PathBuf;
use tauri::{AppHandle, Emitter};

use crate::models::ScanResult;
use crate::services::diff::{calculate_file_hash, compare_directories};

#[tauri::command]
pub async fn scan_folders(
    app: AppHandle,
    source_path: String,
    target_path: String,
) -> Result<ScanResult, String> {
    tokio::task::spawn_blocking(move || {
        let src = PathBuf::from(&source_path);
        let tgt = PathBuf::from(&target_path);
        let app_clone = app.clone();
        compare_directories(&src, &tgt, move |progress| {
            let _ = app_clone.emit("scan-progress", progress);
        })
    })
    .await
    .map_err(|e| format!("Falha na thread de varredura: {}", e))?
}

#[tauri::command]
pub async fn compute_file_hash(file_path: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let p = PathBuf::from(&file_path);
        calculate_file_hash(&p)
    })
    .await
    .map_err(|e| format!("Falha no cálculo do hash: {}", e))?
}
