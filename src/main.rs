#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
mod config;
mod state;

use crate::commands::*;
use crate::state::AppState;
use tauri_plugin_shell::ShellExt;

fn main() {
    let state = AppState::default();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .manage(state)
        .setup(|_app| Ok(()))
        .invoke_handler(tauri::generate_handler![
            connect_device,
            enable_tcpip,
            get_device_ip,
            start_keep_awake,
            stop_keep_awake,
            check_connection,
            kill_adb,
            set_debug_mode,
            set_usb_mode,
            get_device_model,
            get_config,
            save_config_cmd
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| match event {
            tauri::RunEvent::Exit => {
                if let Ok(sidecar) = app_handle.shell().sidecar("adb") {
                    let _ = tauri::async_runtime::block_on(async {
                        let _ = sidecar.args(["kill-server"]).output().await;
                    });
                }
            }
            _ => {}
        });
}
