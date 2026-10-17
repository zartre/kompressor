use std::path::PathBuf;

use crate::format::OutputFormat;

/// The state of a single image's compression as it progresses.
#[derive(Debug, Clone)]
pub enum ProcessingStatus {
    Processing,
    Done { output_size: u64 },
    Error(String),
}

/// A single image dropped into the app, tracked through its compression lifecycle.
#[derive(Debug, Clone)]
pub struct ImageItem {
    pub id: usize,
    pub path: PathBuf,
    pub file_name: String,
    pub original_size: u64,
    /// The output format selected at the time this image was dropped. Later
    /// changes to the app's output format selection do not affect this item.
    pub output_format: OutputFormat,
    /// The quality selected at the time this image was dropped.
    pub quality: u8,
    pub status: ProcessingStatus,
}

/// Formats a byte count as a short, human-readable string, e.g. "1.2 MB".
pub fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} {}", UNITS[unit])
    } else {
        format!("{size:.1} {}", UNITS[unit])
    }
}
