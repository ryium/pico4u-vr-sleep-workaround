//! 3-step wizard progress indicator (Mode -> Prepare -> Operate).

use crate::i18n::I18n;
use crate::ui::Screen;
use crate::ui::theme::ThemePalette;
use gpui::{AnyElement, FontWeight, IntoElement, ParentElement, Styled, div, px, rgba};

pub fn render_step_indicator(current: Screen, i18n: &I18n, palette: ThemePalette) -> AnyElement {
    let steps = [
        (0usize, i18n.step_mode()),
        (1usize, i18n.step_prepare()),
        (2usize, i18n.step_operate()),
    ];
    let current_idx = match current {
        Screen::Mode => 0,
        Screen::Prepare => 1,
        Screen::Operate => 2,
    };

    let first_segment_color = if current_idx >= 1 {
        palette.brand
    } else {
        palette.disabled_bg
    };
    let second_segment_color = if current_idx >= 2 {
        palette.brand
    } else {
        palette.disabled_bg
    };

    let connector_line = div()
        .absolute()
        .top(px(17.0))
        .left(px(76.0))
        .right(px(76.0))
        .h(px(2.0))
        .flex()
        .child(div().flex_1().h_full().bg(first_segment_color))
        .child(div().flex_1().h_full().bg(second_segment_color));

    let step_nodes = steps.into_iter().map(|(idx, label)| {
        let is_done = idx < current_idx;
        let is_active = idx == current_idx;
        render_step_node(label, is_done, is_active, palette)
    });

    div()
        .w_full()
        .relative()
        .flex()
        .items_center()
        .justify_between()
        .px_8()
        .py(px(6.0))
        .child(connector_line)
        .children(step_nodes)
        .into_any_element()
}

fn render_step_node(
    label: &'static str,
    is_done: bool,
    is_active: bool,
    palette: ThemePalette,
) -> impl IntoElement {
    let dot_bg = if is_active || is_done {
        palette.brand
    } else {
        palette.disabled_bg
    };
    let ring_bg = if is_active {
        palette.brand_tint
    } else {
        rgba(0x00000000).into()
    };
    let label_color = if is_active {
        palette.brand
    } else if is_done {
        palette.text
    } else {
        palette.text_muted
    };
    let label_weight = if is_active {
        FontWeight::SEMIBOLD
    } else {
        FontWeight::MEDIUM
    };

    let inner_dot = div()
        .w(px(16.0))
        .h(px(16.0))
        .rounded_full()
        .bg(dot_bg)
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(10.0))
        .font_weight(FontWeight::BOLD)
        .text_color(gpui::white())
        .child(if is_done { "✓" } else { "" });

    let outer_ring = div()
        .w(px(24.0))
        .h(px(24.0))
        .rounded_full()
        .bg(ring_bg)
        .flex()
        .items_center()
        .justify_center()
        .child(inner_dot);

    let text_label = div()
        .text_size(px(11.0))
        .font_weight(label_weight)
        .text_color(label_color)
        .child(label);

    div()
        .flex_1()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(4.0))
        .child(outer_ring)
        .child(text_label)
}
