use crate::commands::SetupStep;
use crate::error::{AppError, ErrorCode};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Ja,
    En,
}

impl Language {
    pub fn detect() -> Self {
        if let Some(locale) = sys_locale::get_locale()
            && locale.to_ascii_lowercase().starts_with("ja")
        {
            return Self::Ja;
        }
        if let Ok(lang) = std::env::var("LANG")
            && lang.to_ascii_lowercase().starts_with("ja")
        {
            return Self::Ja;
        }
        Self::En
    }
}

pub struct I18n {
    pub lang: Language,
}

impl I18n {
    pub const fn new(lang: Language) -> Self {
        Self { lang }
    }

    fn pick<'a>(&self, en: &'a str, ja: &'a str) -> &'a str {
        match self.lang {
            Language::En => en,
            Language::Ja => ja,
        }
    }

    pub fn title(&self) -> &'static str {
        self.pick("pico4u VR Sleep Workaround", "pico4u V睡ツール")
    }

    pub fn btn_check_devices(&self) -> &'static str {
        self.pick("Check Devices (USB)", "デバイス確認 (USB)")
    }

    pub fn btn_setup_wireless(&self) -> &'static str {
        self.pick("Wireless Setup (Auto)", "無線セットアップ (自動)")
    }

    pub fn btn_connect_manual(&self) -> &'static str {
        self.pick("Manual IP Connection", "IP手動接続")
    }

    pub fn status_running(&self) -> &'static str {
        self.pick("RUNNING", "動作中")
    }

    pub fn status_stopped(&self) -> &'static str {
        self.pick("STOPPED", "停止中")
    }

    pub fn log_checking_devices(&self) -> &'static str {
        self.pick(
            "Checking for connected devices...",
            "接続されたデバイスを確認中...",
        )
    }

    pub fn log_wireless_init(&self) -> &'static str {
        self.pick(
            "Starting wireless setup...",
            "無線セットアップを開始しています...",
        )
    }

    pub fn log_wireless_complete(&self) -> &'static str {
        self.pick(
            "Wireless setup complete. You may now disconnect the USB cable.",
            "無線セットアップが完了しました。USBケーブルを外してください。",
        )
    }

    pub fn wireless_status_loading(&self) -> &'static str {
        self.pick(
            "Setting up wireless connection...",
            "無線接続をセットアップ中...",
        )
    }

    pub fn wireless_status_success(&self) -> &'static str {
        self.pick(
            "Wireless setup complete! You can now unplug the USB cable.",
            "無線セットアップ完了！USBケーブルを外してください",
        )
    }

    pub fn wireless_status_error(&self) -> &'static str {
        self.pick(
            "Wireless setup failed. Please check the suggestions below.",
            "無線セットアップに失敗しました。以下の確認手順をお試しください。",
        )
    }

    pub fn wired_status_loading(&self) -> &'static str {
        self.pick("Checking USB connection...", "USB接続を確認中...")
    }

    pub fn wired_status_success(&self) -> &'static str {
        self.pick(
            "USB device detected successfully!",
            "USBデバイスを検出しました！",
        )
    }

    pub fn wired_status_error(&self) -> &'static str {
        self.pick(
            "No USB device found. Please check your cable and try again.",
            "USBデバイスが見つかりません。ケーブルを確認して再試行してください",
        )
    }

    pub fn prompt_enter_ip(&self) -> &'static str {
        self.pick(
            "Enter the device IP address:",
            "デバイスのIPアドレスを入力:",
        )
    }

    pub fn log_connecting_manual(&self, ip: &str) -> String {
        match self.lang {
            Language::En => format!("Connecting to {ip}..."),
            Language::Ja => format!("{ip} に接続中..."),
        }
    }

    pub fn log_stopped_loop(&self) -> &'static str {
        self.pick("Sleep prevention stopped.", "スリープ回避を停止しました")
    }

    pub fn log_started_loop(&self, interval: u64) -> String {
        match self.lang {
            Language::En => {
                format!("Sleep prevention started (sending wake signal every {interval}s).")
            }
            Language::Ja => {
                format!("スリープ回避を開始しました（{interval}秒間隔で信号を送信）")
            }
        }
    }

    pub fn debug_mode(&self) -> &'static str {
        self.pick(
            "Debug Mode (Show event log)",
            "デバッグモード（イベントログを表示）",
        )
    }

    pub fn log_debug_enabled(&self) -> &'static str {
        self.pick("Debug mode enabled.", "デバッグモードを有効にしました。")
    }

    pub fn log_debug_disabled(&self) -> &'static str {
        self.pick("Debug mode disabled.", "デバッグモードを無効にしました。")
    }

    pub fn mode_wired(&self) -> &'static str {
        self.pick("Wired (USB)", "有線 (USB)")
    }

    pub fn mode_wireless(&self) -> &'static str {
        self.pick("Wireless (Wi-Fi)", "無線 (Wi-Fi)")
    }

    pub fn btn_continue(&self) -> &'static str {
        self.pick("Continue", "次へ")
    }

    pub fn btn_back(&self) -> &'static str {
        self.pick("Back", "戻る")
    }

    pub fn btn_apply_changes(&self) -> &'static str {
        self.pick("Apply Changes", "変更を適用")
    }

    pub fn settings_title(&self) -> &'static str {
        self.pick("Settings", "設定")
    }

    pub fn language_label(&self) -> &'static str {
        self.pick("Language", "言語")
    }

    pub fn theme_label(&self) -> &'static str {
        self.pick("Theme", "テーマ")
    }

    pub fn theme_light(&self) -> &'static str {
        self.pick("Light", "ライト")
    }

    pub fn theme_dark(&self) -> &'static str {
        self.pick("Dark", "ダーク")
    }

    pub fn theme_system(&self) -> &'static str {
        self.pick("System", "システムに合わせる")
    }

    pub fn status_checking(&self) -> &'static str {
        self.pick("Looking for device...", "デバイスを探しています...")
    }

    pub fn cancel(&self) -> &'static str {
        self.pick("Cancel", "キャンセル")
    }

    pub fn dim_setting_label(&self) -> &'static str {
        self.pick("Auto-dim screen after (hours):", "自動画面暗転（時間後）:")
    }

    pub fn dim_setting_note(&self) -> &'static str {
        self.pick(
            "Takes effect when sleep prevention starts",
            "スリープ回避の開始時に適用",
        )
    }

    pub fn keep_awake_interval_label(&self) -> &'static str {
        self.pick("Wake signal interval (seconds):", "信号の送信間隔（秒）:")
    }

    pub fn keep_awake_interval_note(&self) -> &'static str {
        self.pick(
            "The lower the value, the less likely the screen is to dim; however, if the value is too high, the floor level may be reset",
            "値が小さいほど画面が暗転しにくくなりますが、大きすぎるとフロアレベルがリセットされる可能性があります",
        )
    }

    pub fn tab_logs(&self) -> &'static str {
        self.pick("Logs", "ログ")
    }

    pub fn auto_connect_wired(&self, attempt: usize) -> String {
        match self.lang {
            Language::En => format!("Reconnecting via USB... (Attempt {attempt}/3)"),
            Language::Ja => format!("有線で再接続中... (試行 {attempt}/3)"),
        }
    }

    pub fn auto_connect_wireless(&self, ip: &str, attempt: usize) -> String {
        match self.lang {
            Language::En => {
                format!("Reconnecting via Wireless ({ip})... (Attempt {attempt}/3)")
            }
            Language::Ja => format!("無線({ip})で再接続中... (試行 {attempt}/3)"),
        }
    }

    pub fn log_auto_connect_success(&self, ip: &str) -> String {
        match self.lang {
            Language::En => format!("Auto-connected to {ip} successfully."),
            Language::Ja => format!("{ip} に自動接続しました。"),
        }
    }

    pub fn log_auto_connect_failed(&self, ip: &str) -> String {
        match self.lang {
            Language::En => format!("Auto-connect to {ip} failed. Please set up manually."),
            Language::Ja => {
                format!("{ip} への自動接続に失敗しました。手動でセットアップしてください。")
            }
        }
    }

    pub fn auto_connect_failed_desc(&self, ip: &str) -> String {
        match self.lang {
            Language::En => {
                format!("Could not reconnect to {ip}. Please follow the suggestions below.")
            }
            Language::Ja => {
                format!("{ip} に再接続できませんでした。以下の確認手順をお試しください。")
            }
        }
    }

    pub fn btn_wireless_setup(&self) -> &'static str {
        self.pick("Wireless Setup", "無線セットアップ")
    }

    pub fn btn_manual_setup(&self) -> &'static str {
        self.pick("Set Up Manually", "手動セットアップ")
    }

    pub fn btn_try_again(&self) -> &'static str {
        self.pick("Retry", "再試行")
    }

    pub fn setup_step_label(&self, step: SetupStep) -> &'static str {
        match step {
            SetupStep::Usb => self.pick("USB connection", "USB接続"),
            SetupStep::Authorized => self.pick("USB debugging authorization", "USBデバッグの許可"),
            SetupStep::Pico => self.pick("Verify PICO 4 Ultra", "PICO 4 Ultra を確認"),
            SetupStep::Wifi => self.pick("Wi-Fi enabled & IP address", "Wi-Fi 有効・IP取得"),
            SetupStep::Wireless => self.pick("Wireless ADB connection", "無線ADB接続確認"),
        }
    }

    pub fn setup_already_wireless(&self) -> &'static str {
        self.pick(
            "Already connected wirelessly — setup is not needed.",
            "既に無線接続済みです。セットアップは不要です。",
        )
    }

    pub fn details_toggle(&self) -> &'static str {
        self.pick("Technical Details", "詳細情報")
    }

    pub fn step_mode(&self) -> &'static str {
        self.pick("Mode", "接続方法")
    }

    pub fn step_prepare(&self) -> &'static str {
        self.pick("Setup", "準備")
    }

    pub fn step_operate(&self) -> &'static str {
        self.pick("Start", "開始")
    }

    pub fn mode_select_heading(&self) -> &'static str {
        self.pick("How would you like to connect?", "どうやって接続しますか？")
    }

    pub fn mode_select_subheading(&self) -> &'static str {
        self.pick(
            "You can change this later in Settings",
            "あとから設定で変更できます",
        )
    }

    pub fn mode_wired_desc(&self) -> &'static str {
        self.pick(
            "Just plug in the cable. Most reliable",
            "ケーブルでつなぐだけ。いちばん安定",
        )
    }

    pub fn mode_wireless_desc(&self) -> &'static str {
        self.pick(
            "Requires USB connection only for initial setup",
            "最初だけUSBで準備が必要です",
        )
    }

    pub fn badge_recommended(&self) -> &'static str {
        self.pick("Recommended", "おすすめ")
    }

    pub fn prepare_wireless_title(&self) -> &'static str {
        self.pick("Wireless Setup", "無線接続の準備")
    }

    pub fn prepare_wired_title(&self) -> &'static str {
        self.pick("Verify USB Connection", "有線接続の確認")
    }

    pub fn prepare_wireless_desc(&self) -> &'static str {
        self.pick(
            "Connect your headset to this PC via USB",
            "PCとヘッドセットをUSBでつないでください",
        )
    }

    pub fn prepare_wired_desc(&self) -> &'static str {
        self.pick(
            "Connect your headset to this PC via USB",
            "ヘッドセットをUSBでPCに接続してください",
        )
    }

    pub fn chip_wireless_connected(&self) -> &'static str {
        self.pick("Wireless", "無線接続中")
    }

    pub fn chip_wireless_connected_ip(&self, ip: &str) -> String {
        match self.lang {
            Language::En => format!("Wireless · {ip}"),
            Language::Ja => format!("無線接続中 · {ip}"),
        }
    }

    pub fn chip_wired_connected(&self) -> &'static str {
        self.pick("USB Connected", "USB接続中")
    }

    pub fn chip_disconnected(&self) -> &'static str {
        self.pick("Disconnected", "未接続")
    }

    pub fn power_btn_start(&self) -> &'static str {
        self.pick("START", "開始")
    }

    pub fn power_btn_tap_to_stop(&self) -> &'static str {
        self.pick("Tap to stop", "タップで停止")
    }

    pub fn power_btn_tap_to_start(&self) -> &'static str {
        self.pick("Start sleep prevention", "スリープ回避を開始")
    }

    pub fn operate_running_desc(&self, interval: u64) -> String {
        match self.lang {
            Language::En => format!("Sending wake signal every {interval}s"),
            Language::Ja => format!("{interval}秒ごとにスリープ回避信号を送信中"),
        }
    }

    pub fn operate_elapsed(&self, time: &str) -> String {
        match self.lang {
            Language::En => format!("Elapsed {time}"),
            Language::Ja => format!("経過 {time}"),
        }
    }

    pub fn operate_dim_after(&self, hours: f64) -> String {
        match self.lang {
            Language::En => format!("Auto-dim after {hours}h"),
            Language::Ja => format!("{hours}時間後に画面を暗くする"),
        }
    }

    pub fn btn_change_connection(&self) -> &'static str {
        self.pick("Change Connection", "接続を変更")
    }

    pub fn error_title(&self, code: ErrorCode) -> &'static str {
        error_title(self.lang, code)
    }

    pub fn error_hints(&self, code: ErrorCode) -> &'static [&'static str] {
        error_hints(self.lang, code)
    }
}

pub fn error_title(lang: Language, code: ErrorCode) -> &'static str {
    match (lang, code) {
        (Language::En, ErrorCode::UsbNotFound) => "Headset not found via USB",
        (Language::En, ErrorCode::UsbUnauthorized) => "USB debugging not authorized",
        (Language::En, ErrorCode::UsbOffline) => "Headset is offline",
        (Language::En, ErrorCode::NotPico) => "Unsupported device detected",
        (Language::En, ErrorCode::WifiDisabled) => "Headset Wi-Fi is turned off",
        (Language::En, ErrorCode::WifiNotConnected) => "Headset is not connected to Wi-Fi",
        (Language::En, ErrorCode::TcpipFailed) => "Failed to switch headset to wireless ADB mode",
        (Language::En, ErrorCode::WirelessRefused) => "Wireless connection refused",
        (Language::En, ErrorCode::WirelessUnreachable) => {
            "Cannot reach the headset over the network"
        }
        (Language::En, ErrorCode::WirelessUnauthorized) => "Wireless ADB authentication failed",
        (Language::En, ErrorCode::WirelessNotFound) => "Wireless device not found",
        (Language::En, ErrorCode::AdbUnavailable) => "Could not start or reach the ADB server",
        (Language::En, ErrorCode::Timeout) => "Connection operation timed out",
        (Language::En, ErrorCode::AlreadyRunning) => "Sleep prevention is already running",
        (Language::En, ErrorCode::InvalidIp) => "Invalid IP address",
        (Language::En, ErrorCode::MultiplePico) => "Multiple PICO devices detected",
        (Language::En, ErrorCode::DeviceLost) => "Connection to headset lost",
        (Language::En, ErrorCode::Unknown) => "An error occurred",

        (Language::Ja, ErrorCode::UsbNotFound) => "ヘッドセットがUSBで見つかりません",
        (Language::Ja, ErrorCode::UsbUnauthorized) => "USBデバッグが許可されていません",
        (Language::Ja, ErrorCode::UsbOffline) => "ヘッドセットが応答していません",
        (Language::Ja, ErrorCode::NotPico) => "PICO 4 Ultra 以外のデバイスが検出されました",
        (Language::Ja, ErrorCode::WifiDisabled) => "ヘッドセットのWi-Fiがオフになっています",
        (Language::Ja, ErrorCode::WifiNotConnected) => "ヘッドセットがWi-Fiに接続されていません",
        (Language::Ja, ErrorCode::TcpipFailed) => "無線ADBの切り替えに失敗しました",
        (Language::Ja, ErrorCode::WirelessRefused) => "無線接続が拒否されました",
        (Language::Ja, ErrorCode::WirelessUnreachable) => "ヘッドセットに到達できません",
        (Language::Ja, ErrorCode::WirelessUnauthorized) => "無線ADBの認証に失敗しました",
        (Language::Ja, ErrorCode::WirelessNotFound) => "無線デバイスが見つかりません",
        (Language::Ja, ErrorCode::AdbUnavailable) => "ADBサーバーを起動できませんでした",
        (Language::Ja, ErrorCode::Timeout) => "接続処理がタイムアウトしました",
        (Language::Ja, ErrorCode::AlreadyRunning) => "スリープ回避が既に動作中です",
        (Language::Ja, ErrorCode::InvalidIp) => "無効なIPアドレスです",
        (Language::Ja, ErrorCode::MultiplePico) => "複数のPICOデバイスが接続されています",
        (Language::Ja, ErrorCode::DeviceLost) => "デバイスとの接続が切断されました",
        (Language::Ja, ErrorCode::Unknown) => "エラーが発生しました",
    }
}

pub fn error_hints(lang: Language, code: ErrorCode) -> &'static [&'static str] {
    match (lang, code) {
        (Language::En, ErrorCode::UsbNotFound) => &[
            "Reconnect the USB cable",
            "Make sure you are using a USB cable that supports data transfer",
            "Verify that Developer Mode and USB Debugging are enabled on the headset",
            "Note: USB ADB is disabled while the headset is in wireless ADB mode. If so, restart the headset.",
        ],
        (Language::En, ErrorCode::UsbUnauthorized) => &[
            "Put on the headset and accept the \"Allow USB debugging\" prompt (checking \"Always allow\" is recommended)",
        ],
        (Language::En, ErrorCode::UsbOffline) => &[
            "Unplug and reconnect the USB cable",
            "Wake up the headset from sleep",
        ],
        (Language::En, ErrorCode::NotPico) => {
            &["Make sure a PICO 4 Ultra (A9210) headset is connected"]
        }
        (Language::En, ErrorCode::WifiDisabled) => &[
            "Turn on Wi-Fi in the headset settings",
            "Connect to the same Wi-Fi network as this PC",
        ],
        (Language::En, ErrorCode::WifiNotConnected) => {
            &["Connect the headset to the same Wi-Fi network as this PC"]
        }
        (Language::En, ErrorCode::TcpipFailed) => &["Restart the headset and try again"],
        (Language::En, ErrorCode::WirelessRefused) => &[
            "Wireless ADB resets when the headset restarts. Connect via USB and run Wireless Setup again.",
        ],
        (Language::En, ErrorCode::WirelessUnreachable) => &[
            "Make sure the headset is powered on and awake",
            "Confirm the PC and headset are on the same Wi-Fi network (subnet)",
            "Check if the headset IP address has changed",
        ],
        (Language::En, ErrorCode::WirelessUnauthorized) => {
            &["Connect via USB and perform Wireless Setup again"]
        }
        (Language::En, ErrorCode::WirelessNotFound) => &[
            "Verify the target IP address",
            "Perform Wireless Setup via USB to enable wireless ADB",
        ],
        (Language::En, ErrorCode::AdbUnavailable) => {
            &["Check for conflicting ADB processes and restart the application"]
        }
        (Language::En, ErrorCode::Timeout) => {
            &["Check your cable and network connection, then try again"]
        }
        (Language::En, ErrorCode::AlreadyRunning) => {
            &["Stop sleep prevention before changing connection settings"]
        }
        (Language::En, ErrorCode::InvalidIp) => &["Enter a valid IPv4 address (e.g. 192.168.1.20)"],
        (Language::En, ErrorCode::MultiplePico) => &["Disconnect other PICO headsets to continue"],
        (Language::En, ErrorCode::DeviceLost) => &["Check the headset cable or Wi-Fi connection"],
        (Language::En, ErrorCode::Unknown) => &["Check your connection and try again"],

        (Language::Ja, ErrorCode::UsbNotFound) => &[
            "USBケーブルを挿し直してください",
            "データ転送に対応したUSBケーブルを使用しているか確認してください",
            "開発者モードとUSBデバッグが有効か確認してください",
            "※ヘッドセットが無線ADBモード中の場合、USB側のADBが無効になります。その場合はヘッドセットを再起動してください",
        ],
        (Language::Ja, ErrorCode::UsbUnauthorized) => &[
            "ヘッドセットを被り、画面に表示されている「USBデバッグを許可」ダイアログで許可してください（「常に許可」を推奨）",
        ],
        (Language::Ja, ErrorCode::UsbOffline) => &[
            "USBケーブルを抜き差ししてください",
            "ヘッドセットのスリープを解除してください",
        ],
        (Language::Ja, ErrorCode::NotPico) => {
            &["PICO 4 Ultra（A9210）が接続されていることを確認してください"]
        }
        (Language::Ja, ErrorCode::WifiDisabled) => &[
            "ヘッドセットの設定でWi-Fiをオンにしてください",
            "PCと同じWi-Fiネットワークに接続してください",
        ],
        (Language::Ja, ErrorCode::WifiNotConnected) => {
            &["ヘッドセットをPCと同じWi-Fiネットワークに接続してください"]
        }
        (Language::Ja, ErrorCode::TcpipFailed) => {
            &["ヘッドセットを再起動してから再試行してください"]
        }
        (Language::Ja, ErrorCode::WirelessRefused) => &[
            "ヘッドセットを再起動すると無線ADBが無効になります。USBケーブルで再セットアップしてください",
        ],
        (Language::Ja, ErrorCode::WirelessUnreachable) => &[
            "ヘッドセットの電源が入っており、スリープしていないか確認してください",
            "PCとヘッドセットが同じWi-Fi（同一サブネット）にあるか確認してください",
            "IPアドレスが変更されていないか確認してください",
        ],
        (Language::Ja, ErrorCode::WirelessUnauthorized) => {
            &["USBケーブルを接続して再度無線セットアップを実行してください"]
        }
        (Language::Ja, ErrorCode::WirelessNotFound) => &[
            "IPアドレスが正しいか確認してください",
            "USBケーブルで無線セットアップをやり直してください",
        ],
        (Language::Ja, ErrorCode::AdbUnavailable) => {
            &["他のADBプロセスが競合していないか確認し、アプリを再起動してください"]
        }
        (Language::Ja, ErrorCode::Timeout) => {
            &["ネットワーク状態やケーブル接続を確認して再試行してください"]
        }
        (Language::Ja, ErrorCode::AlreadyRunning) => &["停止してから再試行してください"],
        (Language::Ja, ErrorCode::InvalidIp) => {
            &["正しいIPv4アドレスを入力してください（例: 192.168.1.20）"]
        }
        (Language::Ja, ErrorCode::MultiplePico) => &["対象外のデバイスを切断してください"],
        (Language::Ja, ErrorCode::DeviceLost) => &["ヘッドセットの接続状態を確認してください"],
        (Language::Ja, ErrorCode::Unknown) => &["接続を確認して再試行してください"],
    }
}

pub fn format_error_log(err: &AppError, lang: Language, with_detail: bool) -> String {
    let title = error_title(lang, err.code);
    if with_detail
        && let Some(ref detail) = err.detail
        && !detail.is_empty()
    {
        return format!("{title} ({detail})");
    }
    title.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_ERROR_CODES: [ErrorCode; 18] = [
        ErrorCode::AdbUnavailable,
        ErrorCode::Timeout,
        ErrorCode::AlreadyRunning,
        ErrorCode::InvalidIp,
        ErrorCode::MultiplePico,
        ErrorCode::UsbNotFound,
        ErrorCode::UsbUnauthorized,
        ErrorCode::UsbOffline,
        ErrorCode::NotPico,
        ErrorCode::WifiDisabled,
        ErrorCode::WifiNotConnected,
        ErrorCode::TcpipFailed,
        ErrorCode::WirelessRefused,
        ErrorCode::WirelessUnreachable,
        ErrorCode::WirelessUnauthorized,
        ErrorCode::WirelessNotFound,
        ErrorCode::DeviceLost,
        ErrorCode::Unknown,
    ];

    #[test]
    fn covers_all_error_codes_in_japanese_and_english() {
        for lang in [Language::Ja, Language::En] {
            for code in ALL_ERROR_CODES {
                let title = error_title(lang, code);
                assert!(!title.is_empty());
                let hints = error_hints(lang, code);
                assert!(!hints.is_empty());
                assert!(hints.iter().all(|h| !h.is_empty()));
            }
        }
    }

    #[test]
    fn formats_error_logs_with_or_without_detail() {
        let err = AppError::with(ErrorCode::UsbNotFound, "check cable");
        assert_eq!(
            format_error_log(&err, Language::Ja, false),
            "ヘッドセットがUSBで見つかりません"
        );
        assert_eq!(
            format_error_log(&err, Language::Ja, true),
            "ヘッドセットがUSBで見つかりません (check cable)"
        );
    }
}
