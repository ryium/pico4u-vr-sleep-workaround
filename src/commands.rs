use crate::adb_client::{run_adb_device_command, run_adb_host_command};
use crate::config::{AppConfig, load_config, save_config};
use crate::connection::{
    self, ConnectionMode, ConnectionStatus, UsbProbe, parse_device_rows, probe_usb,
    wireless_candidates,
};
use crate::error::{AppError, ErrorCode, classify_connect_reply};
use crate::state::AppState;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, State};
use tokio::time::{Duration, timeout};

const OPERATION_TIMEOUT: Duration = Duration::from_secs(25);

#[derive(Debug, Clone, serde::Serialize)]
pub struct SetupOutcome {
    pub ip: String,
    pub already_connected: bool,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SetupStep {
    Usb,
    Authorized,
    Pico,
    Wifi,
    Wireless,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StepState {
    Running,
    Ok,
    Failed,
    #[allow(dead_code)]
    Skipped,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SetupProgress {
    pub step: SetupStep,
    pub state: StepState,
    pub detail: Option<String>,
}

fn emit_progress(app: &AppHandle, step: SetupStep, state: StepState, detail: Option<String>) {
    let _ = app.emit(
        "setup-progress",
        SetupProgress {
            step,
            state,
            detail,
        },
    );
}

#[tauri::command]
pub async fn check_connection_status(
    app: AppHandle,
    mode: Option<ConnectionMode>,
    ip: Option<String>,
) -> Result<ConnectionStatus, AppError> {
    let config = load_config(&app);
    let mode = mode
        .or(config.last_connection_mode)
        .unwrap_or(ConnectionMode::Wired);
    timeout(
        OPERATION_TIMEOUT,
        connection::status(&app, mode, &ip.unwrap_or(config.ip_address)),
    )
    .await
    .map_err(|_| AppError::new(ErrorCode::Timeout))?
}

#[tauri::command]
pub async fn try_auto_connect(
    app: AppHandle,
    state: State<'_, AppState>,
    mode: ConnectionMode,
    ip: String,
) -> Result<ConnectionMode, AppError> {
    let _guard = state.operation.lock().await;
    if state.is_running.load(Ordering::SeqCst) {
        return Err(AppError::new(ErrorCode::AlreadyRunning));
    }
    timeout(OPERATION_TIMEOUT, async {
        if mode == ConnectionMode::Wireless {
            let target = connection::endpoint(&ip)?;
            let reply = run_adb_host_command(Some(&app), &format!("host:connect:{target}")).await?;
            classify_connect_reply(&reply)?;
        }
        connection::require_target(&app, mode, &ip).await?;
        Ok(mode)
    })
    .await
    .map_err(|_| AppError::new(ErrorCode::Timeout))?
}

#[tauri::command]
pub async fn setup_wireless(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<SetupOutcome, AppError> {
    let _guard = state.operation.lock().await;
    if state.is_running.load(Ordering::SeqCst) {
        return Err(AppError::new(ErrorCode::AlreadyRunning));
    }
    timeout(OPERATION_TIMEOUT, async {
        // Step 1: Probe USB or check if wireless is already connected
        emit_progress(&app, SetupStep::Usb, StepState::Running, None);
        let output = run_adb_host_command(Some(&app), "host:devices-l").await?;
        let rows = parse_device_rows(&output);
        let config = load_config(&app);

        let usb_serial = match probe_usb(&rows) {
            UsbProbe::Ready(mut ser) => {
                let s = ser.remove(0);
                emit_progress(&app, SetupStep::Usb, StepState::Ok, None);
                emit_progress(&app, SetupStep::Authorized, StepState::Ok, None);
                s
            }
            UsbProbe::Unauthorized => {
                emit_progress(&app, SetupStep::Usb, StepState::Ok, None);
                emit_progress(&app, SetupStep::Authorized, StepState::Failed, None);
                return Err(AppError::new(ErrorCode::UsbUnauthorized));
            }
            UsbProbe::Offline => {
                emit_progress(&app, SetupStep::Usb, StepState::Failed, None);
                return Err(AppError::new(ErrorCode::UsbOffline));
            }
            UsbProbe::Missing => {
                // If USB device is missing, check if wireless is ALREADY connected!
                // Prioritize saved IP
                let saved = if config.ip_address.is_empty() {
                    None
                } else {
                    Some(config.ip_address.as_str())
                };
                let candidates = wireless_candidates(&rows, saved);
                for target in candidates {
                    if connection::verify_pico(&app, &target)
                        .await
                        .unwrap_or(false)
                    {
                        // Found active wireless PICO device!
                        let ip = target.split(':').next().unwrap_or(&target).to_string();
                        emit_progress(
                            &app,
                            SetupStep::Usb,
                            StepState::Ok,
                            Some("already_connected".into()),
                        );
                        emit_progress(&app, SetupStep::Authorized, StepState::Ok, None);
                        emit_progress(&app, SetupStep::Pico, StepState::Ok, None);
                        emit_progress(&app, SetupStep::Wifi, StepState::Ok, None);
                        emit_progress(&app, SetupStep::Wireless, StepState::Ok, None);
                        return Ok(SetupOutcome {
                            ip,
                            already_connected: true,
                        });
                    }
                }

                emit_progress(&app, SetupStep::Usb, StepState::Failed, None);
                return Err(AppError::new(ErrorCode::UsbNotFound));
            }
        };

        // Step 2: Verify PICO 4 Ultra
        emit_progress(&app, SetupStep::Pico, StepState::Running, None);
        if !connection::verify_pico(&app, &usb_serial).await? {
            emit_progress(&app, SetupStep::Pico, StepState::Failed, None);
            return Err(AppError::new(ErrorCode::NotPico));
        }
        emit_progress(&app, SetupStep::Pico, StepState::Ok, None);

        // Step 3: Wi-Fi state & IP
        emit_progress(&app, SetupStep::Wifi, StepState::Running, None);
        if let Some(false) = connection::wifi_enabled(&app, &usb_serial).await {
            emit_progress(
                &app,
                SetupStep::Wifi,
                StepState::Failed,
                Some("wifi_disabled".into()),
            );
            return Err(AppError::new(ErrorCode::WifiDisabled));
        }

        let ip = connection::wifi_ip(&app, &usb_serial)
            .await?
            .ok_or_else(|| {
                emit_progress(
                    &app,
                    SetupStep::Wifi,
                    StepState::Failed,
                    Some("wifi_not_connected".into()),
                );
                AppError::new(ErrorCode::WifiNotConnected)
            })?;
        emit_progress(&app, SetupStep::Wifi, StepState::Ok, Some(ip.clone()));

        // Step 4: Enable TCP/IP on headset & connect
        emit_progress(&app, SetupStep::Wireless, StepState::Running, None);
        if let Err(e) = run_adb_device_command(Some(&app), Some(&usb_serial), "tcpip:5555").await {
            emit_progress(
                &app,
                SetupStep::Wireless,
                StepState::Failed,
                Some("tcpip_failed".into()),
            );
            return Err(AppError::with(ErrorCode::TcpipFailed, e));
        }

        let target = connection::endpoint(&ip)?;
        let mut last_error = AppError::new(ErrorCode::WirelessNotFound);

        for _attempt in 0..3 {
            tokio::time::sleep(Duration::from_secs(2)).await;
            if let Ok(reply) =
                run_adb_host_command(Some(&app), &format!("host:connect:{target}")).await
                && let Err(e) = classify_connect_reply(&reply)
            {
                last_error = e;
            }
            if connection::target_device(&app, ConnectionMode::Wireless, &ip)
                .await?
                .is_some()
            {
                emit_progress(&app, SetupStep::Wireless, StepState::Ok, None);
                return Ok(SetupOutcome {
                    ip,
                    already_connected: false,
                });
            }
        }

        emit_progress(&app, SetupStep::Wireless, StepState::Failed, None);
        Err(last_error)
    })
    .await
    .map_err(|_| AppError::new(ErrorCode::Timeout))?
}

#[tauri::command]
pub async fn set_debug_mode(state: State<'_, AppState>, enabled: bool) -> Result<(), AppError> {
    state.debug_mode.store(enabled, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub async fn get_config(app: AppHandle) -> Result<AppConfig, AppError> {
    Ok(load_config(&app))
}

#[tauri::command]
pub async fn save_config_cmd(
    app: AppHandle,
    state: State<'_, AppState>,
    config: AppConfig,
) -> Result<(), AppError> {
    let _guard = state.operation.lock().await;
    config
        .validate()
        .map_err(|e| AppError::with(ErrorCode::Unknown, e))?;
    if state.is_running.load(Ordering::SeqCst) {
        let current = load_config(&app);
        if config.last_connection_mode != current.last_connection_mode
            || config.ip_address != current.ip_address
        {
            return Err(AppError::new(ErrorCode::AlreadyRunning));
        }
    }
    save_config(&app, &config).map_err(|e| AppError::with(ErrorCode::Unknown, e))
}
