# ============================================================
# HyperYap Installer
# One-command setup for HyperYap (speech-to-text) + hotkeys
# ============================================================
#Requires -RunAsAdministrator

param(
    [switch]$SkipModel,
    [switch]$SkipAHK,
    [switch]$SkipAutostart
)

$ErrorActionPreference = "Stop"

$repo = "avalonreset/hyperyap"
$modelUrl = "https://github.com/Kieirra/murmure-model/releases/download/1.0.0/parakeet-tdt-0.6b-v3-int8.zip"
$ahkInstallerUrl = "https://www.autohotkey.com/download/ahk-v2.exe"
$appDataDir = "$env:APPDATA\com.al1x-ai.hyperyap"
$startupDir = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Startup"
$installDir = "$env:LOCALAPPDATA\Programs\HyperYap"

Write-Host ""
Write-Host "  __ __                     __  __          " -ForegroundColor Cyan
Write-Host " / // /_ _____  ___ ____   / / / /__ ____  " -ForegroundColor Cyan
Write-Host "/ _  / // / _ \/ -_) __/  / /_/ / _ '/ _ \ " -ForegroundColor Cyan
Write-Host "/_//_/\_, / .__/\__/_/     \____/\_,_/ .__/ " -ForegroundColor Cyan
Write-Host "     /___/_/                        /_/     " -ForegroundColor Cyan
Write-Host ""
Write-Host "  Voice-to-text that just works." -ForegroundColor DarkGray
Write-Host ""

# -----------------------------------------------------------
# 1. Download latest HyperYap release
# -----------------------------------------------------------
Write-Host "[1/5] Downloading latest HyperYap release..." -ForegroundColor Yellow

$releaseApi = "https://api.github.com/repos/$repo/releases/latest"
try {
    $release = Invoke-RestMethod -Uri $releaseApi -Headers @{ "User-Agent" = "HyperYap-Installer" }
    $msiAsset = $release.assets | Where-Object { $_.name -match "\.(msi|exe)$" -and $_.name -match "x64" } | Select-Object -First 1
    if (-not $msiAsset) {
        $msiAsset = $release.assets | Where-Object { $_.name -match "\.(msi|exe)$" } | Select-Object -First 1
    }
    if ($msiAsset) {
        $installerPath = "$env:TEMP\hyperyap-installer$([System.IO.Path]::GetExtension($msiAsset.name))"
        Write-Host "  Downloading $($msiAsset.name)..." -ForegroundColor DarkGray
        Invoke-WebRequest -Uri $msiAsset.browser_download_url -OutFile $installerPath -UseBasicParsing
        Write-Host "  Running installer..." -ForegroundColor DarkGray
        if ($installerPath -match "\.msi$") {
            Start-Process msiexec.exe -ArgumentList "/i `"$installerPath`" /quiet /norestart" -Wait
        } else {
            Start-Process $installerPath -ArgumentList "/S" -Wait
        }
        Write-Host "  HyperYap installed." -ForegroundColor Green
    } else {
        Write-Host "  No installer found in latest release. Skipping app install." -ForegroundColor DarkYellow
        Write-Host "  You may need to build from source or install manually." -ForegroundColor DarkGray
    }
} catch {
    Write-Host "  Could not fetch latest release. Is the repo public?" -ForegroundColor DarkYellow
    Write-Host "  Error: $_" -ForegroundColor DarkGray
    Write-Host "  Continuing with config setup..." -ForegroundColor DarkGray
}

# -----------------------------------------------------------
# 2. Download Parakeet model
# -----------------------------------------------------------
if (-not $SkipModel) {
    Write-Host "[2/5] Downloading speech recognition model (~440MB)..." -ForegroundColor Yellow

    # Find where HyperYap stores resources
    $resourceDirs = @(
        "$installDir\resources",
        "$env:LOCALAPPDATA\com.al1x-ai.hyperyap\resources",
        "$env:APPDATA\com.al1x-ai.hyperyap\resources"
    )

    $targetResourceDir = $resourceDirs | Where-Object { Test-Path (Split-Path $_ -Parent) } | Select-Object -First 1
    if (-not $targetResourceDir) {
        $targetResourceDir = $resourceDirs[0]
    }

    $modelDir = "$targetResourceDir\parakeet-tdt-0.6b-v3-int8"

    if (Test-Path "$modelDir\encoder-model.int8.onnx") {
        Write-Host "  Model already exists. Skipping." -ForegroundColor Green
    } else {
        $modelZip = "$env:TEMP\parakeet-model.zip"
        Write-Host "  Downloading from GitHub..." -ForegroundColor DarkGray
        Invoke-WebRequest -Uri $modelUrl -OutFile $modelZip -UseBasicParsing
        Write-Host "  Extracting model..." -ForegroundColor DarkGray
        New-Item -ItemType Directory -Path $targetResourceDir -Force | Out-Null
        Expand-Archive -Path $modelZip -DestinationPath $targetResourceDir -Force
        Remove-Item $modelZip -Force
        Write-Host "  Model installed to $targetResourceDir" -ForegroundColor Green
    }
} else {
    Write-Host "[2/5] Skipping model download (--SkipModel)" -ForegroundColor DarkGray
}

# -----------------------------------------------------------
# 3. Install AutoHotkey v2 (if needed)
# -----------------------------------------------------------
if (-not $SkipAHK) {
    Write-Host "[3/5] Checking AutoHotkey v2..." -ForegroundColor Yellow

    $ahkExe = Get-Command "AutoHotkey64.exe" -ErrorAction SilentlyContinue
    if (-not $ahkExe) {
        $ahkExe = Get-Command "AutoHotkey32.exe" -ErrorAction SilentlyContinue
    }
    if (-not $ahkExe) {
        # Check common install locations
        $ahkPaths = @(
            "$env:ProgramFiles\AutoHotkey\v2\AutoHotkey64.exe",
            "$env:ProgramFiles\AutoHotkey\v2\AutoHotkey32.exe",
            "${env:ProgramFiles(x86)}\AutoHotkey\v2\AutoHotkey64.exe"
        )
        $ahkExe = $ahkPaths | Where-Object { Test-Path $_ } | Select-Object -First 1
    }

    if (-not $ahkExe) {
        Write-Host "  AutoHotkey v2 not found. Installing..." -ForegroundColor DarkGray
        $ahkInstaller = "$env:TEMP\ahk-v2-setup.exe"
        Invoke-WebRequest -Uri $ahkInstallerUrl -OutFile $ahkInstaller -UseBasicParsing
        Start-Process $ahkInstaller -ArgumentList "/silent" -Wait
        Remove-Item $ahkInstaller -Force
        Write-Host "  AutoHotkey v2 installed." -ForegroundColor Green
    } else {
        Write-Host "  AutoHotkey v2 already installed." -ForegroundColor Green
    }
} else {
    Write-Host "[3/5] Skipping AutoHotkey install (--SkipAHK)" -ForegroundColor DarkGray
}

# -----------------------------------------------------------
# 4. Deploy preset configs
# -----------------------------------------------------------
Write-Host "[4/5] Deploying HyperYap configs..." -ForegroundColor Yellow

# Find presets (either from repo clone or bundled with installer)
$scriptRoot = $PSScriptRoot
$presetsDir = "$scriptRoot\presets"

if (-not (Test-Path $presetsDir)) {
    # Try downloading presets from repo
    Write-Host "  Downloading presets from GitHub..." -ForegroundColor DarkGray
    $presetsDir = "$env:TEMP\hyperyap-presets"
    New-Item -ItemType Directory -Path $presetsDir -Force | Out-Null
    $presetFiles = @("settings.json", "llm_connect.json")
    foreach ($f in $presetFiles) {
        Invoke-WebRequest -Uri "https://raw.githubusercontent.com/$repo/main/presets/$f" -OutFile "$presetsDir\$f" -UseBasicParsing
    }
    New-Item -ItemType Directory -Path "$presetsDir\scripts" -Force | Out-Null
    $scriptFiles = @("hyperyap-hotkeys.ahk", "clipboard-image-paste.ps1")
    foreach ($f in $scriptFiles) {
        Invoke-WebRequest -Uri "https://raw.githubusercontent.com/$repo/main/presets/scripts/$f" -OutFile "$presetsDir\scripts\$f" -UseBasicParsing
    }
}

# Copy app configs
New-Item -ItemType Directory -Path $appDataDir -Force | Out-Null

if (-not (Test-Path "$appDataDir\settings.json")) {
    Copy-Item "$presetsDir\settings.json" "$appDataDir\settings.json"
    Write-Host "  Settings deployed." -ForegroundColor Green
} else {
    Write-Host "  Settings already exist. Skipping (won't overwrite)." -ForegroundColor DarkYellow
}

if (-not (Test-Path "$appDataDir\llm_connect.json")) {
    Copy-Item "$presetsDir\llm_connect.json" "$appDataDir\llm_connect.json"
    Write-Host "  LLM config deployed." -ForegroundColor Green
} else {
    Write-Host "  LLM config already exists. Skipping." -ForegroundColor DarkYellow
}

# Copy AHK scripts to a stable location
$scriptsInstallDir = "$env:LOCALAPPDATA\HyperYap\scripts"
New-Item -ItemType Directory -Path $scriptsInstallDir -Force | Out-Null
Copy-Item "$presetsDir\scripts\*" "$scriptsInstallDir\" -Force
Write-Host "  Hotkey scripts deployed to $scriptsInstallDir" -ForegroundColor Green

# -----------------------------------------------------------
# 5. Set up auto-start
# -----------------------------------------------------------
if (-not $SkipAutostart) {
    Write-Host "[5/5] Configuring auto-start..." -ForegroundColor Yellow

    # AHK hotkeys → Startup folder
    $ahkTarget = "$scriptsInstallDir\hyperyap-hotkeys.ahk"
    $startupLink = "$startupDir\hyperyap-hotkeys.lnk"
    if (-not (Test-Path $startupLink)) {
        $shell = New-Object -ComObject WScript.Shell
        $shortcut = $shell.CreateShortcut($startupLink)
        $shortcut.TargetPath = $ahkTarget
        $shortcut.WorkingDirectory = $scriptsInstallDir
        $shortcut.Description = "HyperYap Hotkeys"
        $shortcut.Save()
        Write-Host "  Hotkeys auto-start enabled." -ForegroundColor Green
    } else {
        Write-Host "  Hotkeys auto-start already configured." -ForegroundColor Green
    }

    # HyperYap app auto-start (via Tauri's built-in autostart if available,
    # otherwise create a startup shortcut)
    $appExePaths = @(
        "$installDir\hyperyap.exe",
        "$installDir\HyperYap.exe",
        "$env:ProgramFiles\HyperYap\hyperyap.exe"
    )
    $appExe = $appExePaths | Where-Object { Test-Path $_ } | Select-Object -First 1

    if ($appExe) {
        $appStartupLink = "$startupDir\HyperYap.lnk"
        if (-not (Test-Path $appStartupLink)) {
            $shell = New-Object -ComObject WScript.Shell
            $shortcut = $shell.CreateShortcut($appStartupLink)
            $shortcut.TargetPath = $appExe
            $shortcut.Description = "HyperYap - Voice to Text"
            $shortcut.Save()
            Write-Host "  HyperYap auto-start enabled." -ForegroundColor Green
        } else {
            Write-Host "  HyperYap auto-start already configured." -ForegroundColor Green
        }
    } else {
        Write-Host "  HyperYap executable not found. Auto-start will be handled by the app's built-in setting." -ForegroundColor DarkYellow
    }
} else {
    Write-Host "[5/5] Skipping auto-start (--SkipAutostart)" -ForegroundColor DarkGray
}

# -----------------------------------------------------------
# Launch
# -----------------------------------------------------------
Write-Host ""
Write-Host "  HyperYap is ready!" -ForegroundColor Green
Write-Host ""
Write-Host "  - Press F13, CapsLock, or Mouse Back to record" -ForegroundColor DarkGray
Write-Host "  - Press again to stop and auto-paste transcription" -ForegroundColor DarkGray
Write-Host "  - Mouse Forward = Enter" -ForegroundColor DarkGray
Write-Host ""

# Start AHK hotkeys now
$ahkScript = "$scriptsInstallDir\hyperyap-hotkeys.ahk"
if (Test-Path $ahkScript) {
    Write-Host "  Starting hotkeys..." -ForegroundColor DarkGray
    Start-Process $ahkScript
}

# Start HyperYap app now
if ($appExe) {
    Write-Host "  Launching HyperYap..." -ForegroundColor DarkGray
    Start-Process $appExe
}

Write-Host ""
Write-Host "  Done. Enjoy yapping." -ForegroundColor Cyan
Write-Host ""
