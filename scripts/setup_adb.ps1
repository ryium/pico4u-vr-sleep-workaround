$ErrorActionPreference = "Stop"

$binDir = Join-Path $PSScriptRoot "..\bin"
if (!(Test-Path $binDir)) {
    New-Item -ItemType Directory -Path $binDir | Out-Null
}

$zipUrl = "https://dl.google.com/android/repository/platform-tools-latest-windows.zip"
$zipPath = Join-Path $env:TEMP "platform-tools.zip"
$extractPath = Join-Path $env:TEMP "platform-tools-extract"

Write-Host "Downloading ADB..."
Invoke-WebRequest -Uri $zipUrl -OutFile $zipPath

Write-Host "Extracting..."
Expand-Archive -Path $zipPath -DestinationPath $extractPath -Force

$toolsDir = Join-Path $extractPath "platform-tools"

# Target file name for Tauri sidecar (Windows x64)
$targetName = "adb-x86_64-pc-windows-msvc.exe"

Write-Host "Copying files to $binDir..."
Copy-Item (Join-Path $toolsDir "adb.exe") (Join-Path $binDir $targetName) -Force
Copy-Item (Join-Path $toolsDir "AdbWinApi.dll") (Join-Path $binDir "AdbWinApi.dll") -Force
Copy-Item (Join-Path $toolsDir "AdbWinUsbApi.dll") (Join-Path $binDir "AdbWinUsbApi.dll") -Force
Copy-Item (Join-Path $toolsDir "NOTICE.txt") (Join-Path $binDir "adb-NOTICE.txt") -Force

Write-Host "Cleaning up..."
Remove-Item $zipPath -Force
Remove-Item $extractPath -Recurse -Force

Write-Host "ADB setup complete."
