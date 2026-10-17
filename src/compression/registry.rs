use std::collections::HashMap;

use crate::format::OutputFormat;

use super::caesium::CaesiumCompressor;
use super::Compressor;

/// Maps each output format to the compression library responsible for
/// producing it. All formats currently use `libcaesium`, but this
/// indirection means a new library only has to be plugged in here to take
/// over a given format (or to add support for a new one).
pub struct CompressorRegistry {
    backends: HashMap<OutputFormat, fn() -> Box<dyn Compressor>>,
}

impl CompressorRegistry {
    pub fn new() -> Self {
        let mut backends: HashMap<OutputFormat, fn() -> Box<dyn Compressor>> = HashMap::new();
        for format in OutputFormat::ALL {
            backends.insert(format, || Box::new(CaesiumCompressor));
        }
        Self { backends }
    }

    /// Returns the compressor backend registered for `format`.
    ///
    /// # Panics
    ///
    /// Panics if `format` has no registered backend, which would indicate a
    /// bug since every variant of [`OutputFormat`] is registered in [`new`](Self::new).
    pub fn compressor_for(&self, format: OutputFormat) -> Box<dyn Compressor> {
        (self.backends[&format])()
    }
}
