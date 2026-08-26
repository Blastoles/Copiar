use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

use crate::models::{CopyRequest, CopyResult};
use crate::services::copier::execute_copy_task;

pub struct AppCopyState {
    pub is_cancelled: Arc<AtomicBool>,
}

impl Default for AppCopyState {
    fn default() -> Self {
        Self {
            is_cancelled: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[tauri::command]
pub async fn start_copy(
    app: AppHandle,
    state: State<'_, AppCopyState>,
    request: CopyRequest,
) -> Result<CopyResult, String> {
    state.is_cancelled.store(false, Ordering::Relaxed);
    let cancel_flag = Arc::clone(&state.is_cancelled);

    let app_clone = app.clone();
    let result = execute_copy_task(request, cancel_flag, move |progress| {
        let _ = app_clone.emit("copy-progress", progress);
    })
    .await;

    result
}

#[tauri::command]
pub async fn cancel_current_copy(state: State<'_, AppCopyState>) -> Result<(), String> {
    state.is_cancelled.store(true, Ordering::Relaxed);
    Ok(())
}
