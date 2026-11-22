#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod adb_client;
mod commands;
mod config;
mod connection;
mod error;
mod i18n;
mod runtime;
mod state;
mod ui;

use crate::state::AppState;
use crate::ui::{AppAssets, PicoSleepApp, window_options};
use gpui::{App, AppContext as _, Application};
use std::sync::Arc;

fn main() {
    let state = Arc::new(AppState::default());

    Application::new()
        .with_assets(AppAssets)
        .run(move |cx: &mut App| {
            let state_for_quit = Arc::clone(&state);
            cx.on_window_closed(move |cx| {
                if cx.windows().is_empty() {
                    state_for_quit.shutdown();
                    cx.quit();
                }
            })
            .detach();

            let state_for_window = Arc::clone(&state);
            let options = window_options(cx);
            cx.open_window(options, move |window, cx| {
                cx.new(|cx| PicoSleepApp::new(state_for_window, window, cx))
            })
            .expect("error while opening GPUI application window");

            cx.activate(true);
        });
}
