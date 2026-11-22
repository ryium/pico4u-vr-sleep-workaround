use crate::config::load_config;
use crate::state::AppState;
use serde::Deserialize;
use serde_json::{Map, Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::time::Duration;
use tauri::{AppHandle, Manager};

pub const STEAMVR_APP_KEY: &str = "com.tauri.pico4uvrsleepworkaround";
pub const STEAMVR_LAUNCH_ARG: &str = "--steamvr-launch";
const MANIFEST_FILENAME: &str = "pico4u-vr-sleep-workaround.vrmanifest";

#[derive(Debug, Deserialize)]
struct OpenVrPaths {
    #[serde(default)]
    config: Vec<String>,
}

pub fn is_steamvr_launch_arg<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    args.into_iter()
        .any(|arg| arg.as_ref() == STEAMVR_LAUNCH_ARG)
}

pub fn clean_path_str(raw: &str) -> &str {
    raw.strip_prefix(r"\\?\").unwrap_or(raw)
}

pub fn clean_windows_path(path: &Path) -> String {
    clean_path_str(&path.to_string_lossy()).to_string()
}

fn normalize_path_for_compare(raw: &str) -> String {
    clean_path_str(raw)
        .replace('/', "\\")
        .to_ascii_lowercase()
}

pub fn build_vrmanifest(exe_path: &Path) -> Value {
    let binary_path = clean_windows_path(exe_path);
    json!({
        "source": "builtin",
        "applications": [
            {
                "app_key": STEAMVR_APP_KEY,
                "launch_type": "binary",
                "binary_path_windows": binary_path,
                "arguments": STEAMVR_LAUNCH_ARG,
                "is_dashboard_overlay": true,
                "strings": {
                    "en_us": {
                        "name": "pico4u VR Sleep Workaround",
                        "description": "Prevent PICO 4 Ultra from entering sleep mode during VR sleep"
                    },
                    "ja_jp": {
                        "name": "pico4u V睡ツール",
                        "description": "V睡中にPICO 4 Ultraがスリープしてトラッキングがリセットされるのを防ぎます"
                    }
                }
            }
        ]
    })
}

pub fn parse_openvr_config_dirs(content: &str) -> Vec<PathBuf> {
    let Ok(parsed) = serde_json::from_str::<OpenVrPaths>(content) else {
        return Vec::new();
    };
    parsed
        .config
        .into_iter()
        .map(|p| PathBuf::from(clean_path_str(&p)))
        .collect()
}

fn openvrpaths_candidates() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        paths.push(
            PathBuf::from(local_app_data)
                .join("openvr")
                .join("openvrpaths.vrpath"),
        );
    }
    if let Some(xdg_config) = std::env::var_os("XDG_CONFIG_HOME") {
        paths.push(
            PathBuf::from(xdg_config)
                .join("openvr")
                .join("openvrpaths.vrpath"),
        );
    }
    if let Some(home) = std::env::var_os("HOME") {
        let home_dir = PathBuf::from(home);
        paths.push(
            home_dir
                .join(".config")
                .join("openvr")
                .join("openvrpaths.vrpath"),
        );
        paths.push(
            home_dir
                .join("Library")
                .join("Application Support")
                .join("OpenVR")
                .join(".openvr")
                .join("openvrpaths.vrpath"),
        );
    }
    paths
}

pub fn find_steamvr_config_dir() -> Option<PathBuf> {
    for vrpath in openvrpaths_candidates() {
        if let Ok(content) = fs::read_to_string(&vrpath) {
            for dir in parse_openvr_config_dirs(&content) {
                if dir.is_dir() {
                    return Some(dir);
                }
            }
        }
    }

    if cfg!(target_os = "windows") {
        let mut fallbacks = Vec::new();
        if let Some(pf86) = std::env::var_os("PROGRAMFILES(X86)") {
            fallbacks.push(PathBuf::from(pf86).join("Steam").join("config"));
        }
        fallbacks.push(PathBuf::from(r"C:\Program Files (x86)\Steam\config"));
        for dir in fallbacks {
            if dir.is_dir() {
                return Some(dir);
            }
        }
    }

    None
}

fn write_json_atomic(path: &Path, value: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let content = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    let temp = path.with_extension("tmp");
    fs::write(&temp, &content).map_err(|e| e.to_string())?;
    if fs::rename(&temp, path).is_err() {
        let _ = fs::remove_file(&temp);
        fs::write(path, content).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn write_manifest_file(manifest_path: &Path, exe_path: &Path) -> Result<(), String> {
    let manifest = build_vrmanifest(exe_path);
    write_json_atomic(manifest_path, &manifest)
}

pub fn register_manifest_in_appconfig(
    appconfig_path: &Path,
    manifest_path: &Path,
) -> Result<(), String> {
    let mut root = if appconfig_path.exists() {
        let raw = fs::read_to_string(appconfig_path).map_err(|e| e.to_string())?;
        if raw.trim().is_empty() {
            Value::Object(Map::new())
        } else {
            serde_json::from_str::<Value>(&raw).map_err(|e| e.to_string())?
        }
    } else {
        Value::Object(Map::new())
    };

    let obj = root
        .as_object_mut()
        .ok_or_else(|| "appconfig.json root must be an object".to_string())?;

    let manifest_str = clean_windows_path(manifest_path);
    let target_norm = normalize_path_for_compare(&manifest_str);
    let filename_suffix = format!("\\{}", MANIFEST_FILENAME.to_ascii_lowercase());

    let existing_paths: Vec<String> = obj
        .get("manifest_paths")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();

    let mut updated_paths = Vec::with_capacity(existing_paths.len() + 1);
    let mut found = false;
    for entry in &existing_paths {
        let norm = normalize_path_for_compare(entry);
        if norm == target_norm {
            if !found {
                updated_paths.push(manifest_str.clone());
                found = true;
            }
        } else if norm.ends_with(&filename_suffix) {
            // Skip stale manifest path from an older location
            continue;
        } else {
            updated_paths.push(entry.clone());
        }
    }
    if !found {
        updated_paths.push(manifest_str);
    }

    if updated_paths == existing_paths && appconfig_path.exists() {
        return Ok(());
    }

    obj.insert(
        "manifest_paths".to_string(),
        Value::Array(updated_paths.into_iter().map(Value::String).collect()),
    );
    write_json_atomic(appconfig_path, &root)
}

fn vrappconfig_path(steamvr_config_dir: &Path) -> PathBuf {
    steamvr_config_dir
        .join("vrappconfig")
        .join(format!("{STEAMVR_APP_KEY}.vrappconfig"))
}

pub fn read_vrappconfig_autolaunch(steamvr_config_dir: &Path) -> Option<bool> {
    let path = vrappconfig_path(steamvr_config_dir);
    let content = fs::read_to_string(path).ok()?;
    let value = serde_json::from_str::<Value>(&content).ok()?;
    value.get("autolaunch").and_then(|v| v.as_bool())
}

pub fn write_vrappconfig_autolaunch(
    steamvr_config_dir: &Path,
    enabled: bool,
) -> Result<(), String> {
    let path = vrappconfig_path(steamvr_config_dir);
    let mut root = if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
            .filter(|v| v.is_object())
            .unwrap_or_else(|| Value::Object(Map::new()))
    } else {
        Value::Object(Map::new())
    };

    if let Some(obj) = root.as_object_mut() {
        obj.insert("autolaunch".to_string(), Value::Bool(enabled));
        if !obj.contains_key("last_launch_time") {
            obj.insert("last_launch_time".to_string(), Value::String("0".into()));
        }
    }

    write_json_atomic(&path, &root)
}

fn get_manifest_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|dir| dir.join(MANIFEST_FILENAME))
        .map_err(|e| e.to_string())
}

pub fn read_auto_launch() -> Option<bool> {
    let steamvr_config_dir = find_steamvr_config_dir()?;
    read_vrappconfig_autolaunch(&steamvr_config_dir)
}

pub fn ensure_registered(app: &AppHandle) -> Result<(), String> {
    let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
    let manifest_path = get_manifest_path(app)?;
    write_manifest_file(&manifest_path, &exe_path)?;

    if let Some(steamvr_config_dir) = find_steamvr_config_dir() {
        let appconfig_path = steamvr_config_dir.join("appconfig.json");
        register_manifest_in_appconfig(&appconfig_path, &manifest_path)?;

        if read_vrappconfig_autolaunch(&steamvr_config_dir).is_none() {
            let config = load_config(app);
            if config.steamvr_auto_launch {
                write_vrappconfig_autolaunch(&steamvr_config_dir, true)?;
            }
        }
    }
    Ok(())
}

pub fn sync_auto_launch(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
    let manifest_path = get_manifest_path(app)?;
    write_manifest_file(&manifest_path, &exe_path)?;

    if let Some(steamvr_config_dir) = find_steamvr_config_dir() {
        let appconfig_path = steamvr_config_dir.join("appconfig.json");
        register_manifest_in_appconfig(&appconfig_path, &manifest_path)?;
        write_vrappconfig_autolaunch(&steamvr_config_dir, enabled)?;
    }
    Ok(())
}

pub fn stdout_has_process(stdout: &str, process_name: &str) -> bool {
    let needle = format!("\"{}\"", process_name.to_ascii_lowercase());
    stdout
        .lines()
        .any(|line| line.trim().to_ascii_lowercase().starts_with(&needle))
}

#[cfg(target_os = "windows")]
pub async fn is_steamvr_running() -> Option<bool> {
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let output = tokio::process::Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq vrmonitor.exe", "/FO", "CSV", "/NH"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .await
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Some(stdout_has_process(&stdout, "vrmonitor.exe"))
}

#[cfg(not(target_os = "windows"))]
pub async fn is_steamvr_running() -> Option<bool> {
    None
}

pub fn spawn_steamvr_exit_monitor(app: AppHandle) {
    tokio::spawn(async move {
        let mut seen_running = false;
        let mut stopped_streak = 0u8;
        let mut ticker = tokio::time::interval(Duration::from_secs(5));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            ticker.tick().await;
            let state = app.state::<AppState>();
            let should_watch = state.launched_by_steamvr.load(Ordering::Relaxed)
                || load_config(&app).steamvr_auto_launch;

            if !should_watch {
                seen_running = false;
                stopped_streak = 0;
                continue;
            }

            match is_steamvr_running().await {
                Some(true) => {
                    seen_running = true;
                    stopped_streak = 0;
                }
                Some(false) if seen_running => {
                    stopped_streak += 1;
                    if stopped_streak >= 2 {
                        state.cleanup();
                        app.exit(0);
                        break;
                    }
                }
                _ => {}
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_test_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("pico4u_steamvr_test_{label}_{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn detects_steamvr_launch_argument() {
        assert!(is_steamvr_launch_arg(["app.exe", "--steamvr-launch"]));
        assert!(!is_steamvr_launch_arg(["app.exe", "--other"]));
    }

    #[test]
    fn builds_valid_vrmanifest_and_strips_unc_prefix() {
        let exe = Path::new(r"\\?\C:\Program Files\pico4u\pico4u-vr-sleep-workaround.exe");
        let manifest = build_vrmanifest(exe);
        assert_eq!(manifest["source"], "builtin");
        let app = &manifest["applications"][0];
        assert_eq!(app["app_key"], STEAMVR_APP_KEY);
        assert_eq!(app["launch_type"], "binary");
        assert_eq!(
            app["binary_path_windows"],
            r"C:\Program Files\pico4u\pico4u-vr-sleep-workaround.exe"
        );
        assert_eq!(app["arguments"], STEAMVR_LAUNCH_ARG);
        assert_eq!(app["is_dashboard_overlay"], true);
        assert_eq!(app["strings"]["ja_jp"]["name"], "pico4u V睡ツール");
        assert_eq!(
            app["strings"]["en_us"]["name"],
            "pico4u VR Sleep Workaround"
        );
    }

    #[test]
    fn parses_openvrpaths_config_entries() {
        let raw = r#"{
            "config" : [ "C:\\Program Files (x86)\\Steam\\config" ],
            "jsonid" : "vrpathreg",
            "version" : 1
        }"#;
        let dirs = parse_openvr_config_dirs(raw);
        assert_eq!(
            dirs,
            vec![PathBuf::from(r"C:\Program Files (x86)\Steam\config")]
        );
        assert!(parse_openvr_config_dirs("invalid json").is_empty());
    }

    #[test]
    fn registers_manifest_without_duplicates_and_replaces_stale_entry() {
        let dir = temp_test_dir("appconfig");
        let appconfig = dir.join("appconfig.json");
        fs::write(
            &appconfig,
            r#"{
  "manifest_paths": [
    "C:\\Program Files (x86)\\Steam\\config\\steamapps.vrmanifest",
    "C:\\OldPath\\pico4u-vr-sleep-workaround.vrmanifest"
  ],
  "extra_key": true
}"#,
        )
        .unwrap();

        let new_manifest = Path::new(r"C:\NewPath\pico4u-vr-sleep-workaround.vrmanifest");
        register_manifest_in_appconfig(&appconfig, new_manifest).unwrap();
        register_manifest_in_appconfig(&appconfig, new_manifest).unwrap();

        let parsed: Value =
            serde_json::from_str(&fs::read_to_string(&appconfig).unwrap()).unwrap();
        assert_eq!(parsed["extra_key"], true);
        let paths = parsed["manifest_paths"].as_array().unwrap();
        assert_eq!(paths.len(), 2);
        assert_eq!(
            paths[0],
            r"C:\Program Files (x86)\Steam\config\steamapps.vrmanifest"
        );
        assert_eq!(
            paths[1],
            r"C:\NewPath\pico4u-vr-sleep-workaround.vrmanifest"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn reads_and_writes_vrappconfig_preserving_existing_fields() {
        let dir = temp_test_dir("vrappconfig");
        assert_eq!(read_vrappconfig_autolaunch(&dir), None);

        write_vrappconfig_autolaunch(&dir, true).unwrap();
        assert_eq!(read_vrappconfig_autolaunch(&dir), Some(true));

        let path = vrappconfig_path(&dir);
        fs::write(
            &path,
            r#"{"autolaunch":true,"last_launch_time":"1700000000"}"#,
        )
        .unwrap();

        write_vrappconfig_autolaunch(&dir, false).unwrap();
        assert_eq!(read_vrappconfig_autolaunch(&dir), Some(false));

        let parsed: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(parsed["last_launch_time"], "1700000000");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parses_tasklist_csv_output_across_locales() {
        let running_out = "\"vrmonitor.exe\",\"12408\",\"Console\",\"1\",\"142,500 K\"\r\n";
        assert!(stdout_has_process(running_out, "vrmonitor.exe"));

        let not_running_en = "INFO: No tasks are running which match the specified criteria.\r\n";
        assert!(!stdout_has_process(not_running_en, "vrmonitor.exe"));

        let not_running_ja = "情報: 指定された条件に一致するタスクは実行されていません。\r\n";
        assert!(!stdout_has_process(not_running_ja, "vrmonitor.exe"));
    }
}
