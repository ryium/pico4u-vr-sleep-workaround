//! Step 3: Main sleep prevention power button, connection status chip, and elapsed timer screen.

use crate::connection::ConnectionMode;
use crate::ui::components::connection_chip::render_connection_chip;
use crate::ui::components::error_card::render_error_card;
use crate::ui::components::power_button::render_power_button;
use crate::ui::theme::ThemePalette;
use crate::ui::{ActiveSheet, PicoSleepApp, Screen, SetupRunStatus, format_elapsed};
use gpui::{
    AnyElement, Context, FontWeight, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, div, prelude::*, px,
};

impl PicoSleepApp {
    pub(crate) fn render_operate_screen(
        &self,
        palette: ThemePalette,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let i18n = self.i18n();
        let mode = self.config.last_connection_mode;
        let ip = &self.config.ip_address;
        let connected = self.is_connected();
        let is_running = self.is_running;

        let show_wireless_hint = !is_running
            && connected
            && mode == Some(ConnectionMode::Wireless)
            && self.wireless_setup_status == SetupRunStatus::Success;

        let is_already_wireless =
            self.setup_steps.first().and_then(|s| s.detail.as_deref()) == Some("already_connected");

        let wireless_hint_text = if show_wireless_hint {
            if is_already_wireless {
                i18n.setup_already_wireless()
            } else {
                i18n.wireless_status_success()
            }
        } else {
            ""
        };

        let top_chip_section = div()
            .flex()
            .flex_col()
            .items_center()
            .gap_2()
            .child(render_connection_chip(mode, ip, connected, &i18n, palette))
            .child(
                div()
                    .h(px(18.0))
                    .text_size(px(11.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(palette.brand)
                    .text_center()
                    .child(wireless_hint_text),
            );

        let power_disabled = (!connected && !is_running) || self.is_busy;
        let power_btn = render_power_button(
            is_running,
            power_disabled,
            &i18n,
            palette,
            cx.listener(|this, _, _, cx| {
                this.trigger_toggle_keep_awake(cx);
            }),
        );

        let status_line_1 = if is_running {
            i18n.operate_running_desc(self.config.keep_awake_interval_secs)
        } else {
            i18n.status_stopped().to_string()
        };

        let elapsed_str = format_elapsed(self.elapsed_seconds);
        let mut status_line_2 = i18n.operate_elapsed(&elapsed_str);
        if self.config.dim_delay_hours > 0.0 {
            status_line_2.push_str(&format!(
                " · {}",
                i18n.operate_dim_after(self.config.dim_delay_hours)
            ));
        }

        let status_text_block = div()
            .mt_3()
            .flex()
            .flex_col()
            .items_center()
            .child(
                div()
                    .text_size(px(12.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(palette.text)
                    .child(status_line_1),
            )
            .child(
                div()
                    .mt_1()
                    .h(px(16.0))
                    .text_size(px(11.0))
                    .text_color(palette.text_muted)
                    .child(if is_running {
                        status_line_2
                    } else {
                        String::new()
                    }),
            );

        let center_section = div()
            .w_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .child(power_btn)
            .child(status_text_block)
            .when(self.runtime_error.is_some() && !is_running, |el| {
                let err_card = render_error_card(
                    self.runtime_error.as_ref(),
                    None,
                    self.show_error_detail,
                    &i18n,
                    palette,
                    cx.listener(|this, _, _, cx| {
                        this.show_error_detail = !this.show_error_detail;
                        cx.notify();
                    }),
                );
                el.child(div().mt_3().w_full().child(err_card))
            });

        let footer_section = self.render_operate_footer(is_running, palette, cx);

        div()
            .flex_1()
            .flex()
            .flex_col()
            .justify_between()
            .items_center()
            .px_5()
            .py_4()
            .child(top_chip_section)
            .child(center_section)
            .child(footer_section)
            .into_any_element()
    }

    fn render_operate_footer(
        &self,
        is_running: bool,
        palette: ThemePalette,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let i18n = self.i18n();

        let change_conn_btn = div()
            .id("btn-change-connection")
            .py_1()
            .text_size(px(12.0))
            .text_color(if is_running {
                palette.disabled_text
            } else {
                palette.text_muted
            })
            .when(!is_running, |b| {
                b.cursor_pointer()
                    .hover(|s| s.text_color(palette.text))
                    .on_click(cx.listener(|this, _, _, cx| {
                        if !this.is_running {
                            this.reset_setup();
                            this.selected_mode_card = this
                                .config
                                .last_connection_mode
                                .unwrap_or(ConnectionMode::Wireless);
                            this.screen = Screen::Mode;
                            cx.notify();
                        }
                    }))
            })
            .child(i18n.btn_change_connection());

        let logs_btn = div()
            .id("btn-open-logs")
            .py_1()
            .cursor_pointer()
            .text_size(px(12.0))
            .text_color(palette.text_muted)
            .hover(|s| s.text_color(palette.text))
            .on_click(cx.listener(|this, _, _, cx| {
                this.active_sheet = Some(ActiveSheet::Logs);
                cx.notify();
            }))
            .child(i18n.tab_logs());

        div()
            .w_full()
            .px_4()
            .pt_2()
            .pb_2()
            .border_t_1()
            .border_color(palette.border)
            .flex()
            .items_center()
            .justify_between()
            .child(change_conn_btn)
            .when(self.is_debug, |el| el.child(logs_btn))
            .into_any_element()
    }
}
