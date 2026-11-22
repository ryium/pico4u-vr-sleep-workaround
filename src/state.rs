use crate::config::AppConfig;
use std::path::PathBuf;
use std::process::Child;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;

pub struct AppState {
    pub runtime: Runtime,
    pub keep_awake_task: Mutex<Option<JoinHandle<()>>>,
    pub dim_task: Mutex<Option<JoinHandle<()>>>,
    pub is_running: Arc<AtomicBool>,
    pub debug_mode: Arc<AtomicBool>,
    pub adb_child: Mutex<Option<Child>>,
    pub adb_start: tokio::sync::Mutex<()>,
    pub operation: tokio::sync::Mutex<()>,
    pub config_path: Option<PathBuf>,
    pub config: Mutex<AppConfig>,
    pub debug_logs: Mutex<Vec<String>>,
}

impl AppState {
    pub fn new(config_path: Option<PathBuf>) -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to initialize Tokio runtime");
        let initial_config = crate::config::load_config_from_path(config_path.as_deref());
        Self {
            runtime,
            keep_awake_task: Mutex::new(None),
            dim_task: Mutex::new(None),
            is_running: Arc::new(AtomicBool::new(false)),
            debug_mode: Arc::new(AtomicBool::new(false)),
            adb_child: Mutex::new(None),
            adb_start: tokio::sync::Mutex::new(()),
            operation: tokio::sync::Mutex::new(()),
            config_path,
            config: Mutex::new(initial_config),
            debug_logs: Mutex::new(Vec::new()),
        }
    }

    pub fn push_debug_log(&self, message: impl Into<String>) {
        if let Ok(mut logs) = self.debug_logs.lock() {
            logs.push(message.into());
        }
    }

    pub fn drain_debug_logs(&self) -> Vec<String> {
        if let Ok(mut logs) = self.debug_logs.lock() {
            std::mem::take(&mut *logs)
        } else {
            Vec::new()
        }
    }

    pub fn shutdown(&self) {
        self.is_running.store(false, Ordering::SeqCst);
        for slot in [&self.keep_awake_task, &self.dim_task] {
            if let Ok(mut guard) = slot.lock()
                && let Some(task) = guard.take()
            {
                task.abort();
            }
        }
        // Kill only the foreground server child created by this app.
        // Never kill a shared server, or all processes named adb.exe.
        if let Ok(mut guard) = self.adb_child.lock()
            && let Some(mut child) = guard.take()
        {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new(crate::config::default_config_path())
    }
}

impl Drop for AppState {
    fn drop(&mut self) {
        self.shutdown();
    }
}
