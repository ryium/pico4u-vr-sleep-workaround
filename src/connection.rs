use crate::adb_client::{run_adb_device_command, run_adb_host_command};
use crate::error::{AppError, ErrorCode};
use std::net::{Ipv4Addr, SocketAddrV4};
use tauri::AppHandle;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionMode {
    Wired,
    Wireless,
}

#[derive(Default, serde::Serialize)]
pub struct ConnectionStatus {
    pub is_usb_connected: bool,
    pub is_wifi_ip_available: bool,
    pub wifi_ip: Option<String>,
    pub is_adb_tcp_connected: bool,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DeviceState {
    Device,
    Unauthorized,
    Offline,
    Other(String),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DeviceRow {
    pub serial: String,
    pub state: DeviceState,
    pub model: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Device {
    pub serial: String,
    pub model: Option<String>,
}

pub fn parse_device_rows(output: &str) -> Vec<DeviceRow> {
    output
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("List of devices attached") {
                return None;
            }
            let mut fields = trimmed.split_whitespace();
            let serial = fields.next()?.to_string();
            let state_str = fields.next()?;
            let state = match state_str {
                "device" => DeviceState::Device,
                "unauthorized" => DeviceState::Unauthorized,
                "offline" => DeviceState::Offline,
                other => DeviceState::Other(other.to_string()),
            };
            let model = fields.find_map(|field| field.strip_prefix("model:").map(str::to_owned));
            Some(DeviceRow {
                serial,
                state,
                model,
            })
        })
        .collect()
}

pub fn parse_devices(output: &str) -> Vec<Device> {
    parse_device_rows(output)
        .into_iter()
        .filter_map(|row| {
            if row.state == DeviceState::Device {
                Some(Device {
                    serial: row.serial,
                    model: row.model,
                })
            } else {
                None
            }
        })
        .collect()
}

#[derive(Debug, PartialEq, Eq)]
pub enum UsbProbe {
    Ready(Vec<String>),
    Unauthorized,
    Offline,
    Missing,
}

pub fn probe_usb(rows: &[DeviceRow]) -> UsbProbe {
    let mut ready = Vec::new();
    let mut has_unauthorized = false;
    let mut has_offline = false;

    for row in rows {
        if matches_transport(&row.serial, ConnectionMode::Wired, "") {
            match row.state {
                DeviceState::Device => ready.push(row.serial.clone()),
                DeviceState::Unauthorized => has_unauthorized = true,
                DeviceState::Offline => has_offline = true,
                _ => {}
            }
        }
    }

    if !ready.is_empty() {
        UsbProbe::Ready(ready)
    } else if has_unauthorized {
        UsbProbe::Unauthorized
    } else if has_offline {
        UsbProbe::Offline
    } else {
        UsbProbe::Missing
    }
}

pub fn wireless_candidates(rows: &[DeviceRow], saved_ip: Option<&str>) -> Vec<String> {
    let saved_endpoint = saved_ip.and_then(|ip| endpoint(ip).ok());
    let mut candidates = Vec::new();

    for row in rows {
        if row.state == DeviceState::Device
            && matches_transport(&row.serial, ConnectionMode::Wireless, "")
        {
            candidates.push(row.serial.clone());
        }
    }

    if let Some(ref target) = saved_endpoint
        && let Some(pos) = candidates.iter().position(|c| c == target)
    {
        let prioritized = candidates.remove(pos);
        candidates.insert(0, prioritized);
    }

    candidates
}

pub fn parse_wifi_on(output: &str) -> Option<bool> {
    let trimmed = output.trim();
    match trimmed {
        "0" => Some(false),
        "1" | "2" | "3" => Some(true),
        _ => None,
    }
}

pub async fn wifi_enabled(app: &AppHandle, serial: &str) -> Option<bool> {
    let output =
        run_adb_device_command(Some(app), Some(serial), "shell:settings get global wifi_on")
            .await
            .ok()?;
    parse_wifi_on(&output)
}

pub fn endpoint(ip: &str) -> Result<String, AppError> {
    let ip = ip.trim();
    let address = if ip.contains(':') {
        ip.parse::<SocketAddrV4>()
            .map_err(|_| AppError::new(ErrorCode::InvalidIp))?
    } else {
        SocketAddrV4::new(
            ip.parse::<Ipv4Addr>()
                .map_err(|_| AppError::new(ErrorCode::InvalidIp))?,
            5555,
        )
    };
    if address.port() == 0
        || address.ip().is_unspecified()
        || address.ip().is_multicast()
        || address.ip().is_broadcast()
    {
        return Err(AppError::new(ErrorCode::InvalidIp));
    }
    Ok(address.to_string())
}

pub fn is_pico(model: &str) -> bool {
    matches!(model.trim(), "A9210" | "PICO 4 Ultra" | "PICO_4_Ultra")
}

pub async fn verify_pico(app: &AppHandle, serial: &str) -> Result<bool, AppError> {
    let model =
        run_adb_device_command(Some(app), Some(serial), "shell:getprop ro.product.model").await?;
    Ok(is_pico(&model))
}

pub fn matches_transport(serial: &str, mode: ConnectionMode, target: &str) -> bool {
    match mode {
        ConnectionMode::Wired => !serial.contains(':') && !serial.starts_with("adb-"),
        ConnectionMode::Wireless => {
            if target.is_empty() {
                serial.contains(':') && !serial.starts_with("adb-")
            } else {
                serial == target
            }
        }
    }
}

pub async fn target_device(
    app: &AppHandle,
    mode: ConnectionMode,
    ip: &str,
) -> Result<Option<String>, AppError> {
    let target = if mode == ConnectionMode::Wireless {
        endpoint(ip)?
    } else {
        String::new()
    };
    let output = run_adb_host_command(Some(app), "host:devices-l").await?;
    let mut candidates = Vec::new();
    for device in parse_devices(&output)
        .into_iter()
        .filter(|d| matches_transport(&d.serial, mode, &target))
    {
        if device.model.as_deref().is_some_and(|model| !is_pico(model)) {
            continue;
        }
        // Never use transport-any: every command must address a verified headset.
        if verify_pico(app, &device.serial).await? {
            candidates.push(device.serial);
        }
    }
    match candidates.len() {
        0 => Ok(None),
        1 => Ok(candidates.pop()),
        _ => Err(AppError::new(ErrorCode::MultiplePico)),
    }
}

pub async fn require_target(
    app: &AppHandle,
    mode: ConnectionMode,
    ip: &str,
) -> Result<String, AppError> {
    if let Some(target) = target_device(app, mode, ip).await? {
        return Ok(target);
    }

    let output = run_adb_host_command(Some(app), "host:devices-l").await?;
    let rows = parse_device_rows(&output);
    match mode {
        ConnectionMode::Wired => match probe_usb(&rows) {
            UsbProbe::Ready(_) => Err(AppError::new(ErrorCode::NotPico)),
            UsbProbe::Unauthorized => Err(AppError::new(ErrorCode::UsbUnauthorized)),
            UsbProbe::Offline => Err(AppError::new(ErrorCode::UsbOffline)),
            UsbProbe::Missing => Err(AppError::new(ErrorCode::UsbNotFound)),
        },
        ConnectionMode::Wireless => Err(AppError::new(ErrorCode::WirelessNotFound)),
    }
}

pub fn parse_wifi_ip(output: &str) -> Option<String> {
    output.lines().find_map(|line| {
        let mut fields = line.split_whitespace();
        if fields.next()? != "inet" {
            return None;
        }
        let ip = fields.next()?.split('/').next()?.parse::<Ipv4Addr>().ok()?;
        Some(ip.to_string())
    })
}

pub async fn wifi_ip(app: &AppHandle, serial: &str) -> Result<Option<String>, AppError> {
    let output =
        run_adb_device_command(Some(app), Some(serial), "shell:ip addr show wlan0").await?;
    Ok(parse_wifi_ip(&output))
}

pub async fn status(
    app: &AppHandle,
    mode: ConnectionMode,
    ip: &str,
) -> Result<ConnectionStatus, AppError> {
    let mut status = ConnectionStatus::default();
    let usb = target_device(app, ConnectionMode::Wired, "")
        .await
        .ok()
        .flatten();
    let tcp = if mode == ConnectionMode::Wireless && !ip.trim().is_empty() {
        target_device(app, mode, ip).await.ok().flatten()
    } else {
        None
    };
    status.is_usb_connected = usb.is_some();
    status.is_adb_tcp_connected = tcp.is_some();
    if let Some(tcp_serial) = tcp.as_deref() {
        // Only show the USB cable for the same physical headset as the TCP target.
        if let Some(usb_serial) = usb.as_deref()
            && let Ok(identity) =
                run_adb_device_command(Some(app), Some(tcp_serial), "shell:getprop ro.serialno")
                    .await
        {
            status.is_usb_connected = identity.trim() == usb_serial;
        }
    }
    if let Some(serial) = tcp.as_deref().or(usb.as_deref())
        && let Ok(ip) = wifi_ip(app, serial).await
    {
        status.wifi_ip = ip;
        status.is_wifi_ip_available = status.wifi_ip.is_some();
    }
    Ok(status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_only_authorized_device_rows() {
        let rows = parse_devices(
            "List of devices attached\nusb unauthorized\n192.168.1.4:5555 offline\nusb2\tdevice product:pico model:A9210 transport_id:1\nphone device model:Pixel\ngarbage\n",
        );
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[0],
            Device {
                serial: "usb2".into(),
                model: Some("A9210".into())
            }
        );
        assert!(!is_pico(rows[1].model.as_deref().unwrap()));
    }

    #[test]
    fn parses_device_rows_with_states() {
        let text = "List of devices attached\nPA8250MGL1234\tdevice product:pico model:A9210\nusb_unauth\tunauthorized\nusb_off\toffline\n";
        let rows = parse_device_rows(text);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].state, DeviceState::Device);
        assert_eq!(rows[0].model.as_deref(), Some("A9210"));
        assert_eq!(rows[1].state, DeviceState::Unauthorized);
        assert_eq!(rows[2].state, DeviceState::Offline);
    }

    #[test]
    fn test_usb_probe() {
        let rows = parse_device_rows("PA8250MGL1234\tdevice model:A9210\n");
        assert_eq!(
            probe_usb(&rows),
            UsbProbe::Ready(vec!["PA8250MGL1234".into()])
        );

        let rows = parse_device_rows("PA8250MGL1234\tunauthorized\n");
        assert_eq!(probe_usb(&rows), UsbProbe::Unauthorized);

        let rows = parse_device_rows("PA8250MGL1234\toffline\n");
        assert_eq!(probe_usb(&rows), UsbProbe::Offline);

        let rows = parse_device_rows("192.168.1.20:5555\tdevice\n");
        assert_eq!(probe_usb(&rows), UsbProbe::Missing);
    }

    #[test]
    fn test_wireless_candidates() {
        let rows = parse_device_rows("192.168.1.50:5555\tdevice\n192.168.1.20:5555\tdevice\n");
        let candidates = wireless_candidates(&rows, Some("192.168.1.20"));
        assert_eq!(
            candidates,
            vec![
                "192.168.1.20:5555".to_string(),
                "192.168.1.50:5555".to_string()
            ]
        );
    }

    #[test]
    fn test_wifi_on_parsing() {
        assert_eq!(parse_wifi_on("0"), Some(false));
        assert_eq!(parse_wifi_on("1\n"), Some(true));
        assert_eq!(parse_wifi_on(" 2 "), Some(true));
        assert_eq!(parse_wifi_on("unknown"), None);
    }

    #[test]
    fn selection_requires_exact_endpoint_and_transport() {
        assert!(!matches_transport(
            "192.168.1.40:5555",
            ConnectionMode::Wireless,
            "192.168.1.4:5555"
        ));
        assert!(!matches_transport(
            "192.168.1.4:5555",
            ConnectionMode::Wired,
            ""
        ));
        assert!(!matches_transport(
            "adb-serial-service",
            ConnectionMode::Wired,
            ""
        ));
        assert!(matches_transport(
            "serial.with.dot",
            ConnectionMode::Wired,
            ""
        ));
        assert!(!is_pico("not-A9210"));
    }

    #[test]
    fn validates_addresses_and_parses_wlan_output() {
        assert_eq!(endpoint(" 192.168.1.4 ").unwrap(), "192.168.1.4:5555");
        assert_eq!(endpoint("192.168.1.4:1234").unwrap(), "192.168.1.4:1234");
        for ip in ["", "abc", "192.168.1.400", "1.2.3.4:0", "1.2.3.4;reboot"] {
            assert!(endpoint(ip).is_err());
        }
        assert_eq!(
            parse_wifi_ip("inet6 ::1\n inet 192.168.1.4/24 brd 192.168.1.255"),
            Some("192.168.1.4".into())
        );
        assert_eq!(parse_wifi_ip("inet6 fe80::1/64"), None);
    }
}
