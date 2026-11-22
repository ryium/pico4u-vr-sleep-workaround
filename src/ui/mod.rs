//! GPUI root application state, top-level view composition, and window configuration.

mod actions;
pub mod assets;
pub mod components;
mod input;
pub mod screens;
pub mod sheets;
pub mod theme;

pub use assets::AppAssets;

use crate::commands::{self, SetupStepItem, initial_setup_steps};
use crate::config::{AppConfig, ThemeMode};
use crate::connection::{ConnectionMode, ConnectionStatus};
use crate::error::AppError;
use crate::i18n::{I18n, Language, format_error_log};
use crate::state::AppState;
use crate::ui::components::app_header::render_app_header;
use crate::ui::components::sheet::render_sheet;
use crate::ui::theme::ThemePalette;
use chrono::Local;
use gpui::{
    AnyElement, App, Bounds, Context, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement, Render, SharedString, Styled, TitlebarOptions, Window, WindowAppearance,
    WindowBounds, WindowOptions, div, prelude::*, px, size,
};
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Mode,
    Prepare,
    Operate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ActiveSheet {
    Settings,
    Logs,
    ManualIp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AutoConnectStatus {
    Idle,
    Connecting,
    Success,
    Failed,
    Skipped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SetupRunStatus {
    Idle,
    Loading,
    Success,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ActiveInput {
    None,
    ManualIp,
    DimDelay,
    KeepAwakeInterval,
}

pub struct PicoSleepApp {
    pub(crate) state: Arc<AppState>,
    pub(crate) focus_handle: FocusHandle,
    pub(crate) config: AppConfig,
    pub(crate) language: Language,
    pub(crate) theme_mode: ThemeMode,
    pub(crate) screen: Screen,
    pub(crate) selected_mode_card: ConnectionMode,
    pub(crate) target_mode: ConnectionMode,
    pub(crate) active_sheet: Option<ActiveSheet>,
    pub(crate) auto_connect_status: AutoConnectStatus,
    pub(crate) auto_connect_attempt: usize,
    pub(crate) wireless_setup_status: SetupRunStatus,
    pub(crate) wired_setup_status: SetupRunStatus,
    pub(crate) setup_steps: Vec<SetupStepItem>,
    pub(crate) connection_status: Option<ConnectionStatus>,
    pub(crate) connection_error: Option<AppError>,
    pub(crate) runtime_error: Option<AppError>,
    pub(crate) show_error_detail: bool,
    pub(crate) is_busy: bool,
    pub(crate) is_running: bool,
    pub(crate) is_debug: bool,
    pub(crate) started_at: Option<Instant>,
    pub(crate) elapsed_seconds: u64,
    pub(crate) logs: Vec<String>,
    pub(crate) auto_triggered_wired: bool,
    pub(crate) last_window_title: String,

    // Buffered values in Settings Sheet
    pub(crate) pending_language: Language,
    pub(crate) pending_theme: ThemeMode,
    pub(crate) pending_dim_text: String,
    pub(crate) pending_interval_text: String,
    pub(crate) pending_debug: bool,

    // Manual IP input buffer
    pub(crate) manual_ip_text: String,
    pub(crate) active_input: ActiveInput,
}

impl Focusable for PicoSleepApp {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl PicoSleepApp {
    pub fn new(state: Arc<AppState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let config = commands::get_config(&state);
        let language = config.language.unwrap_or_else(Language::detect);
        let theme_mode = config.theme;
        let mode = config.last_connection_mode;
        let should_auto_connect = match mode {
            Some(ConnectionMode::Wired) => true,
            Some(ConnectionMode::Wireless) => !config.ip_address.trim().is_empty(),
            None => false,
        };
        let initial_screen = if mode.is_some() {
            Screen::Operate
        } else {
            Screen::Mode
        };
        let initial_target = mode.unwrap_or(ConnectionMode::Wireless);

        window
            .observe_window_appearance(|window, _cx| {
                window.refresh();
            })
            .detach();

        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle);

        let mut app = Self {
            state,
            focus_handle,
            pending_language: language,
            pending_theme: theme_mode,
            pending_dim_text: format_hours(config.dim_delay_hours),
            pending_interval_text: config.keep_awake_interval_secs.to_string(),
            pending_debug: false,
            manual_ip_text: config.ip_address.clone(),
            active_input: ActiveInput::None,
            config,
            language,
            theme_mode,
            screen: initial_screen,
            selected_mode_card: initial_target,
            target_mode: initial_target,
            active_sheet: None,
            auto_connect_status: if should_auto_connect {
                AutoConnectStatus::Idle
            } else {
                AutoConnectStatus::Skipped
            },
            auto_connect_attempt: 0,
            wireless_setup_status: SetupRunStatus::Idle,
            wired_setup_status: SetupRunStatus::Idle,
            setup_steps: initial_setup_steps(),
            connection_status: None,
            connection_error: None,
            runtime_error: None,
            show_error_detail: false,
            is_busy: false,
            is_running: false,
            is_debug: false,
            started_at: None,
            elapsed_seconds: 0,
            logs: Vec::new(),
            auto_triggered_wired: false,
            last_window_title: String::new(),
        };

        app.apply_preview_env();
        if std::env::var_os("PICO4U_PREVIEW").is_none() {
            app.start_background_loops(cx);
            if should_auto_connect {
                app.trigger_auto_connect(cx);
            }
        }
        app
    }

    fn apply_preview_env(&mut self) {
        let Ok(preview) = std::env::var("PICO4U_PREVIEW") else {
            return;
        };
        self.auto_connect_status = AutoConnectStatus::Skipped;
        self.language = Language::Ja;
        self.pending_language = Language::Ja;
        match preview.as_str() {
            "mode-light" => {
                self.theme_mode = ThemeMode::Light;
                self.screen = Screen::Mode;
                self.selected_mode_card = ConnectionMode::Wireless;
            }
            "mode-dark" => {
                self.theme_mode = ThemeMode::Dark;
                self.screen = Screen::Mode;
                self.selected_mode_card = ConnectionMode::Wireless;
            }
            "wired-light" => {
                self.theme_mode = ThemeMode::Light;
                self.screen = Screen::Operate;
                self.config.last_connection_mode = Some(ConnectionMode::Wired);
                self.connection_status = Some(ConnectionStatus {
                    is_usb_connected: true,
                    is_wifi_ip_available: false,
                    wifi_ip: None,
                    is_adb_tcp_connected: false,
                });
                self.is_debug = true;
            }
            "wired-dark" => {
                self.theme_mode = ThemeMode::Dark;
                self.screen = Screen::Operate;
                self.config.last_connection_mode = Some(ConnectionMode::Wired);
                self.connection_status = Some(ConnectionStatus {
                    is_usb_connected: true,
                    is_wifi_ip_available: false,
                    wifi_ip: None,
                    is_adb_tcp_connected: false,
                });
                self.is_debug = true;
            }
            "settings-light" => {
                self.theme_mode = ThemeMode::Light;
                self.pending_theme = ThemeMode::Light;
                self.screen = Screen::Mode;
                self.active_sheet = Some(ActiveSheet::Settings);
            }
            "settings-dark" => {
                self.theme_mode = ThemeMode::Dark;
                self.pending_theme = ThemeMode::Dark;
                self.screen = Screen::Mode;
                self.active_sheet = Some(ActiveSheet::Settings);
            }
            "mode-en-light" => {
                self.language = Language::En;
                self.pending_language = Language::En;
                self.theme_mode = ThemeMode::Light;
                self.screen = Screen::Mode;
                self.selected_mode_card = ConnectionMode::Wireless;
            }
            _ => {}
        }
    }

    pub(crate) fn i18n(&self) -> I18n {
        I18n::new(self.language)
    }

    pub(crate) fn add_log(&mut self, message: impl Into<String>) {
        let timestamp = Local::now().format("%H:%M:%S");
        let entry = format!("[{timestamp}] {}", message.into());
        self.logs.insert(0, entry);
        if self.logs.len() > 300 {
            self.logs.truncate(300);
        }
    }

    pub(crate) fn log_error(&mut self, err: &AppError) {
        let formatted = format_error_log(err, self.language, self.is_debug);
        self.add_log(formatted);
    }

    pub(crate) fn is_connected(&self) -> bool {
        self.connection_status
            .as_ref()
            .is_some_and(|s| s.is_connected(self.config.last_connection_mode))
    }

    pub(crate) fn reset_setup(&mut self) {
        self.setup_steps = initial_setup_steps();
        self.wireless_setup_status = SetupRunStatus::Idle;
        self.wired_setup_status = SetupRunStatus::Idle;
        self.connection_error = None;
        self.show_error_detail = false;
        self.auto_triggered_wired = false;
    }

    fn render_active_sheet(
        &self,
        sheet: ActiveSheet,
        palette: ThemePalette,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let i18n = self.i18n();
        let title = match sheet {
            ActiveSheet::Settings => i18n.settings_title(),
            ActiveSheet::Logs => i18n.tab_logs(),
            ActiveSheet::ManualIp => i18n.btn_connect_manual(),
        };

        let body = match sheet {
            ActiveSheet::Settings => self.render_settings_sheet(palette, cx),
            ActiveSheet::Logs => self.render_logs_sheet(),
            ActiveSheet::ManualIp => self.render_manual_ip_sheet(palette, cx),
        };

        render_sheet(
            title,
            body,
            palette,
            cx.listener(|this, _, _, cx| {
                this.active_sheet = None;
                this.active_input = ActiveInput::None;
                cx.notify();
            }),
        )
    }
}

impl Render for PicoSleepApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let suffix = if cfg!(debug_assertions) { "-dev" } else { "" };
        let desired_title = format!(
            "{} v{}{}",
            self.i18n().title(),
            env!("CARGO_PKG_VERSION"),
            suffix
        );
        if self.last_window_title != desired_title {
            self.last_window_title = desired_title.clone();
            window.set_window_title(&desired_title);
        }

        let is_dark = match self.theme_mode {
            ThemeMode::Dark => true,
            ThemeMode::Light => false,
            ThemeMode::System => matches!(
                window.appearance(),
                WindowAppearance::Dark | WindowAppearance::VibrantDark
            ),
        };
        let palette = ThemePalette::for_dark(is_dark);

        let show_auto_connect = matches!(
            self.auto_connect_status,
            AutoConnectStatus::Idle | AutoConnectStatus::Connecting | AutoConnectStatus::Failed
        );

        let header = render_app_header(
            palette,
            cx.listener(|this, _, _, cx| {
                this.open_settings_sheet(cx);
            }),
        );

        let main_content = if show_auto_connect {
            self.render_auto_connect_screen(palette, cx)
        } else {
            match self.screen {
                Screen::Mode => self.render_mode_select_screen(palette, cx),
                Screen::Prepare => self.render_prepare_screen(palette, cx),
                Screen::Operate => self.render_operate_screen(palette, cx),
            }
        };

        div()
            .id("root-container")
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down))
            .relative()
            .size_full()
            .bg(palette.bg)
            .text_color(palette.text)
            .flex()
            .flex_col()
            .child(header)
            .child(main_content)
            .when_some(self.active_sheet, |root, sheet| {
                root.child(self.render_active_sheet(sheet, palette, cx))
            })
    }
}

pub(crate) fn format_elapsed(sec: u64) -> String {
    let h = sec / 3600;
    let m = (sec % 3600) / 60;
    let s = sec % 60;
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

pub(crate) fn format_hours(hours: f64) -> String {
    if (hours - hours.round()).abs() < f64::EPSILON {
        format!("{}", hours.round() as i64)
    } else {
        format!("{hours}")
    }
}

pub fn window_options(cx: &App) -> WindowOptions {
    let suffix = if cfg!(debug_assertions) { "-dev" } else { "" };
    let title = format!("pico4u V睡ツール v{}{}", env!("CARGO_PKG_VERSION"), suffix);
    let bounds = Bounds::centered(None, size(px(400.0), px(700.0)), cx);
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: Some(TitlebarOptions {
            title: Some(SharedString::from(title)),
            ..Default::default()
        }),
        is_resizable: false,
        is_minimizable: true,
        app_id: Some("com.tauri.pico4uvrsleepworkaround".to_string()),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_elapsed_time_with_and_without_hours() {
        assert_eq!(format_elapsed(0), "0:00");
        assert_eq!(format_elapsed(65), "1:05");
        assert_eq!(format_elapsed(3661), "1:01:01");
    }

    #[test]
    fn formats_hours_compactly() {
        assert_eq!(format_hours(1.0), "1");
        assert_eq!(format_hours(0.0), "0");
        assert_eq!(format_hours(1.5), "1.5");
    }
}
