$ErrorActionPreference = "Stop"
$iconsDir = Join-Path $PSScriptRoot "..\icons"
if (!(Test-Path $iconsDir)) {
    New-Item -ItemType Directory -Path $iconsDir | Out-Null
}

$baseUrl = "https://raw.githubusercontent.com/tauri-apps/tauri/1.x/examples/api/src-tauri/icons"
$files = @("icon.ico", "icon.png", "32x32.png", "128x128.png")

foreach ($file in $files) {
    $url = "$baseUrl/$file"
    $dest = Join-Path $iconsDir $file
    Write-Host "Downloading $file from $url..."
    try {
        Invoke-WebRequest -Uri $url -OutFile $dest
    } catch {
        Write-Host "Failed to download $file"
        exit 1
    }
}
Write-Host "Icons downloaded."