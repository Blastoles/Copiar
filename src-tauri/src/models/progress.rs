use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CopyOperationType {
    Copy,
    Move,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyRequest {
    pub source_base: String,
    pub target_base: String,
    pub files_to_copy: Vec<String>,
    pub preserve_timestamps: bool,
    pub operation_type: CopyOperationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyProgressEvent {
    pub current_file: String,
    pub file_index: usize,
    pub total_files: usize,
    pub bytes_copied_current_file: u64,
    pub total_bytes_current_file: u64,
    pub total_bytes_copied: u64,
    pub total_bytes_to_copy: u64,
    pub speed_bytes_per_sec: f64,
    pub percentage_total: f64,
    pub is_finished: bool,
    pub has_error: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyResult {
    pub success_count: usize,
    pub error_count: usize,
    pub total_bytes: u64,
    pub duration_ms: u128,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgressEvent {
    pub phase: String, // "source", "target", "comparison"
    pub count: usize,
    pub current_file: Option<String>,
}
