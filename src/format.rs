use std::path::Path;

/// The image formats Kompressor can produce.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OutputFormat {
    Jpg,
    Png,
    WebP,
}

impl OutputFormat {
    /// All supported output formats, in the order they appear in the UI.
    pub const ALL: [OutputFormat; 3] = [OutputFormat::Jpg, OutputFormat::Png, OutputFormat::WebP];

    pub fn label(self) -> &'static str {
        match self {
            OutputFormat::Jpg => "JPG",
            OutputFormat::Png => "PNG",
            OutputFormat::WebP => "WEBP",
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            OutputFormat::Jpg => "jpg",
            OutputFormat::Png => "png",
            OutputFormat::WebP => "webp",
        }
    }

    /// Whether `path`'s extension already matches this output format, meaning
    /// compression should overwrite the file in place rather than converting it.
    pub fn matches_extension(self, path: &Path) -> bool {
        let Some(extension) = path.extension().and_then(|ext| ext.to_str()) else {
            return false;
        };
        let extension = extension.to_ascii_lowercase();
        match self {
            OutputFormat::Jpg => extension == "jpg" || extension == "jpeg",
            OutputFormat::Png => extension == "png",
            OutputFormat::WebP => extension == "webp",
        }
    }
}

/// The image formats Kompressor accepts as input. Notably, WEBP is not
/// supported as an input format, even though it can be produced as output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputFormat {
    Jpg,
    Png,
}

impl InputFormat {
    /// Determines the input format from a file path's extension, returning
    /// `None` for anything unsupported.
    pub fn from_path(path: &Path) -> Option<Self> {
        let extension = path.extension()?.to_str()?.to_ascii_lowercase();
        match extension.as_str() {
            "jpg" | "jpeg" => Some(InputFormat::Jpg),
            "png" => Some(InputFormat::Png),
            _ => None,
        }
    }
}
