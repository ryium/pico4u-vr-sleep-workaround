# Development

Use stable Rust with the Windows MSVC toolchain (and the Windows 10/11 SDK with `fxc.exe` for release HLSL shader compilation in GPUI). The application is written in pure Rust using [GPUI](https://www.gpui.rs/).

## Install and run (Windows x64)

```powershell
.\scripts\setup_adb.ps1  # only if bin/ does not contain ADB and its DLLs
cargo run
```

## Checks and builds

```powershell
cargo fmt --check
cargo test --locked
cargo clippy --locked -- -D warnings
cargo build --release --locked
```

Release distributions must include `pico4u-vr-sleep-workaround.exe`, `adb.exe` (or `bin/adb.exe`), `AdbWinApi.dll`, `AdbWinUsbApi.dll`, and the licenses in `LICENSES/`.

## Repository

- `src/`: Rust application, GPUI user interface, i18n translations, and ADB protocol client.
- `scripts/`: ADB setup and icon utilities.
