//! Diagnostic error card showing localized error title, hints, and collapsible technical details.

use crate::error::AppError;
use crate::i18n::I18n;
use crate::ui::theme::ThemePalette;
use gpui::{
    AnyElement, App, ClickEvent, FontWeight, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, Window, div, prelude::*, px, rgba, svg,
};

pub fn render_error_card(
    error: Option<&AppError>,
    fallback_message: Option<String>,
    show_detail: bool,
    i18n: &I18n,
    palette: ThemePalette,
    on_toggle_detail: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> AnyElement {
    let (title, hints, detail) = if let Some(err) = error {
        (
            i18n.error_title(err.code).to_string(),
            i18n.error_hints(err.code),
            err.detail.clone(),
        )
    } else if let Some(msg) = fallback_message {
        (msg, &[][..], None)
    } else {
        return div().into_any_element();
    };

    let header_row = div()
        .flex()
        .items_center()
        .gap_2()
        .text_size(px(13.0))
        .font_weight(FontWeight::SEMIBOLD)
        .child(
            svg()
                .path("icons/error.svg")
                .size(px(16.0))
                .text_color(palette.error_text),
        )
        .child(div().flex_1().child(title));

    let hints_list = div()
        .flex()
        .flex_col()
        .gap_1()
        .pl_2()
        .children(hints.iter().map(|hint| {
            div()
                .text_size(px(11.5))
                .line_height(px(16.0))
                .child(format!("• {hint}"))
        }));

    let toggle_label = format!(
        "{} {}",
        i18n.details_toggle(),
        if show_detail { "▲" } else { "▼" }
    );

    div()
        .w_full()
        .flex()
        .flex_col()
        .gap_2()
        .rounded_lg()
        .p_3()
        .bg(palette.error_bg)
        .border_1()
        .border_color(palette.error_border)
        .text_color(palette.error_text)
        .child(header_row)
        .when(!hints.is_empty(), |el| el.child(hints_list))
        .when_some(detail.filter(|d| !d.is_empty()), |el, detail_text| {
            let toggle_btn = div()
                .id("btn-toggle-error-detail")
                .cursor_pointer()
                .text_size(px(11.0))
                .underline()
                .on_click(on_toggle_detail)
                .child(toggle_label);

            let detail_box = div()
                .p(px(6.0))
                .rounded_sm()
                .bg(rgba(0x00000033))
                .text_size(px(10.0))
                .child(detail_text);

            let detail_section = div()
                .mt_1()
                .pt_1()
                .border_t_1()
                .border_color(palette.error_border)
                .flex()
                .flex_col()
                .gap_1()
                .child(toggle_btn)
                .when(show_detail, |d| d.child(detail_box));

            el.child(detail_section)
        })
        .into_any_element()
}
