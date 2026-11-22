//! Step 1: Wired (USB) vs Wireless (Wi-Fi) connection mode selection screen.

use crate::connection::ConnectionMode;
use crate::ui::components::choice_card::render_choice_card;
use crate::ui::components::controls::primary_button;
use crate::ui::components::step_indicator::render_step_indicator;
use crate::ui::theme::ThemePalette;
use crate::ui::{PicoSleepApp, Screen};
use gpui::{
    AnyElement, Context, FontWeight, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, div, prelude::*, px,
};

impl PicoSleepApp {
    pub(crate) fn render_mode_select_screen(
        &self,
        palette: ThemePalette,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let i18n = self.i18n();
        let selected = self.selected_mode_card;
        let can_go_back = self.is_connected();

        let step_bar = render_step_indicator(Screen::Mode, &i18n, palette);

        let heading_block = div()
            .mt_5()
            .mb_6()
            .flex()
            .flex_col()
            .items_center()
            .gap_1()
            .child(
                div()
                    .text_size(px(18.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(palette.text)
                    .child(i18n.mode_select_heading()),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(palette.text_muted)
                    .child(i18n.mode_select_subheading()),
            );

        let wired_card = render_choice_card(
            "card-mode-wired",
            selected == ConnectionMode::Wired,
            "icons/usb.svg",
            i18n.mode_wired(),
            i18n.mode_wired_desc(),
            None,
            palette,
            cx.listener(|this, _, _, cx| {
                this.selected_mode_card = ConnectionMode::Wired;
                cx.notify();
            }),
        );

        let wireless_card = render_choice_card(
            "card-mode-wireless",
            selected == ConnectionMode::Wireless,
            "icons/wifi.svg",
            i18n.mode_wireless(),
            i18n.mode_wireless_desc(),
            Some(i18n.badge_recommended()),
            palette,
            cx.listener(|this, _, _, cx| {
                this.selected_mode_card = ConnectionMode::Wireless;
                cx.notify();
            }),
        );

        let cards_group = div()
            .flex()
            .flex_col()
            .gap_3()
            .child(wired_card)
            .child(wireless_card);

        let top_section = div()
            .flex()
            .flex_col()
            .child(step_bar)
            .child(heading_block)
            .child(cards_group);

        let continue_btn = primary_button(
            "btn-mode-continue",
            i18n.btn_continue(),
            false,
            palette,
            cx.listener(|this, _, _, cx| {
                this.reset_setup();
                this.target_mode = this.selected_mode_card;
                this.screen = Screen::Prepare;
                if this.target_mode == ConnectionMode::Wired
                    && this
                        .connection_status
                        .as_ref()
                        .is_some_and(|s| s.is_usb_connected)
                {
                    this.auto_triggered_wired = true;
                    this.trigger_check_devices(cx);
                }
                cx.notify();
            }),
        );

        let back_link = div()
            .id("btn-mode-back")
            .w_full()
            .py_2()
            .cursor_pointer()
            .text_center()
            .text_size(px(12.0))
            .text_color(palette.text_muted)
            .hover(|s| s.text_color(palette.text))
            .on_click(cx.listener(|this, _, _, cx| {
                this.screen = Screen::Operate;
                cx.notify();
            }))
            .child(i18n.btn_back());

        let bottom_section = div()
            .flex()
            .flex_col()
            .gap_2()
            .pt_4()
            .pb_2()
            .child(continue_btn)
            .when(can_go_back, |el| el.child(back_link));

        div()
            .flex_1()
            .flex()
            .flex_col()
            .justify_between()
            .px_5()
            .pt_2()
            .pb_4()
            .child(top_section)
            .child(bottom_section)
            .into_any_element()
    }
}
