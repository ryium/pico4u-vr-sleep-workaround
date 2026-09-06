use crate::adb_client::{run_adb_device_command, run_adb_host_command};
use crate::config::{AppConfig, load_config, save_config};
use crate::connection::{self, ConnectionMode, ConnectionStatus};
use crate::state::AppState;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, State};
use tokio::time::{Duration, timeout};

const OPERATION_TIMEOUT: Duration = Duration::from_secs(25);

#[tauri::command]
pub async fn check_connection_status(
    app: AppHandle,
    mode: Option<ConnectionMode>,
    ip: Option<String>,
) -> Result<ConnectionStatus, String> {
    let config = load_config(&app);
    let mode = mode
        .or(config.last_connection_mode)
        .unwrap_or(ConnectionMode::Wired);
    timeout(
        OPERATION_TIMEOUT,
        connection::status(&app, mode, &ip.unwrap_or(config.ip_address)),
    )
    .await
    .map_err(|_| "connection_timeout".to_string())?
}

#[tauri::command]
pub async fn try_auto_connect(
    app: AppHandle,
    state: State<'_, AppState>,
    mode: ConnectionMode,
    ip: String,
) -> Result<ConnectionMode, String> {
    let _guard = state.operation.lock().await;
    if state.is_running.load(Ordering::SeqCst) {
        return Err("already_running".into());
    }
    timeout(OPERATION_TIMEOUT, async {
        if mode == ConnectionMode::Wireless {
            let target = connection::endpoint(&ip)?;
            run_adb_host_command(Some(&app), &format!("host:connect:{target}")).await?;
        }
        connection::require_target(&app, mode, &ip).await?;
        Ok(mode)
    })
    .await
    .map_err(|_| "connection_timeout".to_string())?
}

#[tauri::command]
pub async fn setup_wireless(app: AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    let _guard = state.operation.lock().await;
    if state.is_running.load(Ordering::SeqCst) {
        return Err("already_running".into());
    }
    timeout(OPERATION_TIMEOUT, async {
        let serial = connection::require_target(&app, ConnectionMode::Wired, "").await?;
        // Read the address before restarting adbd, while USB is still available.
        let ip = connection::wifi_ip(&app, &serial)
            .await?
            .ok_or("wifi_unavailable")?;
        run_adb_device_command(Some(&app), Some(&serial), "tcpip:5555").await?;
        let target = connection::endpoint(&ip)?;
        for attempt in 0..3 {
            tokio::time::sleep(Duration::from_secs(2)).await;
            let _ = run_adb_host_command(Some(&app), &format!("host:connect:{target}")).await;
            if connection::target_device(&app, ConnectionMode::Wireless, &ip)
                .await?
                .is_some()
            {
                return Ok(ip);
            }
            if attempt == 2 {
                return Err("pico_not_connected".into());
            }
        }
        Err("pico_not_connected".into())
    })
    .await
    .map_err(|_| "connection_timeout".to_string())?
}

#[tauri::command]
pub async fn set_debug_mode(state: State<'_, AppState>, enabled: bool) -> Result<(), String> {
    state.debug_mode.store(enabled, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub async fn get_config(app: AppHandle) -> Result<AppConfig, String> {
    Ok(load_config(&app))
}

#[tauri::command]
pub async fn save_config_cmd(
    app: AppHandle,
    state: State<'_, AppState>,
    config: AppConfig,
) -> Result<(), String> {
    let _guard = state.operation.lock().await;
    config.validate()?;
    if state.is_running.load(Ordering::SeqCst) {
        let current = load_config(&app);
        if config.last_connection_mode != current.last_connection_mode
            || config.ip_address != current.ip_address
        {
            return Err("already_running".into());
        }
    }
    save_config(&app, &config)
}
