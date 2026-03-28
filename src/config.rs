use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct AppConfig {
    pub dim_delay_hours: f64,
    pub ip_address: String,
    pub keep_awake_interval_secs: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            dim_delay_hours: 1.0,
            ip_address: String::new(),
            keep_awake_interval_secs: 3,
        }
    }
}

fn get_config_path(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|p| p.join("pico4u_config.json"))
}

pub fn load_config(app: &AppHandle) -> AppConfig {
    if let Some(path) = get_config_path(app) {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(config) = serde_json::from_str(&content) {
                    return config;
                }
            }
        }
    }
    AppConfig::default()
}

pub fn save_config(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    if let Some(path) = get_config_path(app) {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let content = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
        fs::write(path, content).map_err(|e| e.to_string())
    } else {
        Err("Failed to resolve config directory".to_string())
    }
}
