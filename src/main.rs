#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod adb_client;
mod commands;
mod config;
mod connection;
mod error;
mod runtime;
mod state;
mod steamvr;

use crate::commands::*;
use crate::runtime::*;
use crate::state::AppState;
use std::sync::atomic::Ordering;
use tauri::Manager;

fn main() {
    let launched_by_steamvr = crate::steamvr::is_steamvr_launch_arg(std::env::args());

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::default())
        .setup(move |app| {
            let state = app.state::<AppState>();
            state
                .launched_by_steamvr
                .store(launched_by_steamvr, Ordering::SeqCst);

            if let Some(window) = app.get_webview_window("main") {
                let suffix = if cfg!(debug_assertions) { "-dev" } else { "" };
                let title = format!("pico4u V睡ツール v{}{}", env!("CARGO_PKG_VERSION"), suffix);
                let _ = window.set_title(&title);
                if launched_by_steamvr {
                    let _ = window.minimize();
                }
            }

            let _ = crate::steamvr::ensure_registered(app.handle());
            crate::steamvr::spawn_steamvr_exit_monitor(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            check_connection_status,
            try_auto_connect,
            setup_wireless,
            start_keep_awake,
            stop_keep_awake,
            get_running,
            set_debug_mode,
            get_config,
            save_config_cmd
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                let state = app.state::<AppState>();
                state.cleanup();
            }
        });
}
