//! Status pill badge showing the current connection mode, IP address, and online state.

use crate::connection::ConnectionMode;
use crate::i18n::I18n;
use crate::ui::theme::ThemePalette;
use gpui::{AnyElement, FontWeight, IntoElement, ParentElement, Styled, div, px, svg};

pub fn render_connection_chip(
    mode: Option<ConnectionMode>,
    ip: &str,
    connected: bool,
    i18n: &I18n,
    palette: ThemePalette,
) -> AnyElement {
    let chip_label = if connected {
        if mode == Some(ConnectionMode::Wireless) {
            if !ip.is_empty() {
                i18n.chip_wireless_connected_ip(ip)
            } else {
                i18n.chip_wireless_connected().to_string()
            }
        } else {
            i18n.chip_wired_connected().to_string()
        }
    } else {
        i18n.chip_disconnected().to_string()
    };

    let icon_path = if mode == Some(ConnectionMode::Wireless) {
        "icons/wifi.svg"
    } else {
        "icons/usb.svg"
    };

    let dot_color = if connected {
        palette.brand
    } else {
        palette.disabled_text
    };

    let status_dot = div().w(px(8.0)).h(px(8.0)).rounded_full().bg(dot_color);

    let transport_icon = svg()
        .path(icon_path)
        .size(px(14.0))
        .text_color(palette.text_muted);

    let label_text = div()
        .text_size(px(12.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(palette.text)
        .child(chip_label);

    div()
        .px_3()
        .py(px(6.0))
        .rounded_full()
        .bg(palette.subtle_bg)
        .border_1()
        .border_color(palette.border)
        .flex()
        .items_center()
        .gap_2()
        .child(status_dot)
        .child(transport_icon)
        .child(label_text)
        .into_any_element()
}
