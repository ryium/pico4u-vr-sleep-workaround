#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod adb_client;
mod commands;
mod config;
mod state;

use crate::commands::*;
use crate::state::AppState;
use std::sync::atomic::Ordering;
use tauri::Manager;
use tauri_plugin_shell::ShellExt;

fn main() {
    let state = AppState::default();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(state)
        .setup(|_app| Ok(()))
        .invoke_handler(tauri::generate_handler![
            connect_device,
            enable_tcpip,
            get_device_ip,
            start_keep_awake,
            stop_keep_awake,
            check_connection,
            try_auto_connect,
            kill_adb,
            set_debug_mode,
            set_usb_mode,
            disconnect_all_wireless,
            get_device_model,
            get_config,
            save_config_cmd
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| match event {
            tauri::RunEvent::Exit => {
                let state = app_handle.state::<AppState>();
                if state.adb_started_by_us.load(Ordering::SeqCst) {
                    if let Ok(sidecar) = app_handle.shell().sidecar("adb") {
                        let _ = tauri::async_runtime::block_on(async {
                            let _ = sidecar.args(["kill-server"]).output().await;
                        });
                    }
                }
            }
            _ => {}
        });
}
