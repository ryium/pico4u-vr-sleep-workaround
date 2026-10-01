# Development

Use Node.js 24 LTS, stable Rust with the Windows MSVC toolchain, and pnpm **12.3.4** (pinned in both package manifests). Install the Visual Studio C++ build tools and WebView2 required by Tauri.

## Install and run (Windows x64)

```powershell
.\scripts\pnpm.ps1 install --frozen-lockfile
.\scripts\setup_adb.ps1  # only if bin/ does not contain ADB and its DLLs
.\scripts\pnpm.ps1 dev
```

The wrapper bootstraps the pinned pnpm into ignored `.tools/` and puts its native executable on PATH for child processes. This also fixes npm/mise shims that try to interpret pnpm 12's native executable with Node. No global installation changes are needed. With a working pnpm installation, the equivalent commands are `pnpm install --frozen-lockfile` and `pnpm dev`.

The root and `frontend/` intentionally have separate manifests and lockfiles. Root installation installs the frontend with its frozen lockfile. CI reads `packageManager` and uses Node 24. To update dependencies, update the frontend first, then the root.

## Checks and builds

```powershell
.\scripts\pnpm.ps1 -C frontend lint
.\scripts\pnpm.ps1 -C frontend format:check
.\scripts\pnpm.ps1 -C frontend test
.\scripts\pnpm.ps1 -C frontend build
cargo fmt --check
cargo test --locked
cargo clippy --locked -- -D warnings
.\scripts\pnpm.ps1 build --no-sign
```

`pnpm -C frontend dev` previews the UI in a browser. Device operations require the Tauri window. Windows builds produce NSIS/MSI installers in `target/release/bundle/`. The bundle must include `adb.exe`, `AdbWinApi.dll`, `AdbWinUsbApi.dll`, and the licenses. Source sidecar: `bin/adb-x86_64-pc-windows-msvc.exe`.

## Repository

- `src/`: Rust backend and ADB protocol client.
- `frontend/`: React UI.
- `scripts/`: package-manager bootstrap, ADB setup, version synchronization.
- `tauri.conf.json`: fixed 400 × 700 window and bundling configuration.

App version changes use `pnpm version patch` (or an explicit version), which synchronizes the frontend, Rust and Tauri versions. Dependency/UI refreshes do not bump the application version.
