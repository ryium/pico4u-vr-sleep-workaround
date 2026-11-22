use crate::adb_client::run_adb_device_command;
use crate::config::load_config;
use crate::connection::{self, ConnectionMode};
use crate::error::{AppError, ErrorCode};
use crate::state::AppState;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tokio::time::{Duration, MissedTickBehavior, interval, sleep, timeout};

pub fn get_running(state: &AppState) -> bool {
    state.is_running.load(Ordering::SeqCst)
}

pub async fn start_keep_awake(state: &Arc<AppState>, mode: ConnectionMode) -> Result<(), AppError> {
    let _guard = state.operation.lock().await;
    if state.is_running.load(Ordering::SeqCst) {
        return Err(AppError::new(ErrorCode::AlreadyRunning));
    }
    let config = load_config(state);
    config
        .validate()
        .map_err(|e| AppError::with(ErrorCode::Unknown, e))?;
    let serial = timeout(
        Duration::from_secs(25),
        connection::require_target(state, mode, &config.ip_address),
    )
    .await
    .map_err(|_| AppError::new(ErrorCode::Timeout))??;
    state.is_running.store(true, Ordering::SeqCst);
    let debug = state.debug_mode.clone();
    let wake_state = Arc::clone(state);
    let wake_serial = serial.clone();
    let task = tokio::spawn(async move {
        let mut timer = interval(Duration::from_secs(config.keep_awake_interval_secs));
        timer.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            timer.tick().await;
            let result = async {
                if !connection::verify_pico(&wake_state, &wake_serial).await? {
                    return Err(AppError::new(ErrorCode::DeviceLost));
                }
                let power = run_adb_device_command(
                    Some(&wake_state),
                    Some(&wake_serial),
                    "shell:dumpsys power",
                )
                .await;
                if !power.is_ok_and(|output| output.contains("mWakefulness=Awake")) {
                    run_adb_device_command(
                        Some(&wake_state),
                        Some(&wake_serial),
                        "shell:input keyevent 224",
                    )
                    .await?;
                }
                Ok::<_, AppError>(())
            }
            .await;
            if debug.load(Ordering::Relaxed) {
                let message = match result {
                    Ok(()) => "Keep Alive checked".to_string(),
                    Err(e) => format!("Keep Alive: {e}"),
                };
                wake_state.push_debug_log(message);
            }
        }
    });
    *state.keep_awake_task.lock().unwrap() = Some(task);
    if config.dim_delay_hours > 0.0 {
        let dim_state = Arc::clone(state);
        let dim_task = tokio::spawn(async move {
            sleep(Duration::from_secs_f64(config.dim_delay_hours * 3600.0)).await;
            let result = async {
                if !connection::verify_pico(&dim_state, &serial).await? {
                    return Err(AppError::new(ErrorCode::DeviceLost));
                }
                run_adb_device_command(
                    Some(&dim_state),
                    Some(&serial),
                    "shell:settings put system screen_brightness 1",
                )
                .await
                .map_err(AppError::from)
            }
            .await;
            dim_state.push_debug_log(match result {
                Ok(_) => "Auto-dim applied".to_string(),
                Err(e) => format!("Auto-dim: {e}"),
            });
        });
        *state.dim_task.lock().unwrap() = Some(dim_task);
    }
    Ok(())
}

pub async fn stop_keep_awake(state: &AppState) -> Result<(), AppError> {
    let _guard = state.operation.lock().await;
    state.is_running.store(false, Ordering::SeqCst);
    let wake = state.keep_awake_task.lock().unwrap().take();
    let dim = state.dim_task.lock().unwrap().take();
    // Await cancellation before acknowledging stop, including any in-flight I/O.
    for task in [wake, dim].into_iter().flatten() {
        task.abort();
        let _ = task.await;
    }
    Ok(())
}
