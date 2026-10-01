use crate::connection::ConnectionMode;
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
    pub last_connection_mode: Option<ConnectionMode>,
}

impl AppConfig {
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=3600).contains(&self.keep_awake_interval_secs)
            || !self.dim_delay_hours.is_finite()
            || !(0.0..=168.0).contains(&self.dim_delay_hours)
        {
            return Err("invalid_settings".into());
        }
        if !self.ip_address.is_empty() {
            crate::connection::endpoint(&self.ip_address)?;
        }
        Ok(())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            dim_delay_hours: 1.0,
            ip_address: String::new(),
            keep_awake_interval_secs: 3,
            last_connection_mode: None,
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
    if let Some(path) = get_config_path(app)
        && let Ok(content) = fs::read_to_string(path)
        && let Ok(config) = serde_json::from_str(&content)
    {
        return config;
    }
    AppConfig::default()
}

pub fn save_config(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    if let Some(path) = get_config_path(app) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let content = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
        let temporary = path.with_extension("json.tmp");
        fs::write(&temporary, content).map_err(|e| e.to_string())?;
        fs::rename(temporary, path).map_err(|e| e.to_string())
    } else {
        Err("Failed to resolve config directory".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn old_settings_load_with_defaults() {
        let config: AppConfig =
            serde_json::from_str(r#"{"ip_address":"192.168.1.2","last_connection_mode":"wired"}"#)
                .unwrap();
        assert_eq!(config.keep_awake_interval_secs, 3);
        assert_eq!(config.last_connection_mode, Some(ConnectionMode::Wired));
        assert!(config.validate().is_ok());
    }
    #[test]
    fn rejects_invalid_timer_values() {
        let config = AppConfig {
            keep_awake_interval_secs: 0,
            ..AppConfig::default()
        };
        assert!(config.validate().is_err());
        let config = AppConfig {
            dim_delay_hours: f64::NAN,
            ..AppConfig::default()
        };
        assert!(config.validate().is_err());
    }
}
