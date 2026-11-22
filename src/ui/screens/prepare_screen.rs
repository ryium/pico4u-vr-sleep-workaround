//! Step 2: USB verification & Wireless setup diagnostic screen.

use crate::commands::{SetupStep, SetupStepItem, StepState};
use crate::connection::ConnectionMode;
use crate::ui::components::controls::primary_button;
use crate::ui::components::error_card::render_error_card;
use crate::ui::components::setup_checklist::render_setup_checklist;
use crate::ui::components::step_indicator::render_step_indicator;
use crate::ui::theme::ThemePalette;
use crate::ui::{ActiveInput, ActiveSheet, PicoSleepApp, Screen, SetupRunStatus};
use gpui::{
    AnyElement, Context, FontWeight, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, div, prelude::*, px,
};

impl PicoSleepApp {
    pub(crate) fn render_prepare_screen(
        &self,
        palette: ThemePalette,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let i18n = self.i18n();
        let is_wireless = self.target_mode == ConnectionMode::Wireless;
        let is_usb_detected = self
            .connection_status
            .as_ref()
            .is_some_and(|s| s.is_usb_connected);

        let is_busy = self.is_busy
            || if is_wireless {
                self.wireless_setup_status == SetupRunStatus::Loading
            } else {
                self.wired_setup_status == SetupRunStatus::Loading
            };

        let is_error = self.connection_error.is_some()
            || if is_wireless {
                self.wireless_setup_status == SetupRunStatus::Error
            } else {
                self.wired_setup_status == SetupRunStatus::Error
            };

        let display_steps = self.visible_prepare_steps(is_wireless, is_usb_detected);

        let action_label = if is_busy {
            if is_wireless {
                i18n.wireless_status_loading()
            } else {
                i18n.wired_status_loading()
            }
        } else if is_error {
            i18n.btn_try_again()
        } else if is_wireless {
            i18n.btn_setup_wireless()
        } else {
            i18n.btn_check_devices()
        };

        let step_bar = render_step_indicator(Screen::Prepare, &i18n, palette);

        let heading_block = div()
            .mt_4()
            .mb_4()
            .flex()
            .flex_col()
            .items_center()
            .gap_1()
            .child(
                div()
                    .text_size(px(18.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(palette.text)
                    .child(if is_wireless {
                        i18n.prepare_wireless_title()
                    } else {
                        i18n.prepare_wired_title()
                    }),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(palette.text_muted)
                    .child(if is_wireless {
                        i18n.prepare_wireless_desc()
                    } else {
                        i18n.prepare_wired_desc()
                    }),
            );

        let checklist = render_setup_checklist(&display_steps, &i18n, palette);

        let manual_ip_link = div()
            .id("btn-open-manual-ip")
            .mt_3()
            .py_1()
            .w_full()
            .text_center()
            .text_size(px(12.0))
            .underline()
            .text_color(palette.text_muted)
            .when(!is_busy, |b| {
                b.cursor_pointer()
                    .hover(|s| s.text_color(palette.text))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.manual_ip_text = this.config.ip_address.clone();
                        this.active_input = ActiveInput::ManualIp;
                        this.active_sheet = Some(ActiveSheet::ManualIp);
                        cx.notify();
                    }))
            })
            .child(i18n.btn_connect_manual());

        let top_section = div()
            .flex()
            .flex_col()
            .child(step_bar)
            .child(heading_block)
            .child(checklist)
            .when(is_error, |el| {
                let fallback = if is_wireless {
                    i18n.wireless_status_error().to_string()
                } else {
                    i18n.wired_status_error().to_string()
                };
                let err_card = render_error_card(
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
                el.child(div().mt_3().child(err_card))
            })
            .when(is_wireless, |el| el.child(manual_ip_link));

        let primary_action = primary_button(
            "btn-prepare-start",
            action_label,
            is_busy,
            palette,
            cx.listener(move |this, _, _, cx| {
                if is_wireless {
                    this.trigger_setup_wireless(cx);
                } else {
                    this.trigger_check_devices(cx);
                }
            }),
        );

        let back_button = div()
            .id("btn-prepare-back")
            .w_full()
            .py_2()
            .text_center()
            .text_size(px(12.0))
            .text_color(palette.text_muted)
            .when(!is_busy, |b| {
                b.cursor_pointer()
                    .hover(|s| s.text_color(palette.text))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.reset_setup();
                        this.screen = Screen::Mode;
                        cx.notify();
                    }))
            })
            .child(i18n.btn_back());

        let bottom_section = div()
            .flex()
            .flex_col()
            .gap_2()
            .pt_3()
            .pb_2()
            .child(primary_action)
            .child(back_button);

        div()
            .id("prepare-scroll-container")
            .flex_1()
            .flex()
            .flex_col()
            .justify_between()
            .px_5()
            .pt_2()
            .pb_4()
            .overflow_y_scroll()
            .child(top_section)
            .child(bottom_section)
            .into_any_element()
    }

    fn visible_prepare_steps(
        &self,
        is_wireless: bool,
        is_usb_detected: bool,
    ) -> Vec<SetupStepItem> {
        self.setup_steps
            .iter()
            .filter(|s| {
                is_wireless
                    || matches!(
                        s.id,
                        SetupStep::Usb | SetupStep::Authorized | SetupStep::Pico
                    )
            })
            .map(|s| {
                if is_usb_detected
                    && s.state == StepState::Idle
                    && matches!(
                        s.id,
                        SetupStep::Usb | SetupStep::Authorized | SetupStep::Pico
                    )
                {
                    SetupStepItem {
                        id: s.id,
                        state: StepState::Ok,
                        detail: s.detail.clone(),
                    }
                } else {
                    s.clone()
                }
            })
            .collect()
    }
}
