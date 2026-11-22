//! Keyboard input and clipboard handling for modal sheet input fields.

use crate::ui::{ActiveInput, ActiveSheet, PicoSleepApp};
use gpui::{Context, KeyDownEvent, Window};

impl PicoSleepApp {
    pub(crate) fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();
        if key == "escape" {
            if self.active_sheet.is_some() {
                self.active_sheet = None;
                self.active_input = ActiveInput::None;
                cx.notify();
            }
            return;
        }

        if self.active_input == ActiveInput::None {
            return;
        }

        if (event.keystroke.modifiers.control || event.keystroke.modifiers.platform) && key == "v" {
            if let Some(clip) = cx.read_from_clipboard().and_then(|item| item.text()) {
                for ch in clip.chars() {
                    self.push_input_char(ch);
                }
                cx.notify();
            }
            return;
        }

        if key == "backspace" {
            match self.active_input {
                ActiveInput::ManualIp => {
                    self.manual_ip_text.pop();
                }
                ActiveInput::DimDelay => {
                    self.pending_dim_text.pop();
                }
                ActiveInput::KeepAwakeInterval => {
                    self.pending_interval_text.pop();
                }
                ActiveInput::None => {}
            }
            cx.notify();
            return;
        }

        if key == "enter" {
            if self.active_input == ActiveInput::ManualIp {
                let ip = self.manual_ip_text.clone();
                self.trigger_connect_manual(ip, cx);
            } else if self.active_sheet == Some(ActiveSheet::Settings) {
                self.apply_settings(cx);
            }
            return;
        }

        if !event.keystroke.modifiers.control
            && !event.keystroke.modifiers.alt
            && !event.keystroke.modifiers.platform
        {
            if let Some(ref key_char) = event.keystroke.key_char {
                for ch in key_char.chars() {
                    self.push_input_char(ch);
                }
                cx.notify();
            } else if key.len() == 1
                && let Some(ch) = key.chars().next()
            {
                self.push_input_char(ch);
                cx.notify();
            }
        }
    }

    fn push_input_char(&mut self, ch: char) {
        match self.active_input {
            ActiveInput::ManualIp => {
                if (ch.is_ascii_digit() || ch == '.' || ch == ':') && self.manual_ip_text.len() < 24
                {
                    self.manual_ip_text.push(ch);
                }
            }
            ActiveInput::DimDelay => {
                if (ch.is_ascii_digit() || (ch == '.' && !self.pending_dim_text.contains('.')))
                    && self.pending_dim_text.len() < 8
                {
                    self.pending_dim_text.push(ch);
                }
            }
            ActiveInput::KeepAwakeInterval => {
                if ch.is_ascii_digit() && self.pending_interval_text.len() < 6 {
                    self.pending_interval_text.push(ch);
                }
            }
            ActiveInput::None => {}
        }
    }
}
