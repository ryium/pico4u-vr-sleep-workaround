//! Asynchronous actions and background loops for `PicoSleepApp`.

use crate::commands::{self, StepState, initial_setup_steps, update_setup_steps};
use crate::connection::ConnectionMode;
use crate::error::AppError;
use crate::runtime;
use crate::ui::{
    ActiveInput, ActiveSheet, AutoConnectStatus, PicoSleepApp, Screen, SetupRunStatus, format_hours,
};
use gpui::Context;
use std::sync::Arc;
use std::time::{Duration, Instant};

impl PicoSleepApp {
    pub(crate) fn start_background_loops(&self, cx: &mut Context<Self>) {
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
                                    if status.is_connected(this.config.last_connection_mode) {
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

    pub(crate) fn trigger_auto_connect(&mut self, cx: &mut Context<Self>) {
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
                let target_label = if mode == ConnectionMode::Wireless && !ip.is_empty() {
                    ip.clone()
                } else {
                    "USB".to_string()
                };
                if succeeded {
                    this.auto_connect_status = AutoConnectStatus::Success;
                    this.connection_error = None;
                    this.target_mode = mode;
                    this.screen = Screen::Operate;
                    let msg = this.i18n().log_auto_connect_success(&target_label);
                    this.add_log(msg);
                } else {
                    this.auto_connect_status = AutoConnectStatus::Failed;
                    if let Some(err) = last_err {
                        this.log_error(&err);
                        this.connection_error = Some(err);
                    }
                    let msg = this.i18n().log_auto_connect_failed(&target_label);
                    this.add_log(msg);
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(crate) fn apply_setup_progress(&mut self, progress: commands::SetupProgress) {
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

    pub(crate) fn trigger_check_devices(&mut self, cx: &mut Context<Self>) {
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
                        ip_clone,
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

    pub(crate) fn trigger_setup_wireless(&mut self, cx: &mut Context<Self>) {
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

    pub(crate) fn trigger_connect_manual(&mut self, ip: String, cx: &mut Context<Self>) {
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

    pub(crate) fn trigger_toggle_keep_awake(&mut self, cx: &mut Context<Self>) {
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

    pub(crate) fn open_settings_sheet(&mut self, cx: &mut Context<Self>) {
        self.pending_language = self.language;
        self.pending_theme = self.theme_mode;
        self.pending_dim_text = format_hours(self.config.dim_delay_hours);
        self.pending_interval_text = self.config.keep_awake_interval_secs.to_string();
        self.pending_debug = self.is_debug;
        self.active_input = ActiveInput::None;
        self.active_sheet = Some(ActiveSheet::Settings);
        cx.notify();
    }

    pub(crate) fn apply_settings(&mut self, cx: &mut Context<Self>) {
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
}
