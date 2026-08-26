use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use xxhash_rust::xxh3::Xxh3;

use crate::models::{DiffStatus, FileEntry, ScanProgressEvent, ScanResult, ScanSummary};
use crate::services::scanner::scan_directory_tree;

pub fn compare_directories<F>(
    source_path: &Path,
    target_path: &Path,
    progress_callback: F,
) -> Result<ScanResult, String>
where
    F: Fn(ScanProgressEvent) + Send + Sync + Clone + 'static,
{
    let start_time = std::time::Instant::now();

    let cb_src = progress_callback.clone();
    let cb_tgt = progress_callback.clone();

    // Varredura concorrente usando rayon para Origem e Destino
    let (source_map_res, target_map_res) = rayon::join(
        || scan_directory_tree(source_path, move |count, current| {
            cb_src(ScanProgressEvent {
                phase: "source".to_string(),
                count,
                current_file: if current.is_empty() { None } else { Some(current.to_string()) },
            });
        }),
        || scan_directory_tree(target_path, move |count, current| {
            cb_tgt(ScanProgressEvent {
                phase: "target".to_string(),
                count,
                current_file: if current.is_empty() { None } else { Some(current.to_string()) },
            });
        }),
    );

    let source_map = source_map_res?;
    let target_map = target_map_res?;

    let mut all_rel_paths: HashSet<String> = HashSet::new();
    for k in source_map.keys() {
        all_rel_paths.insert(k.clone());
    }
    for k in target_map.keys() {
        all_rel_paths.insert(k.clone());
    }

    let mut sorted_paths: Vec<String> = all_rel_paths.into_iter().collect();
    sorted_paths.sort();

    let mut files: Vec<FileEntry> = Vec::with_capacity(sorted_paths.len());
    let mut summary = ScanSummary::default();

    let total_paths = sorted_paths.len();
    for (index, rel_path) in sorted_paths.into_iter().enumerate() {
        if index % 500 == 0 || index == total_paths - 1 {
            progress_callback(ScanProgressEvent {
                phase: "comparison".to_string(),
                count: index + 1,
                current_file: Some(rel_path.clone()),
            });
        }
        let src_meta = source_map.get(&rel_path);
        let tgt_meta = target_map.get(&rel_path);

        match (src_meta, tgt_meta) {
            (Some(src), Some(tgt)) => {
                summary.total_src_bytes += src.size;
                summary.total_target_bytes += tgt.size;

                let size_diff = src.size as i64 - tgt.size as i64;
                let mtime_diff_secs = (src.mtime_millis - tgt.mtime_millis) / 1000;

                // Tolerância de 1.5s para diferenças entre sistemas de arquivos NTFS/FAT
                let mtime_diff_abs = (src.mtime_millis - tgt.mtime_millis).abs();
                let status = if src.size == tgt.size && mtime_diff_abs <= 1500 {
                    summary.equal_count += 1;
                    DiffStatus::Equal
                } else if src.mtime_millis > tgt.mtime_millis + 1500 {
                    summary.newer_count += 1;
                    summary.different_count += 1;
                    if size_diff > 0 {
                        summary.heavy_count += 1;
                    }
                    DiffStatus::NewerInSource
                } else if src.mtime_millis + 1500 < tgt.mtime_millis {
                    summary.older_count += 1;
                    summary.different_count += 1;
                    if size_diff > 0 {
                        summary.heavy_count += 1;
                    }
                    DiffStatus::OlderInSource
                } else if size_diff > 0 {
                    summary.heavy_count += 1;
                    summary.different_count += 1;
                    DiffStatus::HeavyInSource
                } else if size_diff < 0 {
                    summary.different_count += 1;
                    DiffStatus::LightInSource
                } else {
                    summary.different_count += 1;
                    DiffStatus::Different
                };

                // Pré-selecionar por padrão se for mais recente ou diferente na origem
                let should_select = matches!(status, DiffStatus::NewerInSource | DiffStatus::HeavyInSource);

                files.push(FileEntry {
                    rel_path,
                    src_size: Some(src.size),
                    target_size: Some(tgt.size),
                    src_mtime: Some(src.mtime_millis),
                    target_mtime: Some(tgt.mtime_millis),
                    src_hash: None,
                    target_hash: None,
                    status,
                    size_diff: Some(size_diff),
                    mtime_diff_secs: Some(mtime_diff_secs),
                    selected: should_select,
                });
            }
            (Some(src), None) => {
                summary.total_src_bytes += src.size;
                summary.only_source_count += 1;

                files.push(FileEntry {
                    rel_path,
                    src_size: Some(src.size),
                    target_size: None,
                    src_mtime: Some(src.mtime_millis),
                    target_mtime: None,
                    src_hash: None,
                    target_hash: None,
                    status: DiffStatus::OnlyInSource,
                    size_diff: Some(src.size as i64),
                    mtime_diff_secs: None,
                    selected: true, // Novo arquivo deve vir marcado
                });
            }
            (None, Some(tgt)) => {
                summary.total_target_bytes += tgt.size;
                summary.only_target_count += 1;

                files.push(FileEntry {
                    rel_path,
                    src_size: None,
                    target_size: Some(tgt.size),
                    src_mtime: None,
                    target_mtime: Some(tgt.mtime_millis),
                    src_hash: None,
                    target_hash: None,
                    status: DiffStatus::OnlyInTarget,
                    size_diff: Some(-(tgt.size as i64)),
                    mtime_diff_secs: None,
                    selected: false,
                });
            }
            (None, None) => {}
        }
    }

    summary.total_items = files.len();
    summary.scan_duration_ms = start_time.elapsed().as_millis();

    Ok(ScanResult {
        source_path: source_path.to_string_lossy().to_string(),
        target_path: target_path.to_string_lossy().to_string(),
        files,
        summary,
    })
}

pub fn calculate_file_hash(path: &Path) -> Result<String, String> {
    let file = File::open(path).map_err(|e| format!("Falha ao abrir '{}': {}", path.display(), e))?;
    let mut reader = BufReader::with_capacity(128 * 1024, file);
    let mut hasher = Xxh3::new();
    let mut buffer = [0u8; 64 * 1024];

    loop {
        let n = reader.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(format!("{:016x}", hasher.digest()))
}
