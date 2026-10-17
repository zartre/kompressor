use gpui::{rgb, Hsla, WindowAppearance};

/// A small set of semantic colors that adapt to the system's light/dark appearance.
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub background: Hsla,
    pub surface: Hsla,
    pub border: Hsla,
    pub text: Hsla,
    pub text_muted: Hsla,
    pub accent: Hsla,
    pub error: Hsla,
}

impl Theme {
    pub fn for_appearance(appearance: WindowAppearance) -> Self {
        match appearance {
            WindowAppearance::Dark | WindowAppearance::VibrantDark => Self::dark(),
            WindowAppearance::Light | WindowAppearance::VibrantLight => Self::light(),
        }
    }

    fn light() -> Self {
        Self {
            background: rgb(0xf5f5f5).into(),
            surface: rgb(0xffffff).into(),
            border: rgb(0xd7d7d7).into(),
            text: rgb(0x1a1a1a).into(),
            text_muted: rgb(0x767676).into(),
            accent: rgb(0x2f6fed).into(),
            error: rgb(0xc62828).into(),
        }
    }

    fn dark() -> Self {
        Self {
            background: rgb(0x1e1e1e).into(),
            surface: rgb(0x2a2a2a).into(),
            border: rgb(0x3f3f3f).into(),
            text: rgb(0xe8e8e8).into(),
            text_muted: rgb(0x9a9a9a).into(),
            accent: rgb(0x5b9bff).into(),
            error: rgb(0xff6b6b).into(),
        }
    }
}
