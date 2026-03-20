<h1>
  <img src="./icons/128x128.png" width="72" alt="Icon" valign="middle" />
  pico4u-vr-sleep-workaround
</h1>

pico4u-vr-sleep-workaround is a workaround tool designed to prevent the Pico 4 Ultra VR headset from going to sleep during development or specific use cases. It operates by periodically sending a keep-alive signal via ADB.

[日本語版（Japanese Version）はこちら](.github/README_ja.md)

## 🌟 Features

- **Keep-Alive Signal**: Prevents the headset from entering sleep mode by sending the `keyevent 224` (wake up) command every 3 seconds.
- **Connection Modes**: Supports both Wired (USB) and Wireless (TCP/IP) ADB connections.
- **Device Validation**: Automatically detects the connected device and verifies it is a Pico 4 Ultra (checks for `A9210` in the model name).
- **Auto-Dimming**: Optionally dims the headset's screen brightness to the lowest level (`1`) after a configurable delay to prevent screen burn-in and save power.

## 📸 Screenshot

![UI](./assets/screenshot.png)

## 🚀 Setup & Usage

1. Download the latest release for your platform (Windows `nsis` / `msi` installers are provided).
2. Install and launch the application.
3. Ensure your Pico 4 Ultra is connected via USB or Wi-Fi with **USB Debugging** enabled.
4. Select your connection mode and click **Start Keep Alive**.

## 👨‍💻 For Developers

For information on how to set up the development environment, build the app from source, and contribute to the project, please refer to the [CONTRIBUTING.md](./CONTRIBUTING.md) guide.
