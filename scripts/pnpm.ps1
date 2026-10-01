# Use the native Windows binary; older npm/mise shims try to run it with Node.
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path $PSScriptRoot -Parent
$version = (Get-Content (Join-Path $projectRoot 'package.json') -Raw | ConvertFrom-Json).packageManager.Split('@')[1]
$toolsDir = Join-Path $projectRoot '.tools'
$binaryDir = Join-Path $toolsDir 'node_modules/@pnpm/exe.win32-x64'
$binary = Join-Path $binaryDir 'pnpm.exe'
if (!(Test-Path -LiteralPath $binary) -or (Get-Content (Join-Path $binaryDir 'package.json') -Raw | ConvertFrom-Json).version -ne $version) {
    npm install --prefix $toolsDir --no-audit --no-fund "pnpm@$version"
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
$previousPath = $env:PATH
try {
    $env:PATH = "$binaryDir;$previousPath"
    & $binary @args
    $result = $LASTEXITCODE
} finally {
    $env:PATH = $previousPath
}
exit $result
