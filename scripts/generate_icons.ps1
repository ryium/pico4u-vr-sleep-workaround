Add-Type -AssemblyName System.Drawing

$iconsDir = Join-Path $PSScriptRoot "..\icons"
if (!(Test-Path $iconsDir)) {
    New-Item -ItemType Directory -Path $iconsDir | Out-Null
}

function Create-Icon($path, $size) {
    $bmp = New-Object System.Drawing.Bitmap $size, $size
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.Clear([System.Drawing.Color]::Blue)
    $g.Dispose()
    
    $icon = [System.Drawing.Icon]::FromHandle($bmp.GetHicon())
    $fileStream = New-Object System.IO.FileStream $path, 'Create'
    $icon.Save($fileStream)
    $fileStream.Close()
    $bmp.Dispose()
    $icon.Dispose()
}

function Create-Png($path, $size) {
    $bmp = New-Object System.Drawing.Bitmap $size, $size
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.Clear([System.Drawing.Color]::Blue)
    $g.Dispose()
    
    $bmp.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
}

Create-Icon (Join-Path $iconsDir "icon.ico") 256
Create-Png (Join-Path $iconsDir "icon.png") 512
Create-Png (Join-Path $iconsDir "32x32.png") 32
Create-Png (Join-Path $iconsDir "128x128.png") 128

Write-Host "Generated placeholder icons."
