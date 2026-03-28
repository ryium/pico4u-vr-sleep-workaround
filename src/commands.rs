use crate::config::{AppConfig, load_config, save_config};
use crate::state::AppState;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_shell::ShellExt;
use tokio::time::{Duration, interval, sleep};

// Helper to run ADB command via Sidecar
async fn run_adb_command(app: &AppHandle, args: &[String]) -> Result<String, String> {
    let command = app
        .shell()
        .sidecar("adb")
        .map_err(|e| format!("Sidecar configuration error: {}", e))?;

    let output = command
        .args(args)
        .output()
        .await
        .map_err(|e| format!("Failed to execute ADB command: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.is_empty() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(stderr.to_string())
        }
    }
}

#[tauri::command]
pub async fn connect_device(app: AppHandle, ip: Option<String>) -> Result<String, String> {
    let args = match ip.as_deref().filter(|s| !s.trim().is_empty()) {
        Some(ip_addr) => {
            let connection_str = if ip_addr.contains(':') {
                ip_addr.to_string()
            } else {
                format!("{}:5555", ip_addr)
            };
            vec!["connect".to_string(), connection_str]
        }
        None => vec!["devices".to_string()],
    };

    run_adb_command(&app, &args).await
}

#[tauri::command]
pub async fn enable_tcpip(app: AppHandle) -> Result<String, String> {
    run_adb_command(&app, &vec!["tcpip".to_string(), "5555".to_string()]).await
}

#[tauri::command]
pub async fn set_usb_mode(app: AppHandle) -> Result<String, String> {
    run_adb_command(&app, &vec!["usb".to_string()]).await
}

#[tauri::command]
pub async fn get_device_ip(app: AppHandle) -> Result<String, String> {
    let output = run_adb_command(
        &app,
        &vec![
            "shell".to_string(),
            "ip".to_string(),
            "addr".to_string(),
            "show".to_string(),
            "wlan0".to_string(),
        ],
    )
    .await?;

    // Parse output for inet address
    // Expected format: "    inet 192.168.1.10/24 ..."
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("inet ") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() > 1 {
                // parts[1] should be IP/CIDR (e.g., 192.168.1.10/24)
                if let Some(ip) = parts[1].split('/').next() {
                    return Ok(ip.to_string());
                }
            }
        }
    }

    Err("Could not find IP address for wlan0. Ensure device is connected via USB.".to_string())
}

#[tauri::command]
pub async fn get_device_model(app: AppHandle) -> Result<String, String> {
    let output = run_adb_command(
        &app,
        &vec![
            "shell".to_string(),
            "getprop".to_string(),
            "ro.product.model".to_string(),
        ],
    )
    .await?;
    Ok(output.trim().to_string())
}

#[tauri::command]
pub async fn set_debug_mode(state: State<'_, AppState>, enabled: bool) -> Result<(), String> {
    state.debug_mode.store(enabled, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub async fn start_keep_awake(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    _mode: String,
) -> Result<(), String> {
    if state.is_running.swap(true, Ordering::SeqCst) {
        return Err("Keep awake task is already running.".to_string());
    }

    let is_running = state.is_running.clone();
    let debug_mode = state.debug_mode.clone();
    let app_handle_task = app_handle.clone();
    let config = load_config(&app_handle);
    let interval_secs = config.keep_awake_interval_secs;

    // Main Keep-Awake Task
    let task = tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(interval_secs));

        while is_running.load(Ordering::Relaxed) {
            interval.tick().await;

            // Check device status first (optional but safer)
            let status_output = run_adb_command(
                &app_handle_task,
                &vec![
                    "shell".to_string(),
                    "dumpsys".to_string(),
                    "power".to_string(),
                ],
            )
            .await;

            let should_wake = match status_output {
                Ok(output) => {
                    // Look for mWakefulness=Awake
                    !output.contains("mWakefulness=Awake")
                }
                Err(_) => true, // If check fails, try to wake up anyway
            };

            if should_wake {
                let res = run_adb_command(
                    &app_handle_task,
                    &vec![
                        "shell".to_string(),
                        "input".to_string(),
                        "keyevent".to_string(),
                        "224".to_string(),
                    ],
                )
                .await;

                if debug_mode.load(Ordering::Relaxed) {
                    let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
                    match res {
                        Ok(_) => {
                            let _ = app_handle_task
                                .emit("debug-log", format!("[{}] Sent wakeup event", timestamp));
                        }
                        Err(e) => {
                            let _ = app_handle_task
                                .emit("debug-log", format!("[{}] Wakeup error: {}", timestamp, e));
                        }
                    }
                }
            } else if debug_mode.load(Ordering::Relaxed) {
                let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
                let _ = app_handle_task.emit(
                    "debug-log",
                    format!("[{}] Device is already awake, skipping", timestamp),
                );
            }
        }
    });

    // Store task handle
    if let Ok(mut lock) = state.keep_awake_task.lock() {
        *lock = Some(task);
    }

    // Auto-Dim Task
    let config = load_config(&app_handle);
    if config.dim_delay_hours > 0.0 {
        let hours = config.dim_delay_hours;
        let is_running_dim = state.is_running.clone();
        let app_handle_dim = app_handle.clone();

        let dim_task = tokio::spawn(async move {
            let seconds = (hours * 3600.0) as u64;
            if seconds > 0 {
                sleep(Duration::from_secs(seconds)).await;
            }

            // Only execute if still running
            if is_running_dim.load(Ordering::Relaxed) {
                let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
                let _ = app_handle_dim.emit(
                    "debug-log",
                    format!("[{}] Executing auto-dim...", timestamp),
                );

                let _ = run_adb_command(
                    &app_handle_dim,
                    &vec![
                        "shell".to_string(),
                        "settings".to_string(),
                        "put".to_string(),
                        "system".to_string(),
                        "screen_brightness".to_string(),
                        "1".to_string(),
                    ],
                )
                .await;
            }
        });

        if let Ok(mut lock) = state.dim_task.lock() {
            *lock = Some(dim_task);
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn stop_keep_awake(state: State<'_, AppState>) -> Result<(), String> {
    if !state.is_running.swap(false, Ordering::SeqCst) {
        return Err("Keep awake task is not running.".to_string());
    }

    if let Ok(mut lock) = state.keep_awake_task.lock() {
        if let Some(task) = lock.take() {
            task.abort();
        }
    }

    if let Ok(mut lock) = state.dim_task.lock() {
        if let Some(task) = lock.take() {
            task.abort();
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn check_connection(app: AppHandle) -> Result<String, String> {
    run_adb_command(&app, &vec!["devices".to_string()]).await
}

#[tauri::command]
pub async fn kill_adb(app: AppHandle) -> Result<String, String> {
    run_adb_command(&app, &vec!["kill-server".to_string()]).await
}

#[tauri::command]
pub async fn get_config(app: AppHandle) -> Result<AppConfig, String> {
    Ok(load_config(&app))
}

#[tauri::command]
pub async fn save_config_cmd(app: AppHandle, config: AppConfig) -> Result<(), String> {
    save_config(&app, &config)
}
