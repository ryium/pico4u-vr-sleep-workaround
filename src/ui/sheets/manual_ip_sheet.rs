//! Manual IPv4 address entry sheet for Wireless ADB connection.

use crate::ui::components::controls::{primary_button, secondary_button};
use crate::ui::theme::ThemePalette;
use crate::ui::{ActiveInput, PicoSleepApp};
use gpui::{
    AnyElement, Context, FontWeight, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, div, px,
};

impl PicoSleepApp {
    pub(crate) fn render_manual_ip_sheet(
        &self,
        palette: ThemePalette,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let i18n = self.i18n();
        let ip_focused = self.active_input == ActiveInput::ManualIp;

        let prompt_label = div()
            .text_size(px(12.5))
            .font_weight(FontWeight::MEDIUM)
            .text_color(palette.text)
            .child(i18n.prompt_enter_ip());

        let ip_display = if self.manual_ip_text.is_empty() {
            "192.168.1.20".to_string()
        } else {
            self.manual_ip_text.clone()
        };

        let ip_input_box = div()
            .id("input-manual-ip")
            .w_full()
            .px_3()
            .py_2()
            .rounded_lg()
            .bg(palette.subtle_bg)
            .border_1()
            .border_color(if ip_focused {
                palette.brand
            } else {
                palette.border
            })
            .cursor_pointer()
            .on_click(cx.listener(|this, _, window, cx| {
                this.active_input = ActiveInput::ManualIp;
                window.focus(&this.focus_handle);
                cx.notify();
            }))
            .text_size(px(14.0))
            .text_color(if self.manual_ip_text.is_empty() {
                palette.text_muted
            } else {
                palette.text
            })
            .child(ip_display);

        let connect_btn = primary_button(
            "btn-manual-ip-connect",
            i18n.btn_continue(),
            self.manual_ip_text.trim().is_empty(),
            palette,
            cx.listener(|this, _, _, cx| {
                let ip = this.manual_ip_text.clone();
                this.trigger_connect_manual(ip, cx);
            }),
        );

        let cancel_btn = secondary_button(
            "btn-manual-ip-cancel",
            i18n.cancel(),
            false,
            palette,
            cx.listener(|this, _, _, cx| {
                this.active_sheet = None;
                this.active_input = ActiveInput::None;
                cx.notify();
            }),
        );

        let actions = div()
            .flex()
            .flex_col()
            .gap_2()
            .child(connect_btn)
            .child(cancel_btn);

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_4()
            .child(prompt_label)
            .child(ip_input_box)
            .child(actions)
            .into_any_element()
    }
}
