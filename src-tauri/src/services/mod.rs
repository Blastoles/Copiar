pub mod copier;
pub mod diff;
pub mod scanner;

pub use copier::*;
pub use diff::*;
pub use scanner::*;

#[cfg(windows)]
pub fn adjust_long_path(path: &std::path::Path) -> std::path::PathBuf {
    let path_str = path.to_string_lossy().replace("/", "\\");
    if path_str.starts_with(r"\\?\") {
        std::path::PathBuf::from(path_str)
    } else if path_str.starts_with(r"\\") {
        std::path::PathBuf::from(format!(r"\\?\UNC\{}", &path_str[2..]))
    } else {
        std::path::PathBuf::from(format!(r"\\?\{}", path_str))
    }
}

#[cfg(not(windows))]
pub fn adjust_long_path(path: &std::path::Path) -> std::path::PathBuf {
    path.to_path_buf()
}
