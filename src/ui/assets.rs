//! Embedded SVG icon assets for the GPUI user interface.

use gpui::{AssetSource, SharedString};
use std::borrow::Cow;

const USB_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#000000" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 5v14"/><path d="M12 16.5 6.5 13v-2.5"/><path d="M12 14.5 17.5 11V8.5"/><polygon points="12,2 8.8,6 15.2,6" fill="#000000" stroke="none"/><circle cx="12" cy="20" r="2.2" fill="#000000" stroke="none"/><circle cx="6.5" cy="10" r="2" fill="#000000" stroke="none"/><rect x="15.6" y="6.6" width="3.8" height="3.8" rx="0.6" fill="#000000" stroke="none"/></svg>"##;

const WIFI_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#000000" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12.55a11 11 0 0 1 14.08 0"/><path d="M1.42 9a16 16 0 0 1 21.16 0"/><path d="M8.53 16.11a6 6 0 0 1 6.95 0"/><circle cx="12" cy="20" r="1.5" fill="#000000" stroke="none"/></svg>"##;

const SETTINGS_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#000000" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>"##;

const PLAY_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="#000000"><path d="M8 5v14l11-7z"/></svg>"##;

const MOON_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="#000000"><path d="M12 3a9 9 0 1 0 9 9c0-.46-.04-.92-.1-1.36a5.389 5.389 0 0 1-4.4 2.26 5.403 5.403 0 0 1-3.14-9.8c-.44-.06-.9-.1-1.36-.1z"/></svg>"##;

const ERROR_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="#000000"><path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd"/></svg>"##;

pub struct AppAssets;

impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        let content = match path {
            "icons/usb.svg" => Some(USB_SVG.as_bytes()),
            "icons/wifi.svg" => Some(WIFI_SVG.as_bytes()),
            "icons/settings.svg" => Some(SETTINGS_SVG.as_bytes()),
            "icons/play.svg" => Some(PLAY_SVG.as_bytes()),
            "icons/moon.svg" => Some(MOON_SVG.as_bytes()),
            "icons/error.svg" => Some(ERROR_SVG.as_bytes()),
            _ => None,
        };
        Ok(content.map(Cow::Borrowed))
    }

    fn list(&self, _path: &str) -> gpui::Result<Vec<SharedString>> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_svg_assets_load_successfully() {
        let assets = AppAssets;
        for path in [
            "icons/usb.svg",
            "icons/wifi.svg",
            "icons/settings.svg",
            "icons/play.svg",
            "icons/moon.svg",
            "icons/error.svg",
        ] {
            let loaded = assets.load(path).unwrap();
            assert!(loaded.is_some(), "missing asset: {path}");
        }
        assert!(assets.load("icons/nonexistent.svg").unwrap().is_none());
    }
}
