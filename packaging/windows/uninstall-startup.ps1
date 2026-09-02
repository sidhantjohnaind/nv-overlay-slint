# Uninstalls NV-Overlay from Windows User Startup

$binName = "nv-overlay-slint.exe"
$startupFolder = [Environment]::GetFolderPath("Startup")
$shortcutPath = Join-Path $startupFolder "NV-Overlay.lnk"

if (Test-Path $shortcutPath) {
    Remove-Item -Force $shortcutPath
    Write-Host "Removed startup shortcut: $shortcutPath"
}

$targetBinDir = Join-Path $env:LOCALAPPDATA "Programs\NV-Overlay"
if (Test-Path $targetBinDir) {
    Remove-Item -Recurse -Force $targetBinDir
    Write-Host "Removed program files: $targetBinDir"
}

Write-Host "NV-Overlay uninstalled from Windows startup."
