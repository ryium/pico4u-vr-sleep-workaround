#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod adb_client;
mod commands;
mod config;
mod connection;
mod runtime;
mod state;

use crate::commands::*;
use crate::runtime::*;
use crate::state::AppState;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::default())
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
                for slot in [&state.keep_awake_task, &state.dim_task] {
                    if let Some(task) = slot.lock().unwrap().take() {
                        task.abort();
                    }
                }
                // Kill only the foreground server child created by this app.
                // Never kill a shared server, or all processes named adb.exe.
                if let Some(child) = state.adb_child.lock().unwrap().take() {
                    let _ = child.kill();
                }
            }
        });
}
