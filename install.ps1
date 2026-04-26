# FeverCode installer for Windows
# Usage: irm https://github.com/FeverDream-dev/FeverCode/releases/latest/download/fever-installer.ps1 | iex
#
# Environment variables:
#   $env:FEVER_INSTALL_DIR  — override install directory (default: $HOME\.local\bin)
#   $env:FEVER_VERSION      — specific version to install (default: latest)

$ErrorActionPreference = "Stop"

$Repo = "FeverDream-dev/FeverCode"
$Version = if ($env:FEVER_VERSION) { $env:FEVER_VERSION } else { "latest" }
$BinDir = if ($env:FEVER_INSTALL_DIR) { $env:FEVER_INSTALL_DIR } else { "$HOME\.local\bin" }

Write-Host "FeverCode installer"
Write-Host "Version: $Version"
Write-Host "Target: $BinDir"

$Arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
$Target = switch ($Arch) {
    "X64" { "x86_64-pc-windows-msvc" }
    "Arm64" { "aarch64-pc-windows-msvc" }
    default { Write-Error "Unsupported architecture: $Arch"; exit 1 }
}

$Asset = "fevercode-$Target.zip"
if ($Version -eq "latest") {
    $Url = "https://github.com/$Repo/releases/latest/download/$Asset"
} else {
    $Url = "https://github.com/$Repo/releases/download/$Version/$Asset"
}

Write-Host "Downloading $Url"

try {
    Invoke-WebRequest -Uri $Url -OutFile "$env:TEMP\$Asset" -UseBasicParsing
} catch {
    Write-Error "Download failed. No release available yet?"
    Write-Error "Install from source instead:"
    Write-Error "  cargo install --git https://github.com/$Repo fever"
    exit 1
}

New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
Expand-Archive -Path "$env:TEMP\$Asset" -DestinationPath $BinDir -Force

$feverPath = Join-Path $BinDir "fever.exe"
$fevercodePath = Join-Path $BinDir "fevercode.exe"

if ((Test-Path $feverPath) -and -not (Test-Path $fevercodePath)) {
    Copy-Item $feverPath $fevercodePath
}

$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$BinDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$BinDir;$UserPath", "User")
    Write-Host "Added $BinDir to user PATH"
    Write-Host "Open a new terminal for PATH changes to take effect"
}

Write-Host ""
Write-Host "Installed fever to $BinDir"
Write-Host "Run: fever doctor"
