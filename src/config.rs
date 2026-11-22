use crate::connection::ConnectionMode;
use crate::i18n::Language;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    Light,
    Dark,
    #[default]
    System,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(default)]
pub struct AppConfig {
    pub dim_delay_hours: f64,
    pub ip_address: String,
    pub keep_awake_interval_secs: u64,
    pub last_connection_mode: Option<ConnectionMode>,
    pub language: Option<Language>,
    pub theme: ThemeMode,
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
            language: None,
            theme: ThemeMode::System,
        }
    }
}

pub fn default_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|p| {
        p.join("com.tauri.pico4uvrsleepworkaround")
            .join("pico4u_config.json")
    })
}

pub fn load_config_from_path(path: Option<&Path>) -> AppConfig {
    if let Some(path) = path
        && let Ok(content) = fs::read_to_string(path)
        && let Ok(config) = serde_json::from_str(&content)
    {
        return config;
    }
    AppConfig::default()
}

pub fn save_config_to_path(path: Option<&Path>, config: &AppConfig) -> Result<(), String> {
    if let Some(path) = path {
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

pub fn load_config(state: &AppState) -> AppConfig {
    if let Some(ref path) = state.config_path
        && path.exists()
    {
        let loaded = load_config_from_path(Some(path));
        if let Ok(mut guard) = state.config.lock() {
            *guard = loaded.clone();
        }
        return loaded;
    }
    state
        .config
        .lock()
        .map(|c| c.clone())
        .unwrap_or_else(|_| AppConfig::default())
}

pub fn save_config(state: &AppState, config: &AppConfig) -> Result<(), String> {
    if let Some(ref path) = state.config_path {
        save_config_to_path(Some(path), config)?;
    }
    if let Ok(mut guard) = state.config.lock() {
        *guard = config.clone();
    }
    Ok(())
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
        assert_eq!(config.language, None);
        assert_eq!(config.theme, ThemeMode::System);
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

    #[test]
    fn preserves_saved_ip_and_theme_across_roundtrip() {
        let config = AppConfig {
            dim_delay_hours: 2.5,
            ip_address: "192.168.1.20".into(),
            keep_awake_interval_secs: 5,
            last_connection_mode: Some(ConnectionMode::Wireless),
            language: Some(Language::Ja),
            theme: ThemeMode::Dark,
        };
        let serialized = serde_json::to_string(&config).unwrap();
        let loaded: AppConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(loaded, config);
    }
}
