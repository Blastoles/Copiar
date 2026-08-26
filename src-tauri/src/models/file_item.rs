use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DiffStatus {
    Equal,
    Different,
    NewerInSource,
    OlderInSource,
    HeavyInSource,
    LightInSource,
    OnlyInSource,
    OnlyInTarget,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub rel_path: String,
    pub src_size: Option<u64>,
    pub target_size: Option<u64>,
    pub src_mtime: Option<i64>,
    pub target_mtime: Option<i64>,
    pub src_hash: Option<String>,
    pub target_hash: Option<String>,
    pub status: DiffStatus,
    pub size_diff: Option<i64>,
    pub mtime_diff_secs: Option<i64>,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScanSummary {
    pub total_items: usize,
    pub equal_count: usize,
    pub different_count: usize,
    pub newer_count: usize,
    pub older_count: usize,
    pub heavy_count: usize,
    pub only_source_count: usize,
    pub only_target_count: usize,
    pub total_src_bytes: u64,
    pub total_target_bytes: u64,
    pub scan_duration_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub source_path: String,
    pub target_path: String,
    pub files: Vec<FileEntry>,
    pub summary: ScanSummary,
}
