use std::time::Duration;
use tauri::{AppHandle, Manager};
use tauri_plugin_shell::ShellExt;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

const ADB_PORT: u16 = 5037;
const TIMEOUT: Duration = Duration::from_secs(5);

async fn read_adb_status<R: AsyncRead + Unpin>(
    stream: &mut R,
    context: &str,
) -> Result<(), String> {
    let mut status = [0u8; 4];
    stream
        .read_exact(&mut status)
        .await
        .map_err(|e| format!("ADB {context}: failed to read status: {e}"))?;

    match &status {
        b"OKAY" => Ok(()),
        b"FAIL" => {
            let mut length = [0u8; 4];
            stream
                .read_exact(&mut length)
                .await
                .map_err(|e| format!("ADB {context}: failed to read failure reason length: {e}"))?;

            if !length.iter().all(u8::is_ascii_hexdigit) {
                return Err(format!(
                    "ADB {context}: invalid failure reason length: {length:?}"
                ));
            }
            let length = std::str::from_utf8(&length)
                .ok()
                .and_then(|s| usize::from_str_radix(s, 16).ok())
                .ok_or_else(|| format!("ADB {context}: invalid failure reason length"))?;

            let mut reason = vec![0u8; length];
            stream
                .read_exact(&mut reason)
                .await
                .map_err(|e| format!("ADB {context}: failed to read failure reason body: {e}"))?;

            let reason = String::from_utf8_lossy(&reason);
            let reason = if reason.is_empty() {
                "no failure reason provided"
            } else {
                &reason
            };
            Err(format!("ADB {context} rejected: {reason}"))
        }
        _ => Err(format!("ADB {context}: unexpected status: {status:?}")),
    }
}

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

        read_adb_status(&mut stream, &format!("host request '{command}'")).await?;

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

        read_adb_status(
            &mut stream,
            &format!("transport selection '{transport_req}' for device command '{command}'"),
        )
        .await?;

        let req = format!("{:04x}{}", command.len(), command);
        stream
            .write_all(req.as_bytes())
            .await
            .map_err(|e| e.to_string())?;

        read_adb_status(
            &mut stream,
            &format!("device command '{command}' via '{transport_req}'"),
        )
        .await?;

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

#[cfg(test)]
mod tests {
    use super::read_adb_status;
    use tokio::io::{AsyncWriteExt, duplex};

    const CONTEXT: &str =
        "transport selection 'host:transport-any' for device command 'tcpip:5555'";

    #[tokio::test]
    async fn okay_leaves_the_success_payload_unread() {
        let mut response = &b"OKAY0004test"[..];
        assert_eq!(read_adb_status(&mut response, CONTEXT).await, Ok(()));
        assert_eq!(response, b"0004test");
    }

    #[tokio::test]
    async fn failure_preserves_the_reason_and_operation_context() {
        let contexts = [
            "host request 'host:connect:192.168.1.20:5555'",
            CONTEXT,
            "device command 'tcpip:5555' via 'host:transport:PICO123'",
        ];
        let reasons = [
            "more than one device/emulator",
            "device offline",
            "device unauthorized.\nPlease check the confirmation dialog on your device.",
            "デバイスに接続できません",
        ];
        for context in contexts {
            for reason in reasons {
                let reply = format!("FAIL{:04x}{reason}TAIL", reason.len());
                let mut response = reply.as_bytes();
                let error = read_adb_status(&mut response, context).await.unwrap_err();
                assert_eq!(error, format!("ADB {context} rejected: {reason}"));
                assert_eq!(response, b"TAIL");
            }
        }
    }

    #[tokio::test]
    async fn reads_status_and_failure_across_fragmented_receives() {
        let reason = "device offline";
        let reply = format!("OKAYFAIL{:04x}{reason}", reason.len());
        // A one-byte buffer forces the reader to assemble each protocol field.
        let (mut reader, mut writer) = duplex(1);
        let send = tokio::spawn(async move {
            writer.write_all(reply.as_bytes()).await.unwrap();
        });

        assert_eq!(read_adb_status(&mut reader, CONTEXT).await, Ok(()));
        assert_eq!(
            read_adb_status(&mut reader, CONTEXT).await.unwrap_err(),
            format!("ADB {CONTEXT} rejected: {reason}")
        );
        send.await.unwrap();
    }

    #[tokio::test]
    async fn empty_failure_still_reports_rejection() {
        let mut response = &b"FAIL0000"[..];
        assert_eq!(
            read_adb_status(&mut response, CONTEXT).await.unwrap_err(),
            format!("ADB {CONTEXT} rejected: no failure reason provided")
        );
    }

    #[tokio::test]
    async fn truncated_status_reports_a_status_read_error() {
        let mut response = &b"OK"[..];
        let error = read_adb_status(&mut response, CONTEXT).await.unwrap_err();
        assert!(error.starts_with(&format!("ADB {CONTEXT}: failed to read status:")));
    }

    #[tokio::test]
    async fn truncated_failure_length_reports_a_length_read_error() {
        let mut response = &b"FAIL00"[..];
        let error = read_adb_status(&mut response, CONTEXT).await.unwrap_err();
        assert!(error.starts_with(&format!(
            "ADB {CONTEXT}: failed to read failure reason length:"
        )));
    }

    #[tokio::test]
    async fn rejects_non_hexadecimal_failure_lengths() {
        for reply in [b"FAILzzzz", b"FAIL+001", b"FAIL\xff000"] {
            let mut response = &reply[..];
            let error = read_adb_status(&mut response, CONTEXT).await.unwrap_err();
            assert!(error.starts_with(&format!("ADB {CONTEXT}: invalid failure reason length:")));
        }
    }

    #[tokio::test]
    async fn truncated_failure_body_reports_a_body_read_error() {
        let mut response = &b"FAIL0005no"[..];
        let error = read_adb_status(&mut response, CONTEXT).await.unwrap_err();
        assert!(error.starts_with(&format!(
            "ADB {CONTEXT}: failed to read failure reason body:"
        )));
    }

    #[tokio::test]
    async fn unknown_status_reports_a_protocol_error() {
        let mut response = &b"NOPE"[..];
        let error = read_adb_status(&mut response, CONTEXT).await.unwrap_err();
        assert!(error.starts_with(&format!("ADB {CONTEXT}: unexpected status:")));
    }
}
