//! Path and filesystem helpers

use std::path::Path;

/// Recursively compute total size of a directory in bytes. Returns `None` on permission or I/O error.
pub fn dir_size(path: &Path) -> Option<u64> {
    if !path.is_dir() {
        return Some(0);
    }
    let mut total = 0u64;
    let entries = std::fs::read_dir(path).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            total += dir_size(&path)?;
        } else {
            total += entry.metadata().ok().map(|m| m.len()).unwrap_or(0);
        }
    }
    Some(total)
}

/// Format byte count as human-readable string (e.g. 1_048_576 -> "1.0 MB").
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
