//! Application header bar with the settings trigger button.

use crate::ui::theme::ThemePalette;
use gpui::{
    AnyElement, App, ClickEvent, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, Window, div, px, rgba, svg,
};

pub fn render_app_header(
    palette: ThemePalette,
    on_open_settings: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> AnyElement {
    let hover_bg = if palette.is_dark {
        rgba(0xffffff12)
    } else {
        rgba(0x0000000d)
    };

    let gear_icon = svg()
        .path("icons/settings.svg")
        .size(px(20.0))
        .text_color(palette.text_muted);

    let settings_btn = div()
        .id("btn-open-settings")
        .p(px(6.0))
        .rounded_lg()
        .cursor_pointer()
        .hover(move |s| s.bg(hover_bg))
        .on_click(on_open_settings)
        .child(gear_icon);

    div()
        .w_full()
        .flex()
        .justify_end()
        .items_center()
        .px_4()
        .pt_3()
        .child(settings_btn)
        .into_any_element()
}
