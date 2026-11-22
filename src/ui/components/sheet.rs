//! Bottom slide-up modal sheet container.

use crate::ui::theme::ThemePalette;
use gpui::{
    AnyElement, App, ClickEvent, FontWeight, InteractiveElement, IntoElement, MouseButton,
    ParentElement, StatefulInteractiveElement, Styled, Window, div, px, rgba,
};

pub fn render_sheet(
    title: &str,
    body: AnyElement,
    palette: ThemePalette,
    on_close: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> AnyElement {
    let close_button = div()
        .id("btn-close-sheet")
        .px_2()
        .py_1()
        .rounded_md()
        .cursor_pointer()
        .text_size(px(14.0))
        .font_weight(FontWeight::BOLD)
        .text_color(palette.text_muted)
        .hover(|s| s.text_color(palette.text))
        .on_click(on_close)
        .child("✕");

    let header = div()
        .px_5()
        .py_3()
        .border_b_1()
        .border_color(palette.border)
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .text_size(px(14.0))
                .font_weight(FontWeight::BOLD)
                .text_color(palette.text)
                .child(title.to_string()),
        )
        .child(close_button);

    let content = div()
        .id("sheet-body-scroll")
        .p_5()
        .overflow_y_scroll()
        .child(body);

    let panel = div()
        .w_full()
        .max_h(px(620.0))
        .rounded_t_2xl()
        .bg(palette.card)
        .border_t_1()
        .border_color(palette.border)
        .flex()
        .flex_col()
        .child(header)
        .child(content);

    div()
        .id("sheet-backdrop")
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .bg(rgba(0x00000080))
        .flex()
        .flex_col()
        .justify_end()
        .on_mouse_down(MouseButton::Left, |_, _, cx| {
            cx.stop_propagation();
        })
        .child(panel)
        .into_any_element()
}
