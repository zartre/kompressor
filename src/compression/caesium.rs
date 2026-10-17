use std::path::Path;

use caesium::parameters::CSParameters;
use caesium::SupportedFileTypes;

use crate::format::OutputFormat;

use super::{CompressionError, Compressor};

/// Compresses and converts images using the `libcaesium` library.
pub struct CaesiumCompressor;

impl Compressor for CaesiumCompressor {
    fn compress(
        &self,
        input: &Path,
        output: &Path,
        format: OutputFormat,
        quality: u8,
    ) -> Result<(), CompressionError> {
        let mut parameters = CSParameters::new();
        parameters.jpeg.quality = quality as u32;
        parameters.png.quality = quality as u32;
        parameters.webp.quality = quality as u32;

        let input_path = path_to_string(input)?;
        let output_path = path_to_string(output)?;

        let result = if format.matches_extension(input) {
            caesium::compress(input_path, output_path, &parameters)
        } else {
            caesium::convert(
                input_path,
                output_path,
                &parameters,
                to_caesium_format(format),
            )
        };

        result.map_err(|err| CompressionError(err.to_string()))
    }
}

fn to_caesium_format(format: OutputFormat) -> SupportedFileTypes {
    match format {
        OutputFormat::Jpg => SupportedFileTypes::Jpeg,
        OutputFormat::Png => SupportedFileTypes::Png,
        OutputFormat::WebP => SupportedFileTypes::WebP,
    }
}

fn path_to_string(path: &Path) -> Result<String, CompressionError> {
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| CompressionError(format!("Invalid path: {}", path.display())))
}
