use crate::adb_client::{run_adb_device_command, run_adb_host_command};
use crate::config::{AppConfig, load_config, save_config};
use crate::connection::{
    self, ConnectionMode, ConnectionStatus, UsbProbe, parse_device_rows, probe_usb,
    wireless_candidates,
};
use crate::error::{AppError, ErrorCode, classify_connect_reply};
use crate::state::AppState;
use std::sync::atomic::Ordering;
use tokio::time::{Duration, timeout};

const OPERATION_TIMEOUT: Duration = Duration::from_secs(25);

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SetupOutcome {
    pub ip: String,
    pub already_connected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SetupStep {
    Usb,
    Authorized,
    Pico,
    Wifi,
    Wireless,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StepState {
    Idle,
    Running,
    Ok,
    Failed,
    #[allow(dead_code)]
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SetupProgress {
    pub step: SetupStep,
    pub state: StepState,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupStepItem {
    pub id: SetupStep,
    pub state: StepState,
    pub detail: Option<String>,
}

pub fn initial_setup_steps() -> Vec<SetupStepItem> {
    vec![
        SetupStepItem {
            id: SetupStep::Usb,
            state: StepState::Idle,
            detail: None,
        },
        SetupStepItem {
            id: SetupStep::Authorized,
            state: StepState::Idle,
            detail: None,
        },
        SetupStepItem {
            id: SetupStep::Pico,
            state: StepState::Idle,
            detail: None,
        },
        SetupStepItem {
            id: SetupStep::Wifi,
            state: StepState::Idle,
            detail: None,
        },
        SetupStepItem {
            id: SetupStep::Wireless,
            state: StepState::Idle,
            detail: None,
        },
    ]
}

pub fn update_setup_steps(steps: &[SetupStepItem], progress: &SetupProgress) -> Vec<SetupStepItem> {
    steps
        .iter()
        .map(|item| {
            if item.id == progress.step {
                SetupStepItem {
                    id: item.id,
                    state: progress.state,
                    detail: progress.detail.clone(),
                }
            } else {
                item.clone()
            }
        })
        .collect()
}

pub async fn check_connection_status(
    state: &AppState,
    mode: Option<ConnectionMode>,
    ip: Option<String>,
) -> Result<ConnectionStatus, AppError> {
    let config = load_config(state);
    let mode = mode
        .or(config.last_connection_mode)
        .unwrap_or(ConnectionMode::Wired);
    timeout(
        OPERATION_TIMEOUT,
        connection::status(state, mode, &ip.unwrap_or(config.ip_address)),
    )
    .await
    .map_err(|_| AppError::new(ErrorCode::Timeout))?
}

pub async fn try_auto_connect(
    state: &AppState,
    mode: ConnectionMode,
    ip: String,
    mut on_progress: impl FnMut(SetupProgress),
) -> Result<ConnectionMode, AppError> {
    let _guard = state.operation.lock().await;
    if state.is_running.load(Ordering::SeqCst) {
        return Err(AppError::new(ErrorCode::AlreadyRunning));
    }
    timeout(OPERATION_TIMEOUT, async {
        let mut emit = |step: SetupStep, step_state: StepState, detail: Option<String>| {
            on_progress(SetupProgress {
                step,
                state: step_state,
                detail,
            });
        };

        if mode == ConnectionMode::Wireless {
            let target = connection::endpoint(&ip)?;
            let reply =
                run_adb_host_command(Some(state), &format!("host:connect:{target}")).await?;
            classify_connect_reply(&reply)?;
            connection::require_target(state, mode, &ip).await?;
        } else {
            emit(SetupStep::Usb, StepState::Running, None);
            match connection::require_target(state, mode, &ip).await {
                Ok(_) => {
                    emit(SetupStep::Usb, StepState::Ok, None);
                    emit(SetupStep::Authorized, StepState::Ok, None);
                    emit(SetupStep::Pico, StepState::Ok, None);
                }
                Err(err) => {
                    match err.code {
                        ErrorCode::UsbUnauthorized => {
                            emit(SetupStep::Usb, StepState::Ok, None);
                            emit(SetupStep::Authorized, StepState::Failed, None);
                        }
                        ErrorCode::NotPico | ErrorCode::MultiplePico => {
                            emit(SetupStep::Usb, StepState::Ok, None);
                            emit(SetupStep::Authorized, StepState::Ok, None);
                            emit(SetupStep::Pico, StepState::Failed, None);
                        }
                        _ => {
                            emit(SetupStep::Usb, StepState::Failed, None);
                        }
                    }
                    return Err(err);
                }
            }
        }
        Ok(mode)
    })
    .await
    .map_err(|_| AppError::new(ErrorCode::Timeout))?
}

pub async fn setup_wireless(
    state: &AppState,
    mut on_progress: impl FnMut(SetupProgress),
) -> Result<SetupOutcome, AppError> {
    let _guard = state.operation.lock().await;
    if state.is_running.load(Ordering::SeqCst) {
        return Err(AppError::new(ErrorCode::AlreadyRunning));
    }
    timeout(OPERATION_TIMEOUT, async {
        let mut emit = |step: SetupStep, step_state: StepState, detail: Option<String>| {
            on_progress(SetupProgress {
                step,
                state: step_state,
                detail,
            });
        };

        // Step 1: Probe USB or check if wireless is already connected
        emit(SetupStep::Usb, StepState::Running, None);
        let output = run_adb_host_command(Some(state), "host:devices-l").await?;
        let rows = parse_device_rows(&output);
        let config = load_config(state);

        let usb_serial = match probe_usb(&rows) {
            UsbProbe::Ready(mut ser) => {
                let s = ser.remove(0);
                emit(SetupStep::Usb, StepState::Ok, None);
                emit(SetupStep::Authorized, StepState::Ok, None);
                s
            }
            UsbProbe::Unauthorized => {
                emit(SetupStep::Usb, StepState::Ok, None);
                emit(SetupStep::Authorized, StepState::Failed, None);
                return Err(AppError::new(ErrorCode::UsbUnauthorized));
            }
            UsbProbe::Offline => {
                emit(SetupStep::Usb, StepState::Failed, None);
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
                    if connection::verify_pico(state, &target)
                        .await
                        .unwrap_or(false)
                    {
                        // Found active wireless PICO device!
                        let ip = target.split(':').next().unwrap_or(&target).to_string();
                        emit(
                            SetupStep::Usb,
                            StepState::Ok,
                            Some("already_connected".into()),
                        );
                        emit(SetupStep::Authorized, StepState::Ok, None);
                        emit(SetupStep::Pico, StepState::Ok, None);
                        emit(SetupStep::Wifi, StepState::Ok, None);
                        emit(SetupStep::Wireless, StepState::Ok, None);
                        return Ok(SetupOutcome {
                            ip,
                            already_connected: true,
                        });
                    }
                }

                emit(SetupStep::Usb, StepState::Failed, None);
                return Err(AppError::new(ErrorCode::UsbNotFound));
            }
        };

        // Step 2: Verify PICO 4 Ultra
        emit(SetupStep::Pico, StepState::Running, None);
        if !connection::verify_pico(state, &usb_serial).await? {
            emit(SetupStep::Pico, StepState::Failed, None);
            return Err(AppError::new(ErrorCode::NotPico));
        }
        emit(SetupStep::Pico, StepState::Ok, None);

        // Step 3: Wi-Fi state & IP
        emit(SetupStep::Wifi, StepState::Running, None);
        if let Some(false) = connection::wifi_enabled(state, &usb_serial).await {
            emit(
                SetupStep::Wifi,
                StepState::Failed,
                Some("wifi_disabled".into()),
            );
            return Err(AppError::new(ErrorCode::WifiDisabled));
        }

        let ip = match connection::wifi_ip(state, &usb_serial).await? {
            Some(ip) => ip,
            None => {
                emit(
                    SetupStep::Wifi,
                    StepState::Failed,
                    Some("wifi_not_connected".into()),
                );
                return Err(AppError::new(ErrorCode::WifiNotConnected));
            }
        };
        emit(SetupStep::Wifi, StepState::Ok, Some(ip.clone()));

        // Step 4: Enable TCP/IP on headset & connect
        emit(SetupStep::Wireless, StepState::Running, None);
        if let Err(e) = run_adb_device_command(Some(state), Some(&usb_serial), "tcpip:5555").await {
            emit(
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
                run_adb_host_command(Some(state), &format!("host:connect:{target}")).await
                && let Err(e) = classify_connect_reply(&reply)
            {
                last_error = e;
            }
            if connection::target_device(state, ConnectionMode::Wireless, &ip)
                .await?
                .is_some()
            {
                emit(SetupStep::Wireless, StepState::Ok, None);
                return Ok(SetupOutcome {
                    ip,
                    already_connected: false,
                });
            }
        }

        emit(SetupStep::Wireless, StepState::Failed, None);
        Err(last_error)
    })
    .await
    .map_err(|_| AppError::new(ErrorCode::Timeout))?
}

pub fn set_debug_mode(state: &AppState, enabled: bool) {
    state.debug_mode.store(enabled, Ordering::SeqCst);
}

pub fn get_config(state: &AppState) -> AppConfig {
    load_config(state)
}

pub async fn save_config_cmd(state: &AppState, config: AppConfig) -> Result<(), AppError> {
    let _guard = state.operation.lock().await;
    config
        .validate()
        .map_err(|e| AppError::with(ErrorCode::Unknown, e))?;
    if state.is_running.load(Ordering::SeqCst) {
        let current = load_config(state);
        if config.last_connection_mode != current.last_connection_mode
            || config.ip_address != current.ip_address
        {
            return Err(AppError::new(ErrorCode::AlreadyRunning));
        }
    }
    save_config(state, &config).map_err(|e| AppError::with(ErrorCode::Unknown, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_with_initial_steps_in_idle_state() {
        let steps = initial_setup_steps();
        assert_eq!(steps.len(), 5);
        assert!(steps.iter().all(|s| s.state == StepState::Idle));
    }

    #[test]
    fn updates_target_step_correctly() {
        let steps = initial_setup_steps();
        let progress = SetupProgress {
            step: SetupStep::Usb,
            state: StepState::Ok,
            detail: None,
        };
        let updated = update_setup_steps(&steps, &progress);
        let usb = updated.iter().find(|s| s.id == SetupStep::Usb).unwrap();
        assert_eq!(usb.state, StepState::Ok);
        let wifi = updated.iter().find(|s| s.id == SetupStep::Wifi).unwrap();
        assert_eq!(wifi.state, StepState::Idle);
    }

    #[test]
    fn records_failed_state_and_detail() {
        let steps = initial_setup_steps();
        let progress = SetupProgress {
            step: SetupStep::Wifi,
            state: StepState::Failed,
            detail: Some("wifi_disabled".into()),
        };
        let updated = update_setup_steps(&steps, &progress);
        let wifi = updated.iter().find(|s| s.id == SetupStep::Wifi).unwrap();
        assert_eq!(wifi.state, StepState::Failed);
        assert_eq!(wifi.detail.as_deref(), Some("wifi_disabled"));
    }
}
