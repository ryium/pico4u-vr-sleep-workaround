use std::time::Duration;
use tauri::{AppHandle, Manager};
use tauri_plugin_shell::ShellExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

const ADB_PORT: u16 = 5037;
const TIMEOUT: Duration = Duration::from_secs(5);

pub async fn run_adb_host_command(
    app: Option<&AppHandle>,
    command: &str,
) -> Result<String, String> {
    timeout(TIMEOUT, async {
        let mut stream = connect_adb(app).await?;

        let req = format!("{:04x}{}", command.len(), command);
        stream
            .write_all(req.as_bytes())
            .await
            .map_err(|e| e.to_string())?;

        let mut status = [0u8; 4];
        stream
            .read_exact(&mut status)
            .await
            .map_err(|e| e.to_string())?;

        if &status != b"OKAY" {
            return Err("ADB server rejected request".to_string());
        }

        let mut len_buf = [0u8; 4];
        stream
            .read_exact(&mut len_buf)
            .await
            .map_err(|e| e.to_string())?;

        let len_str = std::str::from_utf8(&len_buf).map_err(|e| e.to_string())?;
        let len = usize::from_str_radix(len_str, 16).map_err(|e| e.to_string())?;

        if len > 0 {
            let mut data = vec![0u8; len];
            stream
                .read_exact(&mut data)
                .await
                .map_err(|e| e.to_string())?;
            Ok(String::from_utf8_lossy(&data).to_string())
        } else {
            Ok("".to_string())
        }
    })
    .await
    .map_err(|_| "ADB host command timed out".to_string())?
}

pub async fn run_adb_device_command(
    app: Option<&AppHandle>,
    serial: Option<&str>,
    command: &str,
) -> Result<String, String> {
    timeout(TIMEOUT, async {
        let mut stream = connect_adb(app).await?;

        let transport_req = if let Some(s) = serial {
            format!("host:transport:{}", s)
        } else {
            "host:transport-any".to_string()
        };

        let req = format!("{:04x}{}", transport_req.len(), transport_req);
        stream
            .write_all(req.as_bytes())
            .await
            .map_err(|e| e.to_string())?;

        let mut status = [0u8; 4];
        stream
            .read_exact(&mut status)
            .await
            .map_err(|e| e.to_string())?;
        if &status != b"OKAY" {
            return Err("ADB server rejected transport. Is the device connected?".to_string());
        }

        let req = format!("{:04x}{}", command.len(), command);
        stream
            .write_all(req.as_bytes())
            .await
            .map_err(|e| e.to_string())?;

        stream
            .read_exact(&mut status)
            .await
            .map_err(|e| e.to_string())?;
        if &status != b"OKAY" {
            return Err(format!("ADB server rejected device command '{}'", command));
        }

        let mut output = String::new();
        stream
            .read_to_string(&mut output)
            .await
            .map_err(|e| e.to_string())?;

        Ok(output)
    })
    .await
    .map_err(|_| "ADB device command timed out".to_string())?
}

async fn connect_adb(app: Option<&AppHandle>) -> Result<TcpStream, String> {
    if let Ok(stream) = TcpStream::connect(("127.0.0.1", ADB_PORT)).await {
        return Ok(stream);
    }
    let app = app.ok_or("ADB server unavailable")?;
    let state = app.state::<crate::state::AppState>();
    let _guard = state.adb_start.lock().await;
    if let Ok(stream) = TcpStream::connect(("127.0.0.1", ADB_PORT)).await {
        return Ok(stream);
    }
    // Foreground mode gives us a concrete child handle. start-server daemonizes,
    // so a boolean cannot safely identify which process to terminate at exit.
    if let Some(child) = state.adb_child.lock().unwrap().take() {
        let _ = child.kill();
    }
    let (mut events, child) = app
        .shell()
        .sidecar("adb")
        .map_err(|e| e.to_string())?
        .args(["server", "nodaemon"])
        .spawn()
        .map_err(|e| e.to_string())?;
    *state.adb_child.lock().unwrap() = Some(child);
    tokio::spawn(async move { while events.recv().await.is_some() {} });
    for _ in 0..20 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        if let Ok(stream) = TcpStream::connect(("127.0.0.1", ADB_PORT)).await {
            return Ok(stream);
        }
    }
    Err("Failed to start ADB server".into())
}
