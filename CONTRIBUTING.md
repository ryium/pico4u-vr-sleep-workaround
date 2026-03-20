# Contributing to pico4u-vr-sleep-workaround

Welcome to the development guide for pico4u-vr-sleep-workaround. This project uses Tauri (Rust) for the backend and Vite + React (TypeScript) for the frontend frontend UI. Follow the instructions below to set up your development environment.

## 🛠️ Prerequisites

To build and develop this project, you will need the following tools installed on your system:

- **[Rust](https://rustup.rs/)**: Used for the Tauri backend.
- **[Node.js (LTS)](https://nodejs.org/)**: Required for the frontend build.
- **[pnpm](https://pnpm.io/)**: Package manager used for the frontend.
- **Android Platform-Tools (ADB)**: The application uses `adb` to communicate with the Pico headset. You must download the ADB executable and place it at `bin/adb.exe` (on Windows) or configure `tauri.conf.json` external bins appropriately.

## 📦 Project Setup

1. **Clone the repository**:
   ```bash
   git clone https://github.com/ryium/pico4u-vr-sleep-workaround.git
   cd pico4u-vr-sleep-workaround
   ```

2. **Install frontend dependencies**:
   ```bash
   pnpm install
   ```
   *Note: Our root `package.json` delegates `install` to the `frontend` folder (`pnpm -C frontend install`).*

3. **ADB Binary Setup**:
   Download Android SDK Platform-Tools and copy the ADB executable and its notice file into the `bin` directory at the project root:
   - `bin/adb.exe` (or `bin/adb` on macOS/Linux)
   - `bin/adb-NOTICE.txt`

## 🏃‍♂️ Development

The project uses Tauri's CLI for development, which will start both the React frontend (via Vite) and the Rust backend.

Run the development server:
```bash
cargo tauri dev
```
Alternatively, using pnpm:
```bash
pnpm tauri dev
```
*(If you just want to run the frontend separately for UI testing: `pnpm -C frontend dev`)*

## 🏗️ Building

To create a production build and generate installers (NSIS/MSI for Windows):

```bash
cargo tauri build
```
This will compile the frontend into `frontend/dist`, compile the Rust backend, bundle the ADB sidecar, and generate the final installer executables in the `target/release/bundle` directory.

## 📜 Repository Structure

- `src/` - Rust source code for the Tauri application (backend logic, ADB commands).
- `frontend/` - Vite / React / TypeScript web UI and components.
- `bin/` - Required directory for custom external binaries (like `adb`).
- `tauri.conf.json` - Core Tauri configuration and window settings.
