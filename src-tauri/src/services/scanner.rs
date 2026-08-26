use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct RawFileMetadata {
    pub rel_path: String,
    pub full_path: PathBuf,
    pub size: u64,
    pub mtime_millis: i64,
}

#[cfg(windows)]
fn is_offline_file(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    // FILE_ATTRIBUTE_OFFLINE = 0x1000
    (metadata.file_attributes() & 0x1000) != 0
}

#[cfg(not(windows))]
fn is_offline_file(_metadata: &fs::Metadata) -> bool {
    false
}

pub fn scan_directory_tree<F>(base_dir: &Path, on_progress: F) -> Result<HashMap<String, RawFileMetadata>, String>
where
    F: Fn(usize, &str) + Send + Sync + Clone + 'static,
{
    if !base_dir.exists() {
        return Err(format!("Diretório '{}' não existe", base_dir.display()));
    }
    if !base_dir.is_dir() {
        return Err(format!("Caminho '{}' não é uma pasta", base_dir.display()));
    }

    let mut map = HashMap::new();
    let mut count = 0;

    for entry in WalkDir::new(base_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            if let Ok(metadata) = fs::metadata(path) {
                if is_offline_file(&metadata) {
                    continue;
                }
                if let Ok(rel) = path.strip_prefix(base_dir) {
                    let rel_str = rel.to_string_lossy().replace('\\', "/");
                    let mtime_millis = metadata
                        .modified()
                        .ok()
                        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                        .map(|d| d.as_millis() as i64)
                        .unwrap_or(0);

                    map.insert(
                        rel_str.clone(),
                        RawFileMetadata {
                            rel_path: rel_str.clone(),
                            full_path: path.to_path_buf(),
                            size: metadata.len(),
                            mtime_millis,
                        },
                    );

                    count += 1;
                    if count % 500 == 0 {
                        on_progress(count, &rel_str);
                    }
                }
            }
        }
    }

    on_progress(count, "");
    Ok(map)
}
