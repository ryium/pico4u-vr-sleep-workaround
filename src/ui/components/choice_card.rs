//! Selectable option card for choosing between Wired and Wireless connection modes.

use crate::ui::theme::ThemePalette;
use gpui::{
    AnyElement, App, ClickEvent, FontWeight, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, Window, div, prelude::*, px, svg,
};

#[allow(clippy::too_many_arguments)]
pub fn render_choice_card(
    id: &'static str,
    selected: bool,
    icon_path: &'static str,
    title: &str,
    description: &str,
    badge: Option<&str>,
    palette: ThemePalette,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> AnyElement {
    let border_col = if selected {
        palette.brand
    } else {
        palette.border
    };
    let bg_col = if selected {
        palette.brand_tint
    } else {
        palette.card
    };
    let icon_bg = if selected {
        palette.brand_tint
    } else {
        palette.subtle_bg
    };
    let icon_col = if selected {
        palette.brand
    } else {
        palette.text_muted
    };

    let icon_box = div()
        .p(px(10.0))
        .rounded_lg()
        .bg(icon_bg)
        .flex_shrink_0()
        .child(svg().path(icon_path).size(px(22.0)).text_color(icon_col));

    let title_el = div()
        .text_size(px(14.0))
        .font_weight(FontWeight::BOLD)
        .text_color(palette.text)
        .child(title.to_string());

    let desc_el = div()
        .text_size(px(12.0))
        .line_height(px(17.0))
        .text_color(palette.text_muted)
        .child(description.to_string());

    let text_column = div()
        .flex()
        .flex_col()
        .gap_1()
        .flex_1()
        .min_w(px(0.0))
        .child(title_el)
        .child(desc_el);

    let content_row = div()
        .w_full()
        .flex()
        .items_start()
        .gap_3()
        .child(icon_box)
        .child(text_column);

    div()
        .id(id)
        .relative()
        .w_full()
        .p_4()
        .rounded_xl()
        .border_1()
        .border_color(border_col)
        .bg(bg_col)
        .cursor_pointer()
        .on_click(on_click)
        .when_some(badge, |el, badge_text| {
            let badge_pill = div()
                .absolute()
                .top(px(14.0))
                .right(px(14.0))
                .px_2()
                .py(px(2.0))
                .rounded_full()
                .bg(palette.brand)
                .text_size(px(10.0))
                .font_weight(FontWeight::BOLD)
                .text_color(gpui::white())
                .child(badge_text.to_string());
            el.child(badge_pill)
        })
        .child(content_row)
        .into_any_element()
}
