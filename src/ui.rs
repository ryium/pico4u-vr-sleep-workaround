use crate::commands::{
    self, SetupStep, SetupStepItem, StepState, initial_setup_steps, update_setup_steps,
};
use crate::config::{AppConfig, ThemeMode};
use crate::connection::{ConnectionMode, ConnectionStatus};
use crate::error::AppError;
use crate::i18n::{I18n, Language, format_error_log};
use crate::runtime;
use crate::state::AppState;
use chrono::Local;
use gpui::{
    AnyElement, App, AssetSource, Bounds, Context, FocusHandle, Focusable, FontWeight, Hsla,
    InteractiveElement, IntoElement, KeyDownEvent, MouseButton, ParentElement, Render,
    SharedString, StatefulInteractiveElement, Styled, TitlebarOptions, Window, WindowAppearance,
    WindowBounds, WindowOptions, div, prelude::*, px, rgb, rgba, size, svg,
};
use std::borrow::Cow;
use std::sync::Arc;
use std::time::{Duration, Instant};

const USB_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#000000" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 5v14"/><path d="M12 16.5 6.5 13v-2.5"/><path d="M12 14.5 17.5 11V8.5"/><polygon points="12,2 8.8,6 15.2,6" fill="#000000" stroke="none"/><circle cx="12" cy="20" r="2.2" fill="#000000" stroke="none"/><circle cx="6.5" cy="10" r="2" fill="#000000" stroke="none"/><rect x="15.6" y="6.6" width="3.8" height="3.8" rx="0.6" fill="#000000" stroke="none"/></svg>"##;

const WIFI_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#000000" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12.55a11 11 0 0 1 14.08 0"/><path d="M1.42 9a16 16 0 0 1 21.16 0"/><path d="M8.53 16.11a6 6 0 0 1 6.95 0"/><circle cx="12" cy="20" r="1.5" fill="#000000" stroke="none"/></svg>"##;

const SETTINGS_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#000000" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>"##;

const PLAY_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="#000000"><path d="M8 5v14l11-7z"/></svg>"##;

const MOON_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="#000000"><path d="M12 3a9 9 0 1 0 9 9c0-.46-.04-.92-.1-1.36a5.389 5.389 0 0 1-4.4 2.26 5.403 5.403 0 0 1-3.14-9.8c-.44-.06-.9-.1-1.36-.1z"/></svg>"##;

const ERROR_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="#000000"><path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd"/></svg>"##;

pub struct AppAssets;

impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        let content = match path {
            "icons/usb.svg" => Some(USB_SVG.as_bytes()),
            "icons/wifi.svg" => Some(WIFI_SVG.as_bytes()),
            "icons/settings.svg" => Some(SETTINGS_SVG.as_bytes()),
            "icons/play.svg" => Some(PLAY_SVG.as_bytes()),
            "icons/moon.svg" => Some(MOON_SVG.as_bytes()),
            "icons/error.svg" => Some(ERROR_SVG.as_bytes()),
            _ => None,
        };
        Ok(content.map(Cow::Borrowed))
    }

    fn list(&self, _path: &str) -> gpui::Result<Vec<SharedString>> {
        Ok(Vec::new())
    }
}

#[derive(Clone, Copy)]
struct ThemePalette {
    is_dark: bool,
    bg: Hsla,
    card: Hsla,
    text: Hsla,
    text_muted: Hsla,
    border: Hsla,
    brand: Hsla,
    brand_hover: Hsla,
    brand_tint: Hsla,
    subtle_bg: Hsla,
    subtle_border: Hsla,
    disabled_bg: Hsla,
    disabled_text: Hsla,
    error_bg: Hsla,
    error_border: Hsla,
    error_text: Hsla,
}

impl ThemePalette {
    fn for_dark(is_dark: bool) -> Self {
        if is_dark {
            Self {
                is_dark: true,
                bg: rgb(0x14181b).into(),
                card: rgb(0x1e2428).into(),
                text: rgb(0xe8ecea).into(),
                text_muted: rgb(0x9aa5a0).into(),
                border: rgb(0x2d353b).into(),
                brand: rgb(0x5cc489).into(),
                brand_hover: rgb(0x4fb379).into(),
                brand_tint: rgba(0x5cc48920).into(),
                subtle_bg: rgb(0x181d20).into(),
                subtle_border: rgb(0x2d353b).into(),
                disabled_bg: rgb(0x262d32).into(),
                disabled_text: rgb(0x6b7570).into(),
                error_bg: rgba(0x450a0a99).into(),
                error_border: rgb(0x7f1d1d).into(),
                error_text: rgb(0xfca5a5).into(),
            }
        } else {
            Self {
                is_dark: false,
                bg: rgb(0xf7f8f7).into(),
                card: rgb(0xffffff).into(),
                text: rgb(0x1f2421).into(),
                text_muted: rgb(0x6b7570).into(),
                border: rgb(0xe5e8e6).into(),
                brand: rgb(0x3fa66b).into(),
                brand_hover: rgb(0x2f8c57).into(),
                brand_tint: rgba(0x3fa66b18).into(),
                subtle_bg: rgb(0xf3f5f4).into(),
                subtle_border: rgb(0xe5e8e6).into(),
                disabled_bg: rgb(0xe5e7eb).into(),
                disabled_text: rgb(0x9ca3af).into(),
                error_bg: rgb(0xfef2f2).into(),
                error_border: rgb(0xfecaca).into(),
                error_text: rgb(0xb91c1c).into(),
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Screen {
    Mode,
    Prepare,
    Operate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActiveSheet {
    Settings,
    Logs,
    ManualIp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AutoConnectStatus {
    Idle,
    Connecting,
    Success,
    Failed,
    Skipped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SetupRunStatus {
    Idle,
    Loading,
    Success,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActiveInput {
    None,
    ManualIp,
    DimDelay,
    KeepAwakeInterval,
}

pub struct PicoSleepApp {
    state: Arc<AppState>,
    focus_handle: FocusHandle,
    config: AppConfig,
    language: Language,
    theme_mode: ThemeMode,
    screen: Screen,
    selected_mode_card: ConnectionMode,
    target_mode: ConnectionMode,
    active_sheet: Option<ActiveSheet>,
    auto_connect_status: AutoConnectStatus,
    auto_connect_attempt: usize,
    wireless_setup_status: SetupRunStatus,
    wired_setup_status: SetupRunStatus,
    setup_steps: Vec<SetupStepItem>,
    connection_status: Option<ConnectionStatus>,
    connection_error: Option<AppError>,
    runtime_error: Option<AppError>,
    show_error_detail: bool,
    is_busy: bool,
    is_running: bool,
    is_debug: bool,
    started_at: Option<Instant>,
    elapsed_seconds: u64,
    logs: Vec<String>,
    auto_triggered_wired: bool,
    last_window_title: String,

    // Pending settings in Settings Sheet
    pending_language: Language,
    pending_theme: ThemeMode,
    pending_dim_text: String,
    pending_interval_text: String,
    pending_debug: bool,

    // Manual IP prompt input
    manual_ip_text: String,
    active_input: ActiveInput,
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

        app.start_background_loops(cx);
        if should_auto_connect {
            app.trigger_auto_connect(cx);
        }
        app
    }

    fn i18n(&self) -> I18n {
        I18n::new(self.language)
    }

    fn add_log(&mut self, message: impl Into<String>) {
        let timestamp = Local::now().format("%H:%M:%S");
        let entry = format!("[{timestamp}] {}", message.into());
        self.logs.insert(0, entry);
        if self.logs.len() > 300 {
            self.logs.truncate(300);
        }
    }

    fn log_error(&mut self, err: &AppError) {
        let formatted = format_error_log(err, self.language, self.is_debug);
        self.add_log(formatted);
    }

    fn is_connected(&self) -> bool {
        self.connection_status
            .as_ref()
            .is_some_and(|s| s.is_connected(self.config.last_connection_mode))
    }

    fn reset_setup(&mut self) {
        self.setup_steps = initial_setup_steps();
        self.wireless_setup_status = SetupRunStatus::Idle;
        self.wired_setup_status = SetupRunStatus::Idle;
        self.connection_error = None;
        self.show_error_detail = false;
        self.auto_triggered_wired = false;
    }

    fn start_background_loops(&self, cx: &mut Context<Self>) {
        let state_for_tick = Arc::clone(&self.state);
        cx.spawn(async move |view, cx| {
            loop {
                gpui::Timer::after(Duration::from_secs(1)).await;
                let drained = state_for_tick.drain_debug_logs();
                let running = runtime::get_running(&state_for_tick);
                let alive = view
                    .update(cx, |this, cx| {
                        let mut changed = false;
                        for msg in drained {
                            this.add_log(msg);
                            changed = true;
                        }
                        if this.is_running != running {
                            this.is_running = running;
                            if !running {
                                this.started_at = None;
                                this.elapsed_seconds = 0;
                            }
                            changed = true;
                        }
                        if this.is_running
                            && let Some(started) = this.started_at
                        {
                            let secs = started.elapsed().as_secs();
                            if secs != this.elapsed_seconds {
                                this.elapsed_seconds = secs;
                                changed = true;
                            }
                        }
                        if changed {
                            cx.notify();
                        }
                    })
                    .is_ok();
                if !alive {
                    break;
                }
            }
        })
        .detach();

        let state_for_poll = Arc::clone(&self.state);
        cx.spawn(async move |view, cx| {
            loop {
                let query = view
                    .update(cx, |this, _cx| {
                        if this.is_busy {
                            None
                        } else {
                            Some((
                                this.config.last_connection_mode,
                                this.config.ip_address.clone(),
                            ))
                        }
                    })
                    .ok()
                    .flatten();

                if let Some((mode, ip)) = query {
                    let st = Arc::clone(&state_for_poll);
                    let res = state_for_poll
                        .runtime
                        .spawn(async move {
                            commands::check_connection_status(&st, mode, Some(ip)).await
                        })
                        .await;

                    if let Ok(Ok(status)) = res {
                        let alive = view
                            .update(cx, |this, cx| {
                                if !this.is_busy {
                                    let connected =
                                        status.is_connected(this.config.last_connection_mode);
                                    if connected {
                                        this.connection_error = None;
                                    }
                                    if !status.is_usb_connected {
                                        this.auto_triggered_wired = false;
                                    }
                                    let should_auto_verify_wired = this.screen == Screen::Prepare
                                        && this.target_mode == ConnectionMode::Wired
                                        && status.is_usb_connected
                                        && !this.is_busy
                                        && this.connection_error.is_none()
                                        && !this.auto_triggered_wired;
                                    this.connection_status = Some(status);
                                    if should_auto_verify_wired {
                                        this.auto_triggered_wired = true;
                                        this.trigger_check_devices(cx);
                                    }
                                    cx.notify();
                                }
                            })
                            .is_ok();
                        if !alive {
                            break;
                        }
                    }
                }

                gpui::Timer::after(Duration::from_secs(2)).await;
            }
        })
        .detach();
    }

    fn trigger_auto_connect(&mut self, cx: &mut Context<Self>) {
        if self.is_busy || self.is_running {
            return;
        }
        let mode = self
            .config
            .last_connection_mode
            .unwrap_or(ConnectionMode::Wired);
        let ip = self.config.ip_address.trim().to_string();
        self.is_busy = true;
        self.connection_error = None;
        self.show_error_detail = false;
        self.auto_connect_status = AutoConnectStatus::Connecting;
        self.auto_connect_attempt = 1;
        self.setup_steps = initial_setup_steps();
        cx.notify();

        let state = Arc::clone(&self.state);
        cx.spawn(async move |view, cx| {
            let mut last_err = None;
            let mut succeeded = false;

            for attempt in 1..=3 {
                let _ = view.update(cx, |this, cx| {
                    this.auto_connect_attempt = attempt;
                    cx.notify();
                });

                let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
                let progress_view = view.clone();
                let mut progress_cx = cx.clone();
                let progress_task = cx.foreground_executor().spawn(async move {
                    while let Some(progress) = rx.recv().await {
                        let _ = progress_view.update(&mut progress_cx, |this, cx| {
                            this.apply_setup_progress(progress);
                            cx.notify();
                        });
                    }
                });

                let st = Arc::clone(&state);
                let ip_clone = ip.clone();
                let join = state.runtime.spawn(async move {
                    commands::try_auto_connect(&st, mode, ip_clone, move |p| {
                        let _ = tx.send(p);
                    })
                    .await
                });

                let outcome = join.await;
                progress_task.await;

                match outcome {
                    Ok(Ok(_)) => {
                        succeeded = true;
                        break;
                    }
                    Ok(Err(err)) => {
                        last_err = Some(err);
                    }
                    Err(e) => {
                        last_err = Some(AppError::from(e.to_string()));
                    }
                }

                if attempt < 3 {
                    gpui::Timer::after(Duration::from_secs(2)).await;
                }
            }

            let st = Arc::clone(&state);
            let ip_for_status = ip.clone();
            let status_res = state
                .runtime
                .spawn(async move {
                    commands::check_connection_status(&st, Some(mode), Some(ip_for_status)).await
                })
                .await
                .ok()
                .and_then(Result::ok);

            let _ = view.update(cx, |this, cx| {
                this.is_busy = false;
                if let Some(st) = status_res {
                    this.connection_status = Some(st);
                }
                if succeeded {
                    this.auto_connect_status = AutoConnectStatus::Success;
                    this.connection_error = None;
                    this.target_mode = mode;
                    this.screen = Screen::Operate;
                    let target_label = if mode == ConnectionMode::Wireless && !ip.is_empty() {
                        ip.clone()
                    } else {
                        "USB".to_string()
                    };
                    let msg = this.i18n().log_auto_connect_success(&target_label);
                    this.add_log(msg);
                } else {
                    this.auto_connect_status = AutoConnectStatus::Failed;
                    if let Some(err) = last_err {
                        this.log_error(&err);
                        this.connection_error = Some(err);
                    }
                    let target_label = if mode == ConnectionMode::Wireless && !ip.is_empty() {
                        ip
                    } else {
                        "USB".to_string()
                    };
                    let msg = this.i18n().log_auto_connect_failed(&target_label);
                    this.add_log(msg);
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn apply_setup_progress(&mut self, progress: commands::SetupProgress) {
        if progress.state == StepState::Failed {
            let step_label = self.i18n().setup_step_label(progress.step);
            let detail_str = progress
                .detail
                .as_deref()
                .map(|d| format!(" ({d})"))
                .unwrap_or_default();
            self.add_log(format!("{step_label}: failed{detail_str}"));
        }
        self.setup_steps = update_setup_steps(&self.setup_steps, &progress);
    }

    fn trigger_check_devices(&mut self, cx: &mut Context<Self>) {
        if self.is_busy || self.is_running {
            return;
        }
        self.setup_steps = initial_setup_steps();
        self.wired_setup_status = SetupRunStatus::Loading;
        self.connection_error = None;
        self.show_error_detail = false;
        self.is_busy = true;
        let log_msg = self.i18n().log_checking_devices();
        self.add_log(log_msg);
        cx.notify();

        let state = Arc::clone(&self.state);
        let ip = self.config.ip_address.clone();
        cx.spawn(async move |view, cx| {
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
            let progress_view = view.clone();
            let mut progress_cx = cx.clone();
            let progress_task = cx.foreground_executor().spawn(async move {
                while let Some(progress) = rx.recv().await {
                    let _ = progress_view.update(&mut progress_cx, |this, cx| {
                        this.apply_setup_progress(progress);
                        cx.notify();
                    });
                }
            });

            let st = Arc::clone(&state);
            let ip_clone = ip.clone();
            let outcome = state
                .runtime
                .spawn(async move {
                    let res = commands::try_auto_connect(
                        &st,
                        ConnectionMode::Wired,
                        ip_clone.clone(),
                        move |p| {
                            let _ = tx.send(p);
                        },
                    )
                    .await?;
                    let mut cfg = commands::get_config(&st);
                    cfg.last_connection_mode = Some(ConnectionMode::Wired);
                    commands::save_config_cmd(&st, cfg.clone()).await?;
                    let status = commands::check_connection_status(
                        &st,
                        Some(ConnectionMode::Wired),
                        Some(cfg.ip_address.clone()),
                    )
                    .await?;
                    Ok::<_, AppError>((res, cfg, status))
                })
                .await;

            progress_task.await;

            let _ = view.update(cx, |this, cx| {
                this.is_busy = false;
                match outcome {
                    Ok(Ok((_mode, cfg, status))) => {
                        this.config = cfg;
                        this.connection_status = Some(status);
                        this.wired_setup_status = SetupRunStatus::Success;
                        this.connection_error = None;
                        let msg = this.i18n().wired_status_success();
                        this.add_log(msg);
                        this.screen = Screen::Operate;
                    }
                    Ok(Err(err)) => {
                        this.wired_setup_status = SetupRunStatus::Error;
                        this.log_error(&err);
                        this.connection_error = Some(err);
                    }
                    Err(e) => {
                        let err = AppError::from(e.to_string());
                        this.wired_setup_status = SetupRunStatus::Error;
                        this.log_error(&err);
                        this.connection_error = Some(err);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn trigger_setup_wireless(&mut self, cx: &mut Context<Self>) {
        if self.is_busy || self.is_running {
            return;
        }
        self.setup_steps = initial_setup_steps();
        self.wireless_setup_status = SetupRunStatus::Loading;
        self.connection_error = None;
        self.show_error_detail = false;
        self.is_busy = true;
        let log_msg = self.i18n().log_wireless_init();
        self.add_log(log_msg);
        cx.notify();

        let state = Arc::clone(&self.state);
        cx.spawn(async move |view, cx| {
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
            let progress_view = view.clone();
            let mut progress_cx = cx.clone();
            let progress_task = cx.foreground_executor().spawn(async move {
                while let Some(progress) = rx.recv().await {
                    let _ = progress_view.update(&mut progress_cx, |this, cx| {
                        this.apply_setup_progress(progress);
                        cx.notify();
                    });
                }
            });

            let st = Arc::clone(&state);
            let outcome = state
                .runtime
                .spawn(async move {
                    let setup_res = commands::setup_wireless(&st, move |p| {
                        let _ = tx.send(p);
                    })
                    .await?;
                    let mut cfg = commands::get_config(&st);
                    cfg.last_connection_mode = Some(ConnectionMode::Wireless);
                    cfg.ip_address = setup_res.ip.clone();
                    commands::save_config_cmd(&st, cfg.clone()).await?;
                    let status = commands::check_connection_status(
                        &st,
                        Some(ConnectionMode::Wireless),
                        Some(cfg.ip_address.clone()),
                    )
                    .await?;
                    Ok::<_, AppError>((setup_res, cfg, status))
                })
                .await;

            progress_task.await;

            let _ = view.update(cx, |this, cx| {
                this.is_busy = false;
                match outcome {
                    Ok(Ok((_outcome, cfg, status))) => {
                        this.config = cfg;
                        this.connection_status = Some(status);
                        this.wireless_setup_status = SetupRunStatus::Success;
                        this.connection_error = None;
                        let msg = this.i18n().log_wireless_complete();
                        this.add_log(msg);
                        this.screen = Screen::Operate;
                    }
                    Ok(Err(err)) => {
                        this.wireless_setup_status = SetupRunStatus::Error;
                        this.log_error(&err);
                        this.connection_error = Some(err);
                    }
                    Err(e) => {
                        let err = AppError::from(e.to_string());
                        this.wireless_setup_status = SetupRunStatus::Error;
                        this.log_error(&err);
                        this.connection_error = Some(err);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn trigger_connect_manual(&mut self, ip: String, cx: &mut Context<Self>) {
        let trimmed = ip.trim().to_string();
        if trimmed.is_empty() || self.is_busy || self.is_running {
            return;
        }
        self.active_sheet = None;
        self.active_input = ActiveInput::None;
        self.is_busy = true;
        self.connection_error = None;
        self.show_error_detail = false;
        let log_msg = self.i18n().log_connecting_manual(&trimmed);
        self.add_log(log_msg);
        cx.notify();

        let state = Arc::clone(&self.state);
        cx.spawn(async move |view, cx| {
            let st = Arc::clone(&state);
            let target_ip = trimmed.clone();
            let outcome = state
                .runtime
                .spawn(async move {
                    commands::try_auto_connect(
                        &st,
                        ConnectionMode::Wireless,
                        target_ip.clone(),
                        |_| {},
                    )
                    .await?;
                    let mut cfg = commands::get_config(&st);
                    cfg.last_connection_mode = Some(ConnectionMode::Wireless);
                    cfg.ip_address = target_ip.clone();
                    commands::save_config_cmd(&st, cfg.clone()).await?;
                    let status = commands::check_connection_status(
                        &st,
                        Some(ConnectionMode::Wireless),
                        Some(target_ip),
                    )
                    .await?;
                    Ok::<_, AppError>((cfg, status))
                })
                .await;

            let _ = view.update(cx, |this, cx| {
                this.is_busy = false;
                match outcome {
                    Ok(Ok((cfg, status))) => {
                        this.config = cfg;
                        this.connection_status = Some(status);
                        this.connection_error = None;
                        let msg = this.i18n().log_auto_connect_success(&trimmed);
                        this.add_log(msg);
                        this.screen = Screen::Operate;
                    }
                    Ok(Err(err)) => {
                        this.wireless_setup_status = SetupRunStatus::Error;
                        this.log_error(&err);
                        this.connection_error = Some(err);
                    }
                    Err(e) => {
                        let err = AppError::from(e.to_string());
                        this.wireless_setup_status = SetupRunStatus::Error;
                        this.log_error(&err);
                        this.connection_error = Some(err);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn trigger_toggle_keep_awake(&mut self, cx: &mut Context<Self>) {
        if self.is_busy {
            return;
        }
        let was_running = self.is_running;
        let mode = self.config.last_connection_mode;
        if !was_running && (mode.is_none() || !self.is_connected()) {
            return;
        }
        self.is_busy = true;
        self.runtime_error = None;
        cx.notify();

        let state = Arc::clone(&self.state);
        cx.spawn(async move |view, cx| {
            let st = Arc::clone(&state);
            let outcome = state
                .runtime
                .spawn(async move {
                    if was_running {
                        runtime::stop_keep_awake(&st).await
                    } else {
                        runtime::start_keep_awake(&st, mode.unwrap()).await
                    }
                })
                .await;

            let running_now = runtime::get_running(&state);
            let _ = view.update(cx, |this, cx| {
                this.is_busy = false;
                this.is_running = running_now;
                match outcome {
                    Ok(Ok(())) => {
                        this.runtime_error = None;
                        if running_now {
                            this.started_at = Some(Instant::now());
                            this.elapsed_seconds = 0;
                            let interval = this.config.keep_awake_interval_secs;
                            let msg = this.i18n().log_started_loop(interval);
                            this.add_log(msg);
                        } else {
                            this.started_at = None;
                            this.elapsed_seconds = 0;
                            let msg = this.i18n().log_stopped_loop();
                            this.add_log(msg);
                        }
                    }
                    Ok(Err(err)) => {
                        this.log_error(&err);
                        this.runtime_error = Some(err);
                    }
                    Err(e) => {
                        let err = AppError::from(e.to_string());
                        this.log_error(&err);
                        this.runtime_error = Some(err);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn open_settings_sheet(&mut self, cx: &mut Context<Self>) {
        self.pending_language = self.language;
        self.pending_theme = self.theme_mode;
        self.pending_dim_text = format_hours(self.config.dim_delay_hours);
        self.pending_interval_text = self.config.keep_awake_interval_secs.to_string();
        self.pending_debug = self.is_debug;
        self.active_input = ActiveInput::None;
        self.active_sheet = Some(ActiveSheet::Settings);
        cx.notify();
    }

    fn apply_settings(&mut self, cx: &mut Context<Self>) {
        let parsed_dim = self
            .pending_dim_text
            .trim()
            .parse::<f64>()
            .unwrap_or(self.config.dim_delay_hours)
            .clamp(0.0, 168.0);
        let parsed_interval = self
            .pending_interval_text
            .trim()
            .parse::<u64>()
            .unwrap_or(self.config.keep_awake_interval_secs)
            .clamp(1, 3600);

        let new_lang = self.pending_language;
        let new_theme = self.pending_theme;
        let new_debug = self.pending_debug;
        let debug_changed = new_debug != self.is_debug;

        self.language = new_lang;
        self.theme_mode = new_theme;
        self.is_debug = new_debug;
        commands::set_debug_mode(&self.state, new_debug);
        if debug_changed {
            let msg = if new_debug {
                self.i18n().log_debug_enabled()
            } else {
                self.i18n().log_debug_disabled()
            };
            self.add_log(msg);
        }

        let mut next_config = self.config.clone();
        next_config.dim_delay_hours = parsed_dim;
        next_config.keep_awake_interval_secs = parsed_interval;
        next_config.language = Some(new_lang);
        next_config.theme = new_theme;

        self.config = next_config.clone();
        self.active_sheet = None;
        self.active_input = ActiveInput::None;
        cx.notify();

        let state = Arc::clone(&self.state);
        cx.spawn(async move |_view, _cx| {
            let st = Arc::clone(&state);
            let _ = state
                .runtime
                .spawn(async move { commands::save_config_cmd(&st, next_config).await })
                .await;
        })
        .detach();
    }

    fn handle_key_down(
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

    fn render_header(&self, palette: ThemePalette, cx: &mut Context<Self>) -> AnyElement {
        div()
            .w_full()
            .flex()
            .justify_end()
            .items_center()
            .px_4()
            .pt_3()
            .child(
                div()
                    .id("btn-open-settings")
                    .p(px(6.0))
                    .rounded_lg()
                    .cursor_pointer()
                    .hover(|s| {
                        s.bg(if palette.is_dark {
                            rgba(0xffffff12)
                        } else {
                            rgba(0x0000000d)
                        })
                    })
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.open_settings_sheet(cx);
                    }))
                    .child(
                        svg()
                            .path("icons/settings.svg")
                            .size(px(20.0))
                            .text_color(palette.text_muted),
                    ),
            )
            .into_any_element()
    }

    fn render_step_indicator(&self, current: Screen, palette: ThemePalette) -> AnyElement {
        let i18n = self.i18n();
        let steps = [
            (0usize, i18n.step_mode()),
            (1usize, i18n.step_prepare()),
            (2usize, i18n.step_operate()),
        ];
        let current_idx = match current {
            Screen::Mode => 0,
            Screen::Prepare => 1,
            Screen::Operate => 2,
        };

        div()
            .w_full()
            .flex()
            .items_center()
            .justify_between()
            .px_8()
            .py(px(6.0))
            .children(steps.into_iter().map(|(idx, label)| {
                let is_done = idx < current_idx;
                let is_active = idx == current_idx;
                let dot_bg = if is_active || is_done {
                    palette.brand
                } else {
                    palette.disabled_bg
                };
                let label_color = if is_active {
                    palette.brand
                } else if is_done {
                    palette.text
                } else {
                    palette.text_muted
                };

                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(6.0))
                    .child(
                        div()
                            .w(px(18.0))
                            .h(px(18.0))
                            .rounded_full()
                            .bg(dot_bg)
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(10.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(gpui::white())
                            .child(if is_done { "✓" } else { "" }),
                    )
                    .child(
                        div()
                            .text_size(px(11.0))
                            .font_weight(if is_active {
                                FontWeight::SEMIBOLD
                            } else {
                                FontWeight::MEDIUM
                            })
                            .text_color(label_color)
                            .child(label),
                    )
            }))
            .into_any_element()
    }

    fn render_error_card(
        &self,
        error: Option<&AppError>,
        fallback_message: Option<String>,
        palette: ThemePalette,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let i18n = self.i18n();
        let (title, hints, detail) = if let Some(err) = error {
            (
                i18n.error_title(err.code).to_string(),
                i18n.error_hints(err.code),
                err.detail.clone(),
            )
        } else if let Some(msg) = fallback_message {
            (msg, &[][..], None)
        } else {
            return div().into_any_element();
        };

        let show_detail = self.show_error_detail;

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_2()
            .rounded_lg()
            .p_3()
            .bg(palette.error_bg)
            .border_1()
            .border_color(palette.error_border)
            .text_color(palette.error_text)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .text_size(px(13.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(
                        svg()
                            .path("icons/error.svg")
                            .size(px(16.0))
                            .text_color(palette.error_text),
                    )
                    .child(div().flex_1().child(title)),
            )
            .when(!hints.is_empty(), |el| {
                el.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .pl_2()
                        .children(hints.iter().map(|hint| {
                            div()
                                .text_size(px(11.5))
                                .line_height(px(16.0))
                                .child(format!("• {hint}"))
                        })),
                )
            })
            .when_some(detail.filter(|d| !d.is_empty()), |el, detail_text| {
                el.child(
                    div()
                        .mt_1()
                        .pt_1()
                        .border_t_1()
                        .border_color(palette.error_border)
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .id("btn-toggle-error-detail")
                                .cursor_pointer()
                                .text_size(px(11.0))
                                .underline()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.show_error_detail = !this.show_error_detail;
                                    cx.notify();
                                }))
                                .child(format!(
                                    "{} {}",
                                    i18n.details_toggle(),
                                    if show_detail { "▲" } else { "▼" }
                                )),
                        )
                        .when(show_detail, |d| {
                            d.child(
                                div()
                                    .p(px(6.0))
                                    .rounded_sm()
                                    .bg(rgba(0x00000033))
                                    .text_size(px(10.0))
                                    .child(detail_text),
                            )
                        }),
                )
            })
            .into_any_element()
    }

    fn render_auto_connect_view(
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

        div()
            .flex_1()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .px_5()
            .child(
                div()
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
                    .when(is_connecting, |card| {
                        card.child(
                            div()
                                .py_4()
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap_3()
                                .child(
                                    div()
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
                                        .child("⟳"),
                                )
                                .child(
                                    div()
                                        .text_size(px(13.5))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(palette.text)
                                        .text_center()
                                        .child(status_label),
                                )
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
                                ),
                        )
                    })
                    .when(
                        self.auto_connect_status == AutoConnectStatus::Failed,
                        |card| {
                            let fallback = i18n.auto_connect_failed_desc(&ip);
                            card.child(self.render_error_card(
                                self.connection_error.as_ref(),
                                Some(fallback),
                                palette,
                                cx,
                            ))
                            .child(
                                div()
                                    .w_full()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .mt_1()
                                    .child(primary_button(
                                        "btn-auto-retry",
                                        i18n.btn_try_again(),
                                        false,
                                        palette,
                                        cx.listener(|this, _, _, cx| {
                                            this.trigger_auto_connect(cx);
                                        }),
                                    ))
                                    .child(secondary_button(
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
                                    ))
                                    .child(
                                        div()
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
                                                this.auto_connect_status =
                                                    AutoConnectStatus::Skipped;
                                                this.screen = Screen::Mode;
                                                cx.notify();
                                            }))
                                            .child(i18n.btn_manual_setup()),
                                    ),
                            )
                        },
                    ),
            )
            .into_any_element()
    }

    fn render_mode_select_screen(
        &self,
        palette: ThemePalette,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let i18n = self.i18n();
        let selected = self.selected_mode_card;
        let can_go_back = self.is_connected();

        div()
            .flex_1()
            .flex()
            .flex_col()
            .justify_between()
            .px_5()
            .pt_2()
            .pb_4()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .child(self.render_step_indicator(Screen::Mode, palette))
                    .child(
                        div()
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
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .child(self.render_choice_card(
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
                            ))
                            .child(self.render_choice_card(
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
                            )),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .pt_4()
                    .pb_2()
                    .child(primary_button(
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
                    ))
                    .when(can_go_back, |el| {
                        el.child(
                            div()
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
                                .child(i18n.btn_back()),
                        )
                    }),
            )
            .into_any_element()
    }

    #[allow(clippy::too_many_arguments)]
    fn render_choice_card(
        &self,
        id: &'static str,
        selected: bool,
        icon_path: &'static str,
        title: &str,
        description: &str,
        badge: Option<&str>,
        palette: ThemePalette,
        on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
    ) -> AnyElement {
        let border_col = if selected {
            palette.brand
        } else {
            palette.border
        };
        let bg_col = if selected {
            palette.brand_tint
        } else {
            palette.card
        };
        let icon_bg = if selected {
            palette.brand_tint
        } else {
            palette.subtle_bg
        };
        let icon_col = if selected {
            palette.brand
        } else {
            palette.text_muted
        };

        div()
            .id(id)
            .w_full()
            .p_4()
            .rounded_xl()
            .border_1()
            .border_color(border_col)
            .bg(bg_col)
            .cursor_pointer()
            .on_click(on_click)
            .child(
                div()
                    .w_full()
                    .flex()
                    .items_start()
                    .justify_between()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .items_start()
                            .gap_3()
                            .flex_1()
                            .child(
                                div().p(px(10.0)).rounded_lg().bg(icon_bg).child(
                                    svg().path(icon_path).size(px(22.0)).text_color(icon_col),
                                ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .flex_1()
                                    .child(
                                        div()
                                            .text_size(px(14.0))
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(palette.text)
                                            .child(title.to_string()),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(12.0))
                                            .line_height(px(17.0))
                                            .text_color(palette.text_muted)
                                            .child(description.to_string()),
                                    ),
                            ),
                    )
                    .when_some(badge, |el, badge_text| {
                        el.child(
                            div()
                                .px_2()
                                .py(px(2.0))
                                .rounded_full()
                                .bg(palette.brand)
                                .text_size(px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(gpui::white())
                                .child(badge_text.to_string()),
                        )
                    }),
            )
            .into_any_element()
    }

    fn render_prepare_screen(&self, palette: ThemePalette, cx: &mut Context<Self>) -> AnyElement {
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

        let display_steps: Vec<SetupStepItem> = self
            .setup_steps
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
            .collect();

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
            .child(
                div()
                    .flex()
                    .flex_col()
                    .child(self.render_step_indicator(Screen::Prepare, palette))
                    .child(
                        div()
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
                            ),
                    )
                    .child(self.render_setup_checklist(&display_steps, palette))
                    .when(is_error, |el| {
                        let fallback = if is_wireless {
                            i18n.wireless_status_error().to_string()
                        } else {
                            i18n.wired_status_error().to_string()
                        };
                        el.child(div().mt_3().child(self.render_error_card(
                            self.connection_error.as_ref(),
                            Some(fallback),
                            palette,
                            cx,
                        )))
                    })
                    .when(is_wireless, |el| {
                        el.child(
                            div()
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
                                .child(i18n.btn_connect_manual()),
                        )
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .pt_3()
                    .pb_2()
                    .child(primary_button(
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
                    ))
                    .child(
                        div()
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
                            .child(i18n.btn_back()),
                    ),
            )
            .into_any_element()
    }

    fn render_setup_checklist(&self, steps: &[SetupStepItem], palette: ThemePalette) -> AnyElement {
        let i18n = self.i18n();
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
            .children(steps.iter().map(|item| {
                let (icon, icon_col, text_col, status_right) = match item.state {
                    StepState::Running => ("⟳", palette.brand, palette.text_muted, "..."),
                    StepState::Ok => ("✓", palette.brand, palette.text, "OK"),
                    StepState::Failed => ("✕", palette.error_text, palette.error_text, "NG"),
                    StepState::Idle | StepState::Skipped => {
                        ("○", palette.text_muted, palette.text_muted, "-")
                    }
                };

                div()
                    .w_full()
                    .py_1()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
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
                            ),
                    )
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(palette.text_muted)
                            .child(status_right),
                    )
            }))
            .into_any_element()
    }

    fn render_operate_screen(&self, palette: ThemePalette, cx: &mut Context<Self>) -> AnyElement {
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

        let chip_label = if connected {
            if mode == Some(ConnectionMode::Wireless) {
                if !ip.is_empty() {
                    i18n.chip_wireless_connected_ip(ip)
                } else {
                    i18n.chip_wireless_connected().to_string()
                }
            } else {
                i18n.chip_wired_connected().to_string()
            }
        } else {
            i18n.chip_disconnected().to_string()
        };

        let chip_icon = if mode == Some(ConnectionMode::Wireless) {
            "icons/wifi.svg"
        } else {
            "icons/usb.svg"
        };

        let power_disabled = (!connected && !is_running) || self.is_busy;
        let power_bg = if power_disabled {
            palette.disabled_bg
        } else if is_running {
            palette.card
        } else {
            palette.brand
        };
        let power_text_col = if power_disabled {
            palette.disabled_text
        } else if is_running {
            palette.brand
        } else {
            gpui::white()
        };

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

        div()
            .flex_1()
            .flex()
            .flex_col()
            .justify_between()
            .items_center()
            .px_5()
            .py_4()
            // Top Connection Chip
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .px_3()
                            .py(px(6.0))
                            .rounded_full()
                            .bg(palette.subtle_bg)
                            .border_1()
                            .border_color(palette.border)
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().w(px(8.0)).h(px(8.0)).rounded_full().bg(if connected {
                                palette.brand
                            } else {
                                palette.disabled_text
                            }))
                            .child(
                                svg()
                                    .path(chip_icon)
                                    .size(px(14.0))
                                    .text_color(palette.text_muted),
                            )
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(palette.text)
                                    .child(chip_label),
                            ),
                    )
                    .child(
                        div()
                            .h(px(18.0))
                            .text_size(px(11.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(palette.brand)
                            .text_center()
                            .child(if show_wireless_hint {
                                if is_already_wireless {
                                    i18n.setup_already_wireless()
                                } else {
                                    i18n.wireless_status_success()
                                }
                            } else {
                                ""
                            }),
                    ),
            )
            // Center Power Button & Status
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .id("btn-power-toggle")
                            .w(px(176.0))
                            .h(px(176.0))
                            .rounded_full()
                            .bg(power_bg)
                            .when(is_running && !power_disabled, |b| {
                                b.border(px(4.0)).border_color(palette.brand)
                            })
                            .when(!power_disabled, |b| {
                                b.cursor_pointer()
                                    .hover(|s| {
                                        if is_running {
                                            s.bg(palette.brand_tint)
                                        } else {
                                            s.bg(palette.brand_hover)
                                        }
                                    })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.trigger_toggle_keep_awake(cx);
                                    }))
                            })
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .child(
                                div().mb_2().child(
                                    svg()
                                        .path(if is_running {
                                            "icons/moon.svg"
                                        } else {
                                            "icons/play.svg"
                                        })
                                        .size(px(40.0))
                                        .text_color(power_text_col),
                                ),
                            )
                            .child(
                                div()
                                    .text_size(px(20.0))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(power_text_col)
                                    .child(if is_running {
                                        i18n.status_running()
                                    } else {
                                        i18n.power_btn_start()
                                    }),
                            )
                            .child(
                                div()
                                    .mt(px(6.0))
                                    .text_size(px(11.0))
                                    .text_color(power_text_col)
                                    .opacity(0.85)
                                    .child(if is_running {
                                        i18n.power_btn_tap_to_stop()
                                    } else {
                                        i18n.power_btn_tap_to_start()
                                    }),
                            ),
                    )
                    .child(
                        div()
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
                            ),
                    )
                    .when(self.runtime_error.is_some() && !is_running, |el| {
                        el.child(div().mt_3().w_full().child(self.render_error_card(
                            self.runtime_error.as_ref(),
                            None,
                            palette,
                            cx,
                        )))
                    }),
            )
            // Bottom Links
            .child(
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
                    .child(
                        div()
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
                            .child(i18n.btn_change_connection()),
                    )
                    .when(self.is_debug, |el| {
                        el.child(
                            div()
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
                                .child(i18n.tab_logs()),
                        )
                    }),
            )
            .into_any_element()
    }

    fn render_sheet_overlay(
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
            ActiveSheet::Logs => self.render_logs_sheet(palette),
            ActiveSheet::ManualIp => self.render_manual_ip_sheet(palette, cx),
        };

        div()
            .id("sheet-backdrop")
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .bg(rgba(0x00000080))
            .flex()
            .flex_col()
            .justify_end()
            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                cx.stop_propagation();
            })
            .child(
                div()
                    .w_full()
                    .max_h(px(620.0))
                    .rounded_t_2xl()
                    .bg(palette.card)
                    .border_t_1()
                    .border_color(palette.border)
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .px_5()
                            .py_3()
                            .border_b_1()
                            .border_color(palette.border)
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_size(px(14.0))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(palette.text)
                                    .child(title),
                            )
                            .child(
                                div()
                                    .id("btn-close-sheet")
                                    .px_2()
                                    .py_1()
                                    .rounded_md()
                                    .cursor_pointer()
                                    .text_size(px(14.0))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(palette.text_muted)
                                    .hover(|s| s.text_color(palette.text))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.active_sheet = None;
                                        this.active_input = ActiveInput::None;
                                        cx.notify();
                                    }))
                                    .child("✕"),
                            ),
                    )
                    .child(
                        div()
                            .id("sheet-body-scroll")
                            .p_5()
                            .overflow_y_scroll()
                            .child(body),
                    ),
            )
            .into_any_element()
    }

    fn render_settings_sheet(&self, palette: ThemePalette, cx: &mut Context<Self>) -> AnyElement {
        let i18n = I18n::new(self.pending_language);
        let dim_focused = self.active_input == ActiveInput::DimDelay;
        let interval_focused = self.active_input == ActiveInput::KeepAwakeInterval;

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_4()
            // Language selector
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_size(px(12.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(palette.text)
                            .child(i18n.language_label()),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(segment_option(
                                "opt-lang-ja",
                                "日本語",
                                self.pending_language == Language::Ja,
                                palette,
                                cx.listener(|this, _, _, cx| {
                                    this.pending_language = Language::Ja;
                                    cx.notify();
                                }),
                            ))
                            .child(segment_option(
                                "opt-lang-en",
                                "English",
                                self.pending_language == Language::En,
                                palette,
                                cx.listener(|this, _, _, cx| {
                                    this.pending_language = Language::En;
                                    cx.notify();
                                }),
                            )),
                    ),
            )
            // Theme selector
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_size(px(12.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(palette.text)
                            .child(i18n.theme_label()),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(segment_option(
                                "opt-theme-light",
                                i18n.theme_light(),
                                self.pending_theme == ThemeMode::Light,
                                palette,
                                cx.listener(|this, _, _, cx| {
                                    this.pending_theme = ThemeMode::Light;
                                    cx.notify();
                                }),
                            ))
                            .child(segment_option(
                                "opt-theme-dark",
                                i18n.theme_dark(),
                                self.pending_theme == ThemeMode::Dark,
                                palette,
                                cx.listener(|this, _, _, cx| {
                                    this.pending_theme = ThemeMode::Dark;
                                    cx.notify();
                                }),
                            ))
                            .child(segment_option(
                                "opt-theme-system",
                                i18n.theme_system(),
                                self.pending_theme == ThemeMode::System,
                                palette,
                                cx.listener(|this, _, _, cx| {
                                    this.pending_theme = ThemeMode::System;
                                    cx.notify();
                                }),
                            )),
                    ),
            )
            // Auto-dim setting
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_size(px(12.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(palette.text)
                            .child(i18n.dim_setting_label()),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .id("input-dim-hours")
                                    .flex_1()
                                    .px_3()
                                    .py(px(7.0))
                                    .rounded_lg()
                                    .bg(palette.subtle_bg)
                                    .border_1()
                                    .border_color(if dim_focused {
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
                                    .child(if self.pending_dim_text.is_empty() {
                                        "0".to_string()
                                    } else {
                                        self.pending_dim_text.clone()
                                    }),
                            )
                            .child(stepper_button(
                                "btn-dim-dec",
                                "−",
                                palette,
                                cx.listener(|this, _, _, cx| {
                                    let val =
                                        this.pending_dim_text.trim().parse::<f64>().unwrap_or(1.0);
                                    let next = (val - 0.5).max(0.0);
                                    this.pending_dim_text = format_hours(next);
                                    cx.notify();
                                }),
                            ))
                            .child(stepper_button(
                                "btn-dim-inc",
                                "+",
                                palette,
                                cx.listener(|this, _, _, cx| {
                                    let val =
                                        this.pending_dim_text.trim().parse::<f64>().unwrap_or(1.0);
                                    let next = (val + 0.5).min(168.0);
                                    this.pending_dim_text = format_hours(next);
                                    cx.notify();
                                }),
                            )),
                    )
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(palette.text_muted)
                            .child(i18n.dim_setting_note()),
                    ),
            )
            // Keep-awake interval setting
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_size(px(12.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(palette.text)
                            .child(i18n.keep_awake_interval_label()),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .id("input-keep-awake-interval")
                                    .flex_1()
                                    .px_3()
                                    .py(px(7.0))
                                    .rounded_lg()
                                    .bg(palette.subtle_bg)
                                    .border_1()
                                    .border_color(if interval_focused {
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
                                    .child(if self.pending_interval_text.is_empty() {
                                        "3".to_string()
                                    } else {
                                        self.pending_interval_text.clone()
                                    }),
                            )
                            .child(stepper_button(
                                "btn-interval-dec",
                                "−",
                                palette,
                                cx.listener(|this, _, _, cx| {
                                    let val = this
                                        .pending_interval_text
                                        .trim()
                                        .parse::<u64>()
                                        .unwrap_or(3);
                                    let next = val.saturating_sub(1).max(1);
                                    this.pending_interval_text = next.to_string();
                                    cx.notify();
                                }),
                            ))
                            .child(stepper_button(
                                "btn-interval-inc",
                                "+",
                                palette,
                                cx.listener(|this, _, _, cx| {
                                    let val = this
                                        .pending_interval_text
                                        .trim()
                                        .parse::<u64>()
                                        .unwrap_or(3);
                                    let next = (val + 1).min(3600);
                                    this.pending_interval_text = next.to_string();
                                    cx.notify();
                                }),
                            )),
                    )
                    .child(
                        div()
                            .text_size(px(11.0))
                            .line_height(px(15.0))
                            .text_color(palette.text_muted)
                            .child(i18n.keep_awake_interval_note()),
                    ),
            )
            // Debug mode checkbox
            .child(
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
                    .child(
                        div()
                            .w(px(18.0))
                            .h(px(18.0))
                            .rounded_sm()
                            .border_1()
                            .border_color(if self.pending_debug {
                                palette.brand
                            } else {
                                palette.border
                            })
                            .bg(if self.pending_debug {
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
                            .child(if self.pending_debug { "✓" } else { "" }),
                    )
                    .child(
                        div()
                            .text_size(px(13.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(palette.text)
                            .child(i18n.debug_mode()),
                    ),
            )
            // Action buttons
            .child(
                div()
                    .pt_3()
                    .border_t_1()
                    .border_color(palette.border)
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(primary_button(
                        "btn-settings-apply",
                        i18n.btn_apply_changes(),
                        false,
                        palette,
                        cx.listener(|this, _, _, cx| {
                            this.apply_settings(cx);
                        }),
                    ))
                    .child(secondary_button(
                        "btn-settings-cancel",
                        i18n.cancel(),
                        false,
                        palette,
                        cx.listener(|this, _, _, cx| {
                            this.active_sheet = None;
                            this.active_input = ActiveInput::None;
                            cx.notify();
                        }),
                    )),
            )
            .into_any_element()
    }

    fn render_logs_sheet(&self, _palette: ThemePalette) -> AnyElement {
        div()
            .id("logs-scroll-box")
            .w_full()
            .h(px(240.0))
            .p_4()
            .rounded_lg()
            .bg(rgb(0x111827))
            .border_1()
            .border_color(rgb(0x1f2937))
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap_1()
            .when(self.logs.is_empty(), |el| {
                el.child(
                    div()
                        .text_size(px(12.0))
                        .italic()
                        .text_color(rgb(0x6b7280))
                        .child("No logs available."),
                )
            })
            .children(self.logs.iter().map(|entry| {
                div()
                    .text_size(px(11.5))
                    .line_height(px(17.0))
                    .text_color(rgb(0xe5e7eb))
                    .child(entry.clone())
            }))
            .into_any_element()
    }

    fn render_manual_ip_sheet(&self, palette: ThemePalette, cx: &mut Context<Self>) -> AnyElement {
        let i18n = self.i18n();
        let ip_focused = self.active_input == ActiveInput::ManualIp;

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .text_size(px(12.5))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(palette.text)
                    .child(i18n.prompt_enter_ip()),
            )
            .child(
                div()
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
                    .child(if self.manual_ip_text.is_empty() {
                        "192.168.1.20".to_string()
                    } else {
                        self.manual_ip_text.clone()
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(primary_button(
                        "btn-manual-ip-connect",
                        i18n.btn_continue(),
                        self.manual_ip_text.trim().is_empty(),
                        palette,
                        cx.listener(|this, _, _, cx| {
                            let ip = this.manual_ip_text.clone();
                            this.trigger_connect_manual(ip, cx);
                        }),
                    ))
                    .child(secondary_button(
                        "btn-manual-ip-cancel",
                        i18n.cancel(),
                        false,
                        palette,
                        cx.listener(|this, _, _, cx| {
                            this.active_sheet = None;
                            this.active_input = ActiveInput::None;
                            cx.notify();
                        }),
                    )),
            )
            .into_any_element()
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

        let main_content = if show_auto_connect {
            self.render_auto_connect_view(palette, cx)
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
            .child(self.render_header(palette, cx))
            .child(main_content)
            .when_some(self.active_sheet, |root, sheet| {
                root.child(self.render_sheet_overlay(sheet, palette, cx))
            })
    }
}

fn primary_button(
    id: &'static str,
    label: &str,
    disabled: bool,
    palette: ThemePalette,
    on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
) -> AnyElement {
    let bg = if disabled {
        palette.disabled_bg
    } else {
        palette.brand
    };
    let fg = if disabled {
        palette.disabled_text
    } else {
        gpui::white()
    };

    div()
        .id(id)
        .w_full()
        .py(px(10.0))
        .px_4()
        .rounded_full()
        .bg(bg)
        .text_color(fg)
        .text_size(px(13.0))
        .font_weight(FontWeight::BOLD)
        .flex()
        .items_center()
        .justify_center()
        .when(!disabled, |b| {
            b.cursor_pointer()
                .hover(|s| s.bg(palette.brand_hover))
                .on_click(on_click)
        })
        .child(label.to_string())
        .into_any_element()
}

fn secondary_button(
    id: &'static str,
    label: &str,
    disabled: bool,
    palette: ThemePalette,
    on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
) -> AnyElement {
    div()
        .id(id)
        .w_full()
        .py(px(9.0))
        .px_4()
        .rounded_full()
        .bg(palette.subtle_bg)
        .border_1()
        .border_color(palette.border)
        .text_color(if disabled {
            palette.disabled_text
        } else {
            palette.text
        })
        .text_size(px(13.0))
        .font_weight(FontWeight::SEMIBOLD)
        .flex()
        .items_center()
        .justify_center()
        .when(!disabled, |b| {
            b.cursor_pointer()
                .hover(|s| s.border_color(palette.brand))
                .on_click(on_click)
        })
        .child(label.to_string())
        .into_any_element()
}

fn segment_option(
    id: &'static str,
    label: &str,
    selected: bool,
    palette: ThemePalette,
    on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
) -> AnyElement {
    div()
        .id(id)
        .flex_1()
        .py(px(7.0))
        .px_2()
        .rounded_lg()
        .border_1()
        .border_color(if selected {
            palette.brand
        } else {
            palette.border
        })
        .bg(if selected {
            palette.brand_tint
        } else {
            palette.subtle_bg
        })
        .text_color(if selected {
            palette.brand
        } else {
            palette.text
        })
        .text_size(px(12.0))
        .font_weight(if selected {
            FontWeight::BOLD
        } else {
            FontWeight::MEDIUM
        })
        .cursor_pointer()
        .flex()
        .items_center()
        .justify_center()
        .on_click(on_click)
        .child(label.to_string())
        .into_any_element()
}

fn stepper_button(
    id: &'static str,
    label: &'static str,
    palette: ThemePalette,
    on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
) -> AnyElement {
    div()
        .id(id)
        .w(px(34.0))
        .h(px(34.0))
        .rounded_lg()
        .bg(palette.subtle_bg)
        .border_1()
        .border_color(palette.border)
        .cursor_pointer()
        .hover(|s| s.border_color(palette.brand))
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(16.0))
        .font_weight(FontWeight::BOLD)
        .text_color(palette.text)
        .on_click(on_click)
        .child(label)
        .into_any_element()
}

fn format_elapsed(sec: u64) -> String {
    let h = sec / 3600;
    let m = (sec % 3600) / 60;
    let s = sec % 60;
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

fn format_hours(hours: f64) -> String {
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

    #[test]
    fn embedded_svg_assets_load_successfully() {
        let assets = AppAssets;
        for path in [
            "icons/usb.svg",
            "icons/wifi.svg",
            "icons/settings.svg",
            "icons/play.svg",
            "icons/moon.svg",
            "icons/error.svg",
        ] {
            let loaded = assets.load(path).unwrap();
            assert!(loaded.is_some(), "missing asset: {path}");
        }
        assert!(assets.load("icons/nonexistent.svg").unwrap().is_none());
    }
}
