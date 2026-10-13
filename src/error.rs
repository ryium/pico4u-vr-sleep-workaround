use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    AdbUnavailable,
    Timeout,
    AlreadyRunning,
    InvalidIp,
    MultiplePico,
    UsbNotFound,
    UsbUnauthorized,
    UsbOffline,
    NotPico,
    WifiDisabled,
    WifiNotConnected,
    TcpipFailed,
    WirelessRefused,
    WirelessUnreachable,
    WirelessUnauthorized,
    WirelessNotFound,
    DeviceLost,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AppError {
    pub code: ErrorCode,
    pub detail: Option<String>,
}

impl AppError {
    pub fn new(code: ErrorCode) -> Self {
        Self { code, detail: None }
    }

    pub fn with(code: ErrorCode, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: Some(detail.into()),
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref detail) = self.detail {
            write!(f, "{:?}: {}", self.code, detail)
        } else {
            write!(f, "{:?}", self.code)
        }
    }
}

impl std::error::Error for AppError {}

impl From<AppError> for String {
    fn from(err: AppError) -> Self {
        err.to_string()
    }
}

impl From<String> for AppError {
    fn from(raw: String) -> Self {
        let code = if raw.contains("Failed to start ADB server")
            || raw.contains("ADB server unavailable")
        {
            ErrorCode::AdbUnavailable
        } else if raw.contains("timed out") || raw == "connection_timeout" {
            ErrorCode::Timeout
        } else if raw.contains("unauthorized") {
            ErrorCode::UsbUnauthorized
        } else if raw.contains("offline") {
            ErrorCode::UsbOffline
        } else if raw == "already_running" {
            ErrorCode::AlreadyRunning
        } else if raw == "invalid_ip" {
            ErrorCode::InvalidIp
        } else if raw == "multiple_pico_devices" {
            ErrorCode::MultiplePico
        } else if raw == "pico_not_connected" {
            ErrorCode::UsbNotFound
        } else {
            ErrorCode::Unknown
        };
        Self {
            code,
            detail: Some(raw),
        }
    }
}

impl From<&str> for AppError {
    fn from(raw: &str) -> Self {
        Self::from(raw.to_string())
    }
}

/// Classify adb `host:connect` response text.
/// adb host returns OKAY even if connection failed, with messages such as:
/// - "connected to 192.168.1.20:5555"
/// - "already connected to 192.168.1.20:5555"
/// - "cannot connect to 192.168.1.20:5555: No connection could be made because the target machine actively refused it. (10061)"
/// - "cannot connect to 192.168.1.20:5555: A connection attempt failed because the connected party did not properly respond after a period of time... (10060)"
/// - "failed to authenticate to 192.168.1.20:5555"
pub fn classify_connect_reply(reply: &str) -> Result<(), AppError> {
    let trimmed = reply.trim();
    let lower = trimmed.to_ascii_lowercase();

    if lower.starts_with("connected to") || lower.starts_with("already connected to") {
        return Ok(());
    }

    let code = if lower.contains("failed to authenticate") {
        ErrorCode::WirelessUnauthorized
    } else if lower.contains("(10061)")
        || lower.contains("refused")
        || lower.contains("接続を拒否されました")
    {
        ErrorCode::WirelessRefused
    } else if lower.contains("(10060)")
        || lower.contains("(10065)")
        || lower.contains("(10051)")
        || lower.contains("timed out")
        || lower.contains("unreachable")
        || lower.contains("no route")
        || lower.contains("到達不能")
        || lower.contains("タイムアウト")
    {
        ErrorCode::WirelessUnreachable
    } else {
        ErrorCode::Unknown
    };

    Err(AppError::with(code, trimmed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_connect_reply_success() {
        assert!(classify_connect_reply("connected to 192.168.1.20:5555").is_ok());
        assert!(classify_connect_reply("already connected to 192.168.1.20:5555\n").is_ok());
    }

    #[test]
    fn test_classify_connect_reply_failures() {
        let refused = "cannot connect to 192.168.1.20:5555: No connection could be made because the target machine actively refused it. (10061)";
        assert_eq!(
            classify_connect_reply(refused),
            Err(AppError::with(ErrorCode::WirelessRefused, refused))
        );

        let unreachable =
            "cannot connect to 192.168.1.20:5555: A connection attempt failed... (10060)";
        assert_eq!(
            classify_connect_reply(unreachable),
            Err(AppError::with(ErrorCode::WirelessUnreachable, unreachable))
        );

        let unauth = "failed to authenticate to 192.168.1.20:5555";
        assert_eq!(
            classify_connect_reply(unauth),
            Err(AppError::with(ErrorCode::WirelessUnauthorized, unauth))
        );

        let unknown = "something strange happened";
        assert_eq!(
            classify_connect_reply(unknown),
            Err(AppError::with(ErrorCode::Unknown, unknown))
        );
    }

    #[test]
    fn test_app_error_from_string() {
        let err = AppError::from("connection_timeout");
        assert_eq!(err.code, ErrorCode::Timeout);

        let err = AppError::from("device unauthorized.\nPlease check...");
        assert_eq!(err.code, ErrorCode::UsbUnauthorized);

        let err = AppError::from("device offline");
        assert_eq!(err.code, ErrorCode::UsbOffline);

        let err = AppError::from("Failed to start ADB server");
        assert_eq!(err.code, ErrorCode::AdbUnavailable);
    }

    #[test]
    fn test_serialization() {
        let err = AppError::with(ErrorCode::UsbNotFound, "none");
        let json = serde_json::to_string(&err).unwrap();
        assert_eq!(json, r#"{"code":"usb_not_found","detail":"none"}"#);
    }
}
