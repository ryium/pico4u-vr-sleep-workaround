//! Step-by-step connection diagnostic checklist component.

use crate::commands::{SetupStepItem, StepState};
use crate::i18n::I18n;
use crate::ui::theme::ThemePalette;
use gpui::{AnyElement, FontWeight, IntoElement, ParentElement, Styled, div, px};

pub fn render_setup_checklist(
    steps: &[SetupStepItem],
    i18n: &I18n,
    palette: ThemePalette,
) -> AnyElement {
    let rows = steps
        .iter()
        .map(|item| render_checklist_row(item, i18n, palette));

    div()
        .w_full()
        .rounded_lg()
        .p_3()
        .bg(palette.subtle_bg)
        .border_1()
        .border_color(palette.subtle_border)
        .flex()
        .flex_col()
        .gap(px(6.0))
        .children(rows)
        .into_any_element()
}

fn render_checklist_row(
    item: &SetupStepItem,
    i18n: &I18n,
    palette: ThemePalette,
) -> impl IntoElement {
    let (icon, icon_col, text_col, status_right) = match item.state {
        StepState::Running => ("⟳", palette.brand, palette.text_muted, "..."),
        StepState::Ok => ("✓", palette.brand, palette.text, "OK"),
        StepState::Failed => ("✕", palette.error_text, palette.error_text, "NG"),
        StepState::Idle | StepState::Skipped => ("○", palette.text_muted, palette.text_muted, "-"),
    };

    let left_group = div()
        .flex()
        .items_center()
        .gap_2()
        .child(
            div()
                .text_size(px(13.0))
                .font_weight(FontWeight::BOLD)
                .text_color(icon_col)
                .child(icon),
        )
        .child(
            div()
                .text_size(px(12.0))
                .text_color(text_col)
                .child(i18n.setup_step_label(item.id)),
        );

    let right_badge = div()
        .text_size(px(11.0))
        .text_color(palette.text_muted)
        .child(status_right);

    div()
        .w_full()
        .py_1()
        .flex()
        .items_center()
        .justify_between()
        .child(left_group)
        .child(right_badge)
}
