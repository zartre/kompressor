pub mod caesium;
pub mod registry;

use std::fmt;
use std::path::{Path, PathBuf};

use crate::format::OutputFormat;

/// An error produced while compressing or converting an image.
#[derive(Debug, Clone)]
pub struct CompressionError(pub String);

impl fmt::Display for CompressionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for CompressionError {}

/// A wrapper around a compression backend. Different libraries tend to
/// support only a handful of formats each, so each output format is handled
/// by whichever backend implements this trait for it (see
/// [`registry::CompressorRegistry`]).
pub trait Compressor: Send + Sync {
    /// Compresses (or converts) `input` into `output` for the given `format`,
    /// at the given `quality` (0-100).
    fn compress(
        &self,
        input: &Path,
        output: &Path,
        format: OutputFormat,
        quality: u8,
    ) -> Result<(), CompressionError>;
}

/// Determines where the output of compressing `input` to `format` should be
/// written. If `format` matches the input file's existing extension, the
/// input is overwritten in place; otherwise a sibling file with the new
/// extension is produced.
pub fn output_path(input: &Path, format: OutputFormat) -> PathBuf {
    if format.matches_extension(input) {
        input.to_path_buf()
    } else {
        input.with_extension(format.extension())
    }
}
