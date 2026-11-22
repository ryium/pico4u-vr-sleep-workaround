//! Large circular start/stop power toggle button on the Operate screen.

use crate::i18n::I18n;
use crate::ui::theme::ThemePalette;
use gpui::{
    AnyElement, App, ClickEvent, FontWeight, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, Window, div, prelude::*, px, svg,
};

pub fn render_power_button(
    is_running: bool,
    disabled: bool,
    i18n: &I18n,
    palette: ThemePalette,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> AnyElement {
    let bg_color = if disabled {
        palette.disabled_bg
    } else if is_running {
        palette.card
    } else {
        palette.brand
    };

    let fg_color = if disabled {
        palette.disabled_text
    } else if is_running {
        palette.brand
    } else {
        gpui::white()
    };

    let hover_bg = if is_running {
        palette.brand_tint
    } else {
        palette.brand_hover
    };

    let icon_path = if is_running {
        "icons/moon.svg"
    } else {
        "icons/play.svg"
    };

    let main_text = if is_running {
        i18n.status_running()
    } else {
        i18n.power_btn_start()
    };

    let sub_text = if is_running {
        i18n.power_btn_tap_to_stop()
    } else {
        i18n.power_btn_tap_to_start()
    };

    let icon_el = div()
        .mb_2()
        .child(svg().path(icon_path).size(px(40.0)).text_color(fg_color));

    let title_el = div()
        .text_size(px(20.0))
        .font_weight(FontWeight::BOLD)
        .text_color(fg_color)
        .child(main_text);

    let subtitle_el = div()
        .mt(px(6.0))
        .text_size(px(11.0))
        .text_color(fg_color)
        .opacity(0.85)
        .child(sub_text);

    div()
        .id("btn-power-toggle")
        .w(px(176.0))
        .h(px(176.0))
        .rounded_full()
        .bg(bg_color)
        .when(is_running && !disabled, |b| {
            b.border(px(4.0)).border_color(palette.brand)
        })
        .when(!disabled, |b| {
            b.cursor_pointer()
                .hover(move |s| s.bg(hover_bg))
                .on_click(on_click)
        })
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .child(icon_el)
        .child(title_el)
        .child(subtitle_el)
        .into_any_element()
}
