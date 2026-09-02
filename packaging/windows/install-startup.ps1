# Installs NV-Overlay into Windows User Startup folder

$binName = "nv-overlay-slint.exe"
$startupFolder = [Environment]::GetFolderPath("Startup")
$sourcePath = Join-Path $PSScriptRoot "..\..\target\release\$binName"

if (-not (Test-Path $sourcePath)) {
    $sourcePath = Join-Path $PSScriptRoot "$binName"
}

if (-not (Test-Path $sourcePath)) {
    Write-Error "Could not find $binName in release folder or current folder."
    exit 1
}

$targetBinDir = Join-Path $env:LOCALAPPDATA "Programs\NV-Overlay"
New-Item -ItemType Directory -Force -Path $targetBinDir | Out-Null
$targetBinPath = Join-Path $targetBinDir $binName
Copy-Item -Force $sourcePath $targetBinPath

$wscript = New-Object -ComObject WScript.Shell
$shortcut = $wscript.CreateShortcut((Join-Path $startupFolder "NV-Overlay.lnk"))
$shortcut.TargetPath = $targetBinPath
$shortcut.Description = "NV-Overlay Performance HUD"
$shortcut.WorkingDirectory = $targetBinDir
$shortcut.Save()

Write-Host "NV-Overlay installed to: $targetBinPath"
Write-Host "Startup shortcut created in: $startupFolder"
Write-Host "NV-Overlay will start automatically upon user login."
