use crate::adb_client::{run_adb_device_command, run_adb_host_command};
use std::net::{Ipv4Addr, SocketAddrV4};
use tauri::AppHandle;

#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
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

#[derive(Debug, PartialEq)]
pub struct Device {
    pub serial: String,
    pub model: Option<String>,
}

pub fn parse_devices(output: &str) -> Vec<Device> {
    output
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let serial = fields.next()?;
            if fields.next()? != "device" {
                return None;
            }
            Some(Device {
                serial: serial.into(),
                model: fields.find_map(|field| field.strip_prefix("model:").map(str::to_owned)),
            })
        })
        .collect()
}

pub fn endpoint(ip: &str) -> Result<String, String> {
    let ip = ip.trim();
    let address = if ip.contains(':') {
        ip.parse::<SocketAddrV4>().map_err(|_| "invalid_ip")?
    } else {
        SocketAddrV4::new(ip.parse::<Ipv4Addr>().map_err(|_| "invalid_ip")?, 5555)
    };
    if address.port() == 0
        || address.ip().is_unspecified()
        || address.ip().is_multicast()
        || address.ip().is_broadcast()
    {
        return Err("invalid_ip".into());
    }
    Ok(address.to_string())
}

pub fn is_pico(model: &str) -> bool {
    matches!(model.trim(), "A9210" | "PICO 4 Ultra" | "PICO_4_Ultra")
}

pub async fn verify_pico(app: &AppHandle, serial: &str) -> Result<bool, String> {
    let model =
        run_adb_device_command(Some(app), Some(serial), "shell:getprop ro.product.model").await?;
    Ok(is_pico(&model))
}

fn matches_transport(serial: &str, mode: ConnectionMode, target: &str) -> bool {
    match mode {
        ConnectionMode::Wired => !serial.contains(':') && !serial.starts_with("adb-"),
        ConnectionMode::Wireless => serial == target,
    }
}

pub async fn target_device(
    app: &AppHandle,
    mode: ConnectionMode,
    ip: &str,
) -> Result<Option<String>, String> {
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
        _ => Err("multiple_pico_devices".into()),
    }
}

pub async fn require_target(
    app: &AppHandle,
    mode: ConnectionMode,
    ip: &str,
) -> Result<String, String> {
    target_device(app, mode, ip)
        .await?
        .ok_or_else(|| "pico_not_connected".into())
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

pub async fn wifi_ip(app: &AppHandle, serial: &str) -> Result<Option<String>, String> {
    let output =
        run_adb_device_command(Some(app), Some(serial), "shell:ip addr show wlan0").await?;
    Ok(parse_wifi_ip(&output))
}

pub async fn status(
    app: &AppHandle,
    mode: ConnectionMode,
    ip: &str,
) -> Result<ConnectionStatus, String> {
    let mut status = ConnectionStatus::default();
    let usb = target_device(app, ConnectionMode::Wired, "").await?;
    let tcp = if mode == ConnectionMode::Wireless && !ip.trim().is_empty() {
        target_device(app, mode, ip).await?
    } else {
        None
    };
    status.is_usb_connected = usb.is_some();
    status.is_adb_tcp_connected = tcp.is_some();
    if let Some(tcp_serial) = tcp.as_deref() {
        // Only show the USB cable for the same physical headset as the TCP target.
        if let Some(usb_serial) = usb.as_deref() {
            let identity =
                run_adb_device_command(Some(app), Some(tcp_serial), "shell:getprop ro.serialno")
                    .await?;
            status.is_usb_connected = identity.trim() == usb_serial;
        }
    }
    if let Some(serial) = tcp.as_deref().or(usb.as_deref()) {
        status.wifi_ip = wifi_ip(app, serial).await?;
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
