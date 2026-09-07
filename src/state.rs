use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;

// Shared state for the background task
pub struct AppState {
    pub keep_awake_task: Mutex<Option<JoinHandle<()>>>,
    pub dim_task: Mutex<Option<JoinHandle<()>>>,
    pub is_running: Arc<AtomicBool>,
    pub debug_mode: Arc<AtomicBool>,
    pub adb_child: Mutex<Option<tauri_plugin_shell::process::CommandChild>>,
    pub adb_start: tokio::sync::Mutex<()>,
    pub operation: tokio::sync::Mutex<()>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            keep_awake_task: Mutex::new(None),
            dim_task: Mutex::new(None),
            is_running: Arc::new(AtomicBool::new(false)),
            debug_mode: Arc::new(AtomicBool::new(false)),
            adb_child: Mutex::new(None),
            adb_start: tokio::sync::Mutex::new(()),
            operation: tokio::sync::Mutex::new(()),
        }
    }
}
