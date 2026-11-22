//! Settings sheet view (language, theme, auto-dim hours, wake signal interval, debug mode).

use crate::config::ThemeMode;
use crate::i18n::{I18n, Language};
use crate::ui::components::controls::{
    primary_button, secondary_button, segment_option, stepper_button,
};
use crate::ui::theme::ThemePalette;
use crate::ui::{ActiveInput, PicoSleepApp, format_hours};
use gpui::{
    AnyElement, Context, FontWeight, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, div, px,
};

impl PicoSleepApp {
    pub(crate) fn render_settings_sheet(
        &self,
        palette: ThemePalette,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let i18n = I18n::new(self.pending_language);

        let language_section = self.render_language_selector(&i18n, palette, cx);
        let theme_section = self.render_theme_selector(&i18n, palette, cx);
        let dim_section = self.render_dim_delay_field(&i18n, palette, cx);
        let interval_section = self.render_interval_field(&i18n, palette, cx);
        let debug_section = self.render_debug_checkbox(&i18n, palette, cx);
        let actions_section = self.render_settings_actions(&i18n, palette, cx);

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_4()
            .child(language_section)
            .child(theme_section)
            .child(dim_section)
            .child(interval_section)
            .child(debug_section)
            .child(actions_section)
            .into_any_element()
    }

    fn render_language_selector(
        &self,
        i18n: &I18n,
        palette: ThemePalette,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let ja_btn = segment_option(
            "opt-lang-ja",
            "日本語",
            self.pending_language == Language::Ja,
            palette,
            cx.listener(|this, _, _, cx| {
                this.pending_language = Language::Ja;
                cx.notify();
            }),
        );

        let en_btn = segment_option(
            "opt-lang-en",
            "English",
            self.pending_language == Language::En,
            palette,
            cx.listener(|this, _, _, cx| {
                this.pending_language = Language::En;
                cx.notify();
            }),
        );

        div()
            .flex()
            .flex_col()
            .gap_1()
            .child(field_label(i18n.language_label(), palette))
            .child(div().flex().gap_2().child(ja_btn).child(en_btn))
            .into_any_element()
    }

    fn render_theme_selector(
        &self,
        i18n: &I18n,
        palette: ThemePalette,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let light_btn = segment_option(
            "opt-theme-light",
            i18n.theme_light(),
            self.pending_theme == ThemeMode::Light,
            palette,
            cx.listener(|this, _, _, cx| {
                this.pending_theme = ThemeMode::Light;
                cx.notify();
            }),
        );

        let dark_btn = segment_option(
            "opt-theme-dark",
            i18n.theme_dark(),
            self.pending_theme == ThemeMode::Dark,
            palette,
            cx.listener(|this, _, _, cx| {
                this.pending_theme = ThemeMode::Dark;
                cx.notify();
            }),
        );

        let system_btn = segment_option(
            "opt-theme-system",
            i18n.theme_system(),
            self.pending_theme == ThemeMode::System,
            palette,
            cx.listener(|this, _, _, cx| {
                this.pending_theme = ThemeMode::System;
                cx.notify();
            }),
        );

        div()
            .flex()
            .flex_col()
            .gap_1()
            .child(field_label(i18n.theme_label(), palette))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(light_btn)
                    .child(dark_btn)
                    .child(system_btn),
            )
            .into_any_element()
    }

    fn render_dim_delay_field(
        &self,
        i18n: &I18n,
        palette: ThemePalette,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let focused = self.active_input == ActiveInput::DimDelay;
        let display_val = if self.pending_dim_text.is_empty() {
            "0".to_string()
        } else {
            self.pending_dim_text.clone()
        };

        let text_box = div()
            .id("input-dim-hours")
            .flex_1()
            .px_3()
            .py(px(7.0))
            .rounded_lg()
            .bg(palette.subtle_bg)
            .border_1()
            .border_color(if focused {
                palette.brand
            } else {
                palette.border
            })
            .cursor_pointer()
            .on_click(cx.listener(|this, _, window, cx| {
                this.active_input = ActiveInput::DimDelay;
                window.focus(&this.focus_handle);
                cx.notify();
            }))
            .text_size(px(13.0))
            .text_color(palette.text)
            .child(display_val);

        let dec_btn = stepper_button(
            "btn-dim-dec",
            "−",
            palette,
            cx.listener(|this, _, _, cx| {
                let val = this.pending_dim_text.trim().parse::<f64>().unwrap_or(1.0);
                this.pending_dim_text = format_hours((val - 0.5).max(0.0));
                cx.notify();
            }),
        );

        let inc_btn = stepper_button(
            "btn-dim-inc",
            "+",
            palette,
            cx.listener(|this, _, _, cx| {
                let val = this.pending_dim_text.trim().parse::<f64>().unwrap_or(1.0);
                this.pending_dim_text = format_hours((val + 0.5).min(168.0));
                cx.notify();
            }),
        );

        let input_row = div()
            .flex()
            .items_center()
            .gap_2()
            .child(text_box)
            .child(dec_btn)
            .child(inc_btn);

        div()
            .flex()
            .flex_col()
            .gap_1()
            .child(field_label(i18n.dim_setting_label(), palette))
            .child(input_row)
            .child(assistive_note(i18n.dim_setting_note(), palette))
            .into_any_element()
    }

    fn render_interval_field(
        &self,
        i18n: &I18n,
        palette: ThemePalette,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let focused = self.active_input == ActiveInput::KeepAwakeInterval;
        let display_val = if self.pending_interval_text.is_empty() {
            "3".to_string()
        } else {
            self.pending_interval_text.clone()
        };

        let text_box = div()
            .id("input-keep-awake-interval")
            .flex_1()
            .px_3()
            .py(px(7.0))
            .rounded_lg()
            .bg(palette.subtle_bg)
            .border_1()
            .border_color(if focused {
                palette.brand
            } else {
                palette.border
            })
            .cursor_pointer()
            .on_click(cx.listener(|this, _, window, cx| {
                this.active_input = ActiveInput::KeepAwakeInterval;
                window.focus(&this.focus_handle);
                cx.notify();
            }))
            .text_size(px(13.0))
            .text_color(palette.text)
            .child(display_val);

        let dec_btn = stepper_button(
            "btn-interval-dec",
            "−",
            palette,
            cx.listener(|this, _, _, cx| {
                let val = this
                    .pending_interval_text
                    .trim()
                    .parse::<u64>()
                    .unwrap_or(3);
                this.pending_interval_text = val.saturating_sub(1).max(1).to_string();
                cx.notify();
            }),
        );

        let inc_btn = stepper_button(
            "btn-interval-inc",
            "+",
            palette,
            cx.listener(|this, _, _, cx| {
                let val = this
                    .pending_interval_text
                    .trim()
                    .parse::<u64>()
                    .unwrap_or(3);
                this.pending_interval_text = (val + 1).min(3600).to_string();
                cx.notify();
            }),
        );

        let input_row = div()
            .flex()
            .items_center()
            .gap_2()
            .child(text_box)
            .child(dec_btn)
            .child(inc_btn);

        div()
            .flex()
            .flex_col()
            .gap_1()
            .child(field_label(i18n.keep_awake_interval_label(), palette))
            .child(input_row)
            .child(assistive_note(i18n.keep_awake_interval_note(), palette))
            .into_any_element()
    }

    fn render_debug_checkbox(
        &self,
        i18n: &I18n,
        palette: ThemePalette,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let checked = self.pending_debug;

        let checkbox_box = div()
            .w(px(18.0))
            .h(px(18.0))
            .rounded_sm()
            .border_1()
            .border_color(if checked {
                palette.brand
            } else {
                palette.border
            })
            .bg(if checked {
                palette.brand
            } else {
                palette.subtle_bg
            })
            .flex()
            .items_center()
            .justify_center()
            .text_size(px(12.0))
            .font_weight(FontWeight::BOLD)
            .text_color(gpui::white())
            .child(if checked { "✓" } else { "" });

        let label = div()
            .text_size(px(13.0))
            .font_weight(FontWeight::BOLD)
            .text_color(palette.text)
            .child(i18n.debug_mode());

        div()
            .id("chk-debug-mode")
            .py_1()
            .cursor_pointer()
            .flex()
            .items_center()
            .gap_2()
            .on_click(cx.listener(|this, _, _, cx| {
                this.pending_debug = !this.pending_debug;
                cx.notify();
            }))
            .child(checkbox_box)
            .child(label)
            .into_any_element()
    }

    fn render_settings_actions(
        &self,
        i18n: &I18n,
        palette: ThemePalette,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let apply_btn = primary_button(
            "btn-settings-apply",
            i18n.btn_apply_changes(),
            false,
            palette,
            cx.listener(|this, _, _, cx| {
                this.apply_settings(cx);
            }),
        );

        let cancel_btn = secondary_button(
            "btn-settings-cancel",
            i18n.cancel(),
            false,
            palette,
            cx.listener(|this, _, _, cx| {
                this.active_sheet = None;
                this.active_input = ActiveInput::None;
                cx.notify();
            }),
        );

        div()
            .pt_3()
            .border_t_1()
            .border_color(palette.border)
            .flex()
            .flex_col()
            .gap_2()
            .child(apply_btn)
            .child(cancel_btn)
            .into_any_element()
    }
}

fn field_label(label: &'static str, palette: ThemePalette) -> impl IntoElement {
    div()
        .text_size(px(12.0))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(palette.text)
        .child(label)
}

fn assistive_note(note: &'static str, palette: ThemePalette) -> impl IntoElement {
    div()
        .text_size(px(11.0))
        .line_height(px(15.0))
        .text_color(palette.text_muted)
        .child(note)
}
