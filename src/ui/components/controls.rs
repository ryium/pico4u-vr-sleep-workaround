//! Reusable form controls: primary/secondary buttons, segmented options, and stepper buttons.

use crate::ui::theme::ThemePalette;
use gpui::{
    AnyElement, App, ClickEvent, FontWeight, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, Window, div, prelude::*, px,
};

pub fn primary_button(
    id: &'static str,
    label: &str,
    disabled: bool,
    palette: ThemePalette,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> AnyElement {
    let bg = if disabled {
        palette.disabled_bg
    } else {
        palette.brand
    };
    let fg = if disabled {
        palette.disabled_text
    } else {
        gpui::white()
    };

    div()
        .id(id)
        .w_full()
        .py(px(10.0))
        .px_4()
        .rounded_full()
        .bg(bg)
        .text_color(fg)
        .text_size(px(13.0))
        .font_weight(FontWeight::BOLD)
        .flex()
        .items_center()
        .justify_center()
        .when(!disabled, |b| {
            b.cursor_pointer()
                .hover(|s| s.bg(palette.brand_hover))
                .on_click(on_click)
        })
        .child(label.to_string())
        .into_any_element()
}

pub fn secondary_button(
    id: &'static str,
    label: &str,
    disabled: bool,
    palette: ThemePalette,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> AnyElement {
    let fg = if disabled {
        palette.disabled_text
    } else {
        palette.text
    };

    div()
        .id(id)
        .w_full()
        .py(px(9.0))
        .px_4()
        .rounded_full()
        .bg(palette.subtle_bg)
        .border_1()
        .border_color(palette.border)
        .text_color(fg)
        .text_size(px(13.0))
        .font_weight(FontWeight::SEMIBOLD)
        .flex()
        .items_center()
        .justify_center()
        .when(!disabled, |b| {
            b.cursor_pointer()
                .hover(|s| s.border_color(palette.brand))
                .on_click(on_click)
        })
        .child(label.to_string())
        .into_any_element()
}

pub fn segment_option(
    id: &'static str,
    label: &str,
    selected: bool,
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
        palette.subtle_bg
    };
    let text_col = if selected {
        palette.brand
    } else {
        palette.text
    };
    let weight = if selected {
        FontWeight::BOLD
    } else {
        FontWeight::MEDIUM
    };

    div()
        .id(id)
        .flex_1()
        .py(px(7.0))
        .px_2()
        .rounded_lg()
        .border_1()
        .border_color(border_col)
        .bg(bg_col)
        .text_color(text_col)
        .text_size(px(12.0))
        .font_weight(weight)
        .cursor_pointer()
        .flex()
        .items_center()
        .justify_center()
        .on_click(on_click)
        .child(label.to_string())
        .into_any_element()
}

pub fn stepper_button(
    id: &'static str,
    label: &'static str,
    palette: ThemePalette,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> AnyElement {
    div()
        .id(id)
        .w(px(34.0))
        .h(px(34.0))
        .rounded_lg()
        .bg(palette.subtle_bg)
        .border_1()
        .border_color(palette.border)
        .cursor_pointer()
        .hover(|s| s.border_color(palette.brand))
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(16.0))
        .font_weight(FontWeight::BOLD)
        .text_color(palette.text)
        .on_click(on_click)
        .child(label)
        .into_any_element()
}
