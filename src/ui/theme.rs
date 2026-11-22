//! Color tokens for Light and Dark themes.

use gpui::{Hsla, rgb, rgba};

#[derive(Clone, Copy)]
pub struct ThemePalette {
    pub is_dark: bool,
    pub bg: Hsla,
    pub card: Hsla,
    pub text: Hsla,
    pub text_muted: Hsla,
    pub border: Hsla,
    pub brand: Hsla,
    pub brand_hover: Hsla,
    pub brand_tint: Hsla,
    pub subtle_bg: Hsla,
    pub subtle_border: Hsla,
    pub disabled_bg: Hsla,
    pub disabled_text: Hsla,
    pub error_bg: Hsla,
    pub error_border: Hsla,
    pub error_text: Hsla,
}

impl ThemePalette {
    pub fn for_dark(is_dark: bool) -> Self {
        if is_dark {
            Self {
                is_dark: true,
                bg: rgb(0x14181b).into(),
                card: rgb(0x1e2428).into(),
                text: rgb(0xe8ecea).into(),
                text_muted: rgb(0x9aa5a0).into(),
                border: rgb(0x2d353b).into(),
                brand: rgb(0x5cc489).into(),
                brand_hover: rgb(0x4fb379).into(),
                brand_tint: rgba(0x5cc48920).into(),
                subtle_bg: rgb(0x181d20).into(),
                subtle_border: rgb(0x2d353b).into(),
                disabled_bg: rgb(0x262d32).into(),
                disabled_text: rgb(0x6b7570).into(),
                error_bg: rgba(0x450a0a99).into(),
                error_border: rgb(0x7f1d1d).into(),
                error_text: rgb(0xfca5a5).into(),
            }
        } else {
            Self {
                is_dark: false,
                bg: rgb(0xf7f8f7).into(),
                card: rgb(0xffffff).into(),
                text: rgb(0x1f2421).into(),
                text_muted: rgb(0x6b7570).into(),
                border: rgb(0xe5e8e6).into(),
                brand: rgb(0x3fa66b).into(),
                brand_hover: rgb(0x2f8c57).into(),
                brand_tint: rgba(0x3fa66b18).into(),
                subtle_bg: rgb(0xf3f5f4).into(),
                subtle_border: rgb(0xe5e8e6).into(),
                disabled_bg: rgb(0xe5e7eb).into(),
                disabled_text: rgb(0x9ca3af).into(),
                error_bg: rgb(0xfef2f2).into(),
                error_border: rgb(0xfecaca).into(),
                error_text: rgb(0xb91c1c).into(),
            }
        }
    }
}
