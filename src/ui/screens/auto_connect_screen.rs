//! Startup auto-connect progress and failure recovery view.

use crate::connection::ConnectionMode;
use crate::ui::components::controls::{primary_button, secondary_button};
use crate::ui::components::error_card::render_error_card;
use crate::ui::theme::ThemePalette;
use crate::ui::{AutoConnectStatus, PicoSleepApp, Screen};
use gpui::{
    AnyElement, Context, FontWeight, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, div, prelude::*, px,
};

impl PicoSleepApp {
    pub(crate) fn render_auto_connect_screen(
        &self,
        palette: ThemePalette,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let i18n = self.i18n();
        let mode = self.config.last_connection_mode;
        let ip = self.config.ip_address.clone();
        let attempt = self.auto_connect_attempt.max(1);

        let is_connecting = matches!(
            self.auto_connect_status,
            AutoConnectStatus::Idle | AutoConnectStatus::Connecting
        );

        let status_label = if self.auto_connect_status == AutoConnectStatus::Idle || mode.is_none()
        {
            i18n.status_checking().to_string()
        } else if mode == Some(ConnectionMode::Wired) {
            i18n.auto_connect_wired(attempt)
        } else {
            i18n.auto_connect_wireless(&ip, attempt)
        };

        let spinner = div()
            .w(px(32.0))
            .h(px(32.0))
            .rounded_full()
            .border_2()
            .border_color(palette.brand)
            .flex()
            .items_center()
            .justify_center()
            .text_size(px(14.0))
            .text_color(palette.brand)
            .child("⟳");

        let status_text = div()
            .text_size(px(13.5))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(palette.text)
            .text_center()
            .child(status_label);

        let connecting_block = div()
            .py_4()
            .flex()
            .flex_col()
            .items_center()
            .gap_3()
            .child(spinner)
            .child(status_text)
            .when(
                mode == Some(ConnectionMode::Wireless) && !ip.is_empty(),
                |el| {
                    el.child(
                        div()
                            .text_size(px(12.0))
                            .text_color(palette.text_muted)
                            .child(ip.clone()),
                    )
                },
            );

        let card = div()
            .w_full()
            .rounded_2xl()
            .p_6()
            .bg(palette.card)
            .border_1()
            .border_color(palette.border)
            .flex()
            .flex_col()
            .items_center()
            .gap_4()
            .when(is_connecting, |c| c.child(connecting_block))
            .when(self.auto_connect_status == AutoConnectStatus::Failed, |c| {
                c.child(self.render_auto_connect_failed(&ip, palette, cx))
            });

        div()
            .flex_1()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .px_5()
            .child(card)
            .into_any_element()
    }

    fn render_auto_connect_failed(
        &self,
        ip: &str,
        palette: ThemePalette,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let i18n = self.i18n();
        let fallback = i18n.auto_connect_failed_desc(ip);

        let error_card = render_error_card(
            self.connection_error.as_ref(),
            Some(fallback),
            self.show_error_detail,
            &i18n,
            palette,
            cx.listener(|this, _, _, cx| {
                this.show_error_detail = !this.show_error_detail;
                cx.notify();
            }),
        );

        let retry_btn = primary_button(
            "btn-auto-retry",
            i18n.btn_try_again(),
            false,
            palette,
            cx.listener(|this, _, _, cx| {
                this.trigger_auto_connect(cx);
            }),
        );

        let wireless_setup_btn = secondary_button(
            "btn-auto-wireless-setup",
            i18n.btn_wireless_setup(),
            false,
            palette,
            cx.listener(|this, _, _, cx| {
                this.reset_setup();
                this.auto_connect_status = AutoConnectStatus::Skipped;
                this.target_mode = ConnectionMode::Wireless;
                this.screen = Screen::Prepare;
                cx.notify();
            }),
        );

        let manual_setup_link = div()
            .id("btn-auto-manual-setup")
            .w_full()
            .py_1()
            .cursor_pointer()
            .text_center()
            .text_size(px(12.0))
            .text_color(palette.text_muted)
            .hover(|s| s.text_color(palette.text))
            .on_click(cx.listener(|this, _, _, cx| {
                this.reset_setup();
                this.auto_connect_status = AutoConnectStatus::Skipped;
                this.screen = Screen::Mode;
                cx.notify();
            }))
            .child(i18n.btn_manual_setup());

        let actions = div()
            .w_full()
            .flex()
            .flex_col()
            .gap_2()
            .mt_1()
            .child(retry_btn)
            .child(wireless_setup_btn)
            .child(manual_setup_link);

        div()
            .w_full()
            .flex()
            .flex_col()
            .items_center()
            .gap_4()
            .child(error_card)
            .child(actions)
            .into_any_element()
    }
}
