use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::fs::{self, File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::models::{CopyOperationType, CopyProgressEvent, CopyRequest, CopyResult};

pub async fn execute_copy_task<F>(
    request: CopyRequest,
    is_cancelled: Arc<AtomicBool>,
    mut on_progress: F,
) -> Result<CopyResult, String>
where
    F: FnMut(CopyProgressEvent) + Send + 'static,
{
    let start_time = Instant::now();
    let src_base = PathBuf::from(&request.source_base);
    let tgt_base = PathBuf::from(&request.target_base);

    if !src_base.exists() {
        return Err(format!("Pasta de origem não existe: {}", src_base.display()));
    }
    if !tgt_base.exists() {
        fs::create_dir_all(&tgt_base)
            .await
            .map_err(|e| format!("Falha ao criar pasta de destino: {}", e))?;
    }

    // Calcular tamanho total a ser copiado
    let mut total_bytes_to_copy: u64 = 0;
    for rel in &request.files_to_copy {
        let src_file = src_base.join(rel);
        if let Ok(meta) = std::fs::metadata(&src_file) {
            total_bytes_to_copy += meta.len();
        }
    }

    let total_files = request.files_to_copy.len();
    let mut success_count = 0;
    let mut error_count = 0;
    let mut errors = Vec::new();
    let mut total_bytes_copied: u64 = 0;

    let mut last_emit = Instant::now();
    let mut last_bytes_count: u64 = 0;
    let mut current_speed: f64 = 0.0;

    for (idx, rel_path) in request.files_to_copy.iter().enumerate() {
        if is_cancelled.load(Ordering::Relaxed) {
            return Err("Operação cancelada pelo usuário".to_string());
        }

        let src_file = src_base.join(rel_path);
        let tgt_file = tgt_base.join(rel_path);

        if let Some(parent) = tgt_file.parent() {
            if !parent.exists() {
                let _ = fs::create_dir_all(parent).await;
            }
        }

        let file_meta = match std::fs::metadata(&src_file) {
            Ok(m) => m,
            Err(e) => {
                error_count += 1;
                errors.push(format!("Falha ao ler {}: {}", rel_path, e));
                continue;
            }
        };

        let file_size = file_meta.len();
        let mut bytes_copied_this_file: u64 = 0;

        let res: Result<(), String> = async {
            let mut reader = File::open(&src_file)
                .await
                .map_err(|e| format!("Abrir origem: {}", e))?;

            let mut writer = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&tgt_file)
                .await
                .map_err(|e| format!("Criar destino: {}", e))?;

            let mut buffer = vec![0u8; 64 * 1024];

            loop {
                if is_cancelled.load(Ordering::Relaxed) {
                    return Err("Cancelado".to_string());
                }

                let read_bytes = reader
                    .read(&mut buffer)
                    .await
                    .map_err(|e| format!("Erro de leitura: {}", e))?;

                if read_bytes == 0 {
                    break;
                }

                writer
                    .write_all(&buffer[..read_bytes])
                    .await
                    .map_err(|e| format!("Erro de escrita: {}", e))?;

                bytes_copied_this_file += read_bytes as u64;
                total_bytes_copied += read_bytes as u64;

                let now = Instant::now();
                let elapsed_since_last_emit = now.duration_since(last_emit).as_secs_f64();
                if elapsed_since_last_emit >= 0.15 {
                    let bytes_delta = total_bytes_copied.saturating_sub(last_bytes_count);
                    current_speed = (bytes_delta as f64) / elapsed_since_last_emit;
                    last_bytes_count = total_bytes_copied;
                    last_emit = now;

                    let pct = if total_bytes_to_copy > 0 {
                        (total_bytes_copied as f64 / total_bytes_to_copy as f64) * 100.0
                    } else {
                        100.0
                    };

                    on_progress(CopyProgressEvent {
                        current_file: rel_path.clone(),
                        file_index: idx + 1,
                        total_files,
                        bytes_copied_current_file: bytes_copied_this_file,
                        total_bytes_current_file: file_size,
                        total_bytes_copied,
                        total_bytes_to_copy,
                        speed_bytes_per_sec: current_speed,
                        percentage_total: pct,
                        is_finished: false,
                        has_error: false,
                        error_message: None,
                    });
                }
            }

            writer.flush().await.map_err(|e| format!("Flush destino: {}", e))?;
            drop(writer);
            drop(reader);

            // Preservar mtime
            if request.preserve_timestamps {
                if let Ok(mtime) = file_meta.modified() {
                    let ft = filetime::FileTime::from_system_time(mtime);
                    let _ = filetime::set_file_mtime(&tgt_file, ft);
                }
            }

            // Se for mover, remove origem
            if request.operation_type == CopyOperationType::Move {
                let _ = fs::remove_file(&src_file).await;
            }

            Ok(())
        }
        .await;

        match res {
            Ok(_) => {
                success_count += 1;
            }
            Err(e) => {
                if e == "Cancelado" {
                    return Err("Operação cancelada pelo usuário".to_string());
                }
                error_count += 1;
                errors.push(format!("{}: {}", rel_path, e));
            }
        }
    }

    let pct = 100.0;
    on_progress(CopyProgressEvent {
        current_file: "Concluído".to_string(),
        file_index: total_files,
        total_files,
        bytes_copied_current_file: 0,
        total_bytes_current_file: 0,
        total_bytes_copied,
        total_bytes_to_copy,
        speed_bytes_per_sec: 0.0,
        percentage_total: pct,
        is_finished: true,
        has_error: error_count > 0,
        error_message: if error_count > 0 {
            Some(format!("{} arquivos falharam", error_count))
        } else {
            None
        },
    });

    Ok(CopyResult {
        success_count,
        error_count,
        total_bytes: total_bytes_copied,
        duration_ms: start_time.elapsed().as_millis(),
        errors,
    })
}
