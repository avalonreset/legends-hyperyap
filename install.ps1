# ============================================================
# HyperYap Installer
# One-command setup: voice-to-text + terminal + hotkeys
# Install everything. No options. That's the point.
# ============================================================
#Requires -RunAsAdministrator

param(
    [switch]$KeepConfig
)

$ErrorActionPreference = "Stop"

$repo = "avalonreset/hyperyap"
$modelUrl = "https://github.com/Kieirra/murmure-model/releases/download/1.0.0/parakeet-tdt-0.6b-v3-int8.zip"
$ahkInstallerUrl = "https://www.autohotkey.com/download/ahk-v2.exe"
$appDataDir = "$env:APPDATA\com.avalonreset.hyperyap"
$startupDir = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Startup"
$installDir = "$env:LOCALAPPDATA\Programs\HyperYap"
$btRepo = "avalonreset/benjaminterm"
$scriptsInstallDir = "$env:LOCALAPPDATA\HyperYap\scripts"

Write-Host ""
Write-Host "  __ __                     __  __          " -ForegroundColor Cyan
Write-Host " / // /_ _____  ___ ____   / / / /__ ____  " -ForegroundColor Cyan
Write-Host "/ _  / // / _ \/ -_) __/  / /_/ / _ '/ _ \ " -ForegroundColor Cyan
Write-Host "/_//_/\_, / .__/\__/_/     \____/\_,_/ .__/ " -ForegroundColor Cyan
Write-Host "     /___/_/                        /_/     " -ForegroundColor Cyan
Write-Host ""
Write-Host "  The complete vibe coding system." -ForegroundColor DarkGray
Write-Host "  Voice-to-text + terminal + hotkeys. Windows only." -ForegroundColor DarkGray
Write-Host ""

Write-Host "  This will:" -ForegroundColor White
Write-Host "    - Uninstall MURmure (if installed)" -ForegroundColor DarkGray
Write-Host "    - Install/update BenjaminTerm terminal" -ForegroundColor DarkGray
Write-Host "    - Install AutoHotkey v2 (if not present)" -ForegroundColor DarkGray
Write-Host "    - Download NVIDIA Parakeet speech model (~440MB)" -ForegroundColor DarkGray
Write-Host "    - Deploy HyperYap configs (overwrites existing)" -ForegroundColor DarkGray
Write-Host "    - Set everything to auto-start on boot" -ForegroundColor DarkGray
Write-Host ""

if ($KeepConfig) {
    Write-Host "  --KeepConfig: existing configs will be preserved." -ForegroundColor DarkYellow
    Write-Host ""
}

if ([Environment]::UserInteractive) {
    Read-Host "  Press Enter to continue (Ctrl+C to cancel)"
    Write-Host ""
}

# -----------------------------------------------------------
# 0. Remove old MURmure (if present)
# -----------------------------------------------------------
$murmureUninstall = Get-ItemProperty "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*",
    "HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*",
    "HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*" -ErrorAction SilentlyContinue |
    Where-Object { $_.DisplayName -match "murmure" -or $_.DisplayName -match "Murmure" } |
    Select-Object -First 1

if ($murmureUninstall) {
    Write-Host "  Found old MURmure installation. Removing..." -ForegroundColor Yellow
    Get-Process -Name "murmure" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
    $uninstallStr = $murmureUninstall.UninstallString
    if ($uninstallStr) {
        if ($uninstallStr -match "msiexec") {
            $productCode = $murmureUninstall.PSChildName
            Start-Process msiexec.exe -ArgumentList "/x $productCode /quiet /norestart" -Wait -ErrorAction SilentlyContinue
        } else {
            Start-Process cmd.exe -ArgumentList "/c `"$uninstallStr`" /S" -Wait -ErrorAction SilentlyContinue
        }
    }
    Write-Host "  MURmure uninstalled." -ForegroundColor Green
} else {
    Get-Process -Name "murmure" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
}

# Remove old MURmure startup entries
@("$startupDir\murmure.lnk", "$startupDir\Murmure.lnk",
  "$startupDir\murmure-hotkeys.ahk", "$startupDir\murmure-hotkeys.lnk") | ForEach-Object {
    if (Test-Path $_) {
        Remove-Item $_ -Force -ErrorAction SilentlyContinue
        Write-Host "  Removed old startup entry: $(Split-Path $_ -Leaf)" -ForegroundColor DarkGray
    }
}

# -----------------------------------------------------------
# 1. Install HyperYap voice engine
# -----------------------------------------------------------
Write-Host "[1/5] Installing HyperYap voice engine..." -ForegroundColor Yellow

# Kill any running HyperYap before installing (NSIS upgrades in-place, no uninstall needed)
Get-Process -Name "hyperyap", "HyperYap" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 1

try {
    $releaseApi = "https://api.github.com/repos/$repo/releases/latest"
    $release = Invoke-RestMethod -Uri $releaseApi -Headers @{ "User-Agent" = "HyperYap-Installer" }
    # Always use the NSIS exe installer (not MSI). NSIS handles upgrades and running processes properly.
    $setupAsset = $release.assets | Where-Object { $_.name -match "x64-setup\.exe$" } | Select-Object -First 1
    if (-not $setupAsset) {
        $setupAsset = $release.assets | Where-Object { $_.name -match "\.(exe|msi)$" -and $_.name -ne "HyperYap-Setup.exe" } | Select-Object -First 1
    }
    if ($setupAsset) {
        $installerPath = "$env:TEMP\hyperyap-installer$([System.IO.Path]::GetExtension($setupAsset.name))"
        Write-Host "  Downloading $($setupAsset.name)..." -ForegroundColor DarkGray
        Invoke-WebRequest -Uri $setupAsset.browser_download_url -OutFile $installerPath -UseBasicParsing
        Write-Host "  Running installer..." -ForegroundColor DarkGray
        if ($installerPath -match "\.msi$") {
            Start-Process msiexec.exe -ArgumentList "/i `"$installerPath`" /quiet /norestart" -Wait
        } else {
            Start-Process $installerPath -ArgumentList "/S" -Wait
        }
        Remove-Item $installerPath -Force -ErrorAction SilentlyContinue
        Write-Host "  HyperYap installed." -ForegroundColor Green
    } else {
        Write-Host "  No release found yet. Build from source with: pnpm tauri build" -ForegroundColor DarkYellow
    }
} catch {
    Write-Host "  No release available yet. Continuing with setup..." -ForegroundColor DarkYellow
}

# -----------------------------------------------------------
# 2. Install BenjaminTerm
# -----------------------------------------------------------
Write-Host "[2/5] Installing BenjaminTerm terminal..." -ForegroundColor Yellow

try {
    $btHeaders = @{ "User-Agent" = "HyperYap-Installer" }
    $btLatestReleaseApi = "https://api.github.com/repos/$btRepo/releases/latest"
    try {
        $btLatestRelease = Invoke-RestMethod -Uri $btLatestReleaseApi -Headers $btHeaders
    } catch {
        $btReleaseApi = "https://api.github.com/repos/$btRepo/releases"
        $btLatestRelease = @(Invoke-RestMethod -Uri $btReleaseApi -Headers $btHeaders) |
            Where-Object { -not $_.draft -and -not $_.prerelease } |
            Select-Object -First 1
    }

    if (-not $btLatestRelease) {
        throw "No stable BenjaminTerm release found."
    }

    Write-Host "  Latest BenjaminTerm release: $($btLatestRelease.tag_name)" -ForegroundColor DarkGray

    $btSetupAsset = @($btLatestRelease.assets) |
        Where-Object { $_.name -match "(?i)setup\.exe$" } |
        Select-Object -First 1

    if ($btSetupAsset) {
        $btInstallerPath = "$env:TEMP\BenjaminTerm-Setup.exe"
        Write-Host "  Downloading $($btSetupAsset.name)..." -ForegroundColor DarkGray
        Invoke-WebRequest -Uri $btSetupAsset.browser_download_url -OutFile $btInstallerPath -UseBasicParsing
        Write-Host "  Running installer..." -ForegroundColor DarkGray
        Start-Process $btInstallerPath -ArgumentList "/S" -Wait
        Remove-Item $btInstallerPath -Force -ErrorAction SilentlyContinue
        Write-Host "  BenjaminTerm installed." -ForegroundColor Green
    } else {
        # Fall back to portable zip
        $btZipAsset = @($btLatestRelease.assets) |
            Where-Object { $_.name -match "(?i)windows.*\.zip$" -and $_.name -notmatch "(?i)sha256" } |
            Select-Object -First 1
        if ($btZipAsset) {
            $btZipPath = "$env:TEMP\BenjaminTerm.zip"
            $btInstallDir = "$env:LOCALAPPDATA\Programs\BenjaminTerm"
            Write-Host "  Downloading $($btZipAsset.name) (portable)..." -ForegroundColor DarkGray
            Invoke-WebRequest -Uri $btZipAsset.browser_download_url -OutFile $btZipPath -UseBasicParsing
            New-Item -ItemType Directory -Path $btInstallDir -Force | Out-Null
            Expand-Archive -Path $btZipPath -DestinationPath $btInstallDir -Force
            Remove-Item $btZipPath -Force
            Write-Host "  BenjaminTerm installed (portable)." -ForegroundColor Green
        } else {
            Write-Host "  No installer found. Get it at: https://github.com/$btRepo/releases" -ForegroundColor DarkYellow
        }
    }
} catch {
    Write-Host "  Could not fetch BenjaminTerm. Get it at: https://github.com/$btRepo/releases" -ForegroundColor DarkYellow
}

# -----------------------------------------------------------
# 3. Download speech recognition model
# -----------------------------------------------------------
Write-Host "[3/5] Setting up speech recognition model (~440MB)..." -ForegroundColor Yellow

$resourceDirs = @(
    "$installDir\resources",
    "$env:LOCALAPPDATA\com.avalonreset.hyperyap\resources",
    "$env:APPDATA\com.avalonreset.hyperyap\resources"
)
$targetResourceDir = $resourceDirs | Where-Object { Test-Path (Split-Path $_ -Parent) } | Select-Object -First 1
if (-not $targetResourceDir) { $targetResourceDir = $resourceDirs[0] }
$modelDir = "$targetResourceDir\parakeet-tdt-0.6b-v3-int8"

if (Test-Path "$modelDir\encoder-model.int8.onnx") {
    Write-Host "  Model already downloaded." -ForegroundColor Green
} else {
    $modelZip = "$env:TEMP\parakeet-model.zip"
    $maxRetries = 30
    $downloaded = $false

    for ($attempt = 1; $attempt -le $maxRetries; $attempt++) {
        Write-Host "  Downloading from GitHub (attempt $attempt/$maxRetries)..." -ForegroundColor DarkGray
        # Use curl.exe with resume support — handles flaky connections gracefully
        & curl.exe -L -C - -o $modelZip --retry 3 --retry-delay 2 --connect-timeout 30 $modelUrl 2>&1 | Write-Host
        if ($LASTEXITCODE -eq 0) {
            $downloaded = $true
            break
        }
        Write-Host "  Download interrupted, retrying in 3 seconds..." -ForegroundColor Yellow
        Start-Sleep -Seconds 3
    }

    if (-not $downloaded) {
        throw "Failed to download speech model after $maxRetries attempts. Check your internet connection."
    }

    Write-Host "  Extracting model..." -ForegroundColor DarkGray
    New-Item -ItemType Directory -Path $targetResourceDir -Force | Out-Null
    Expand-Archive -Path $modelZip -DestinationPath $targetResourceDir -Force

    # Verify the model was extracted correctly
    if (-not (Test-Path "$modelDir\encoder-model.int8.onnx")) {
        Remove-Item $modelZip -Force -ErrorAction SilentlyContinue
        throw "Model extraction failed — encoder-model.int8.onnx not found. The download may have been corrupted."
    }

    Remove-Item $modelZip -Force
    Write-Host "  Model installed." -ForegroundColor Green
}

# -----------------------------------------------------------
# 4. Install AutoHotkey v2 + deploy hotkey scripts
# -----------------------------------------------------------
Write-Host "[4/5] Setting up hotkeys..." -ForegroundColor Yellow

$ahkExe = Get-Command "AutoHotkey64.exe" -ErrorAction SilentlyContinue
if (-not $ahkExe) { $ahkExe = Get-Command "AutoHotkey32.exe" -ErrorAction SilentlyContinue }
if (-not $ahkExe) {
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
    Write-Host "  AutoHotkey v2 found." -ForegroundColor Green
}

# Deploy hotkey scripts
$scriptRoot = $PSScriptRoot
$presetsDir = "$scriptRoot\presets"

if (-not (Test-Path $presetsDir)) {
    Write-Host "  Downloading presets from GitHub..." -ForegroundColor DarkGray
    $presetsDir = "$env:TEMP\hyperyap-presets"
    New-Item -ItemType Directory -Path $presetsDir -Force | Out-Null
    @("settings.json", "llm_connect.json") | ForEach-Object {
        Invoke-WebRequest -Uri "https://raw.githubusercontent.com/$repo/main/presets/$_" -OutFile "$presetsDir\$_" -UseBasicParsing
    }
    New-Item -ItemType Directory -Path "$presetsDir\scripts" -Force | Out-Null
    @("hyperyap-hotkeys.ahk", "clipboard-image-paste.ps1") | ForEach-Object {
        Invoke-WebRequest -Uri "https://raw.githubusercontent.com/$repo/main/presets/scripts/$_" -OutFile "$presetsDir\scripts\$_" -UseBasicParsing
    }
}

New-Item -ItemType Directory -Path $scriptsInstallDir -Force | Out-Null
Copy-Item "$presetsDir\scripts\*" "$scriptsInstallDir\" -Force
Write-Host "  Hotkey scripts deployed." -ForegroundColor Green

# -----------------------------------------------------------
# 5. Deploy configs + auto-start
# -----------------------------------------------------------
Write-Host "[5/5] Deploying configs and auto-start..." -ForegroundColor Yellow

New-Item -ItemType Directory -Path $appDataDir -Force | Out-Null

if (-not $KeepConfig -or -not (Test-Path "$appDataDir\settings.json")) {
    Copy-Item "$presetsDir\settings.json" "$appDataDir\settings.json" -Force
    Write-Host "  Settings deployed." -ForegroundColor Green
} else {
    Write-Host "  Settings preserved (--KeepConfig)." -ForegroundColor DarkYellow
}

if (-not $KeepConfig -or -not (Test-Path "$appDataDir\llm_connect.json")) {
    Copy-Item "$presetsDir\llm_connect.json" "$appDataDir\llm_connect.json" -Force
    Write-Host "  LLM config deployed." -ForegroundColor Green
} else {
    Write-Host "  LLM config preserved (--KeepConfig)." -ForegroundColor DarkYellow
}

# AHK hotkeys auto-start
$ahkTarget = "$scriptsInstallDir\hyperyap-hotkeys.ahk"
$startupLink = "$startupDir\hyperyap-hotkeys.lnk"
$shell = New-Object -ComObject WScript.Shell
$shortcut = $shell.CreateShortcut($startupLink)
$shortcut.TargetPath = $ahkTarget
$shortcut.WorkingDirectory = $scriptsInstallDir
$shortcut.Description = "HyperYap Hotkeys"
$shortcut.Save()
Write-Host "  Hotkeys auto-start configured." -ForegroundColor Green

# HyperYap app auto-start (handled by app itself on first launch)
Write-Host "  Voice engine auto-start will activate on first launch." -ForegroundColor Green

# -----------------------------------------------------------
# Launch everything
# -----------------------------------------------------------
Write-Host ""
Write-Host "  HyperYap is ready!" -ForegroundColor Green
Write-Host ""
Write-Host "  - Press F13, CapsLock, or Mouse Back to record" -ForegroundColor DarkGray
Write-Host "  - Press again to stop and auto-paste transcription" -ForegroundColor DarkGray
Write-Host "  - Mouse Forward = Enter" -ForegroundColor DarkGray
Write-Host ""

# Start hotkeys
if (Test-Path $ahkTarget) {
    Write-Host "  Starting hotkeys..." -ForegroundColor DarkGray
    Start-Process $ahkTarget
}

# Start HyperYap
$appExePaths = @("$installDir\hyperyap.exe", "$installDir\HyperYap.exe", "$env:ProgramFiles\HyperYap\hyperyap.exe")
$appExe = $appExePaths | Where-Object { Test-Path $_ } | Select-Object -First 1
if ($appExe) {
    Write-Host "  Launching HyperYap..." -ForegroundColor DarkGray
    Start-Process $appExe
}

# Start BenjaminTerm
$btExePaths = @(
    "$env:ProgramFiles\BenjaminTerm\benjaminterm-gui.exe",
    "$env:LOCALAPPDATA\Programs\BenjaminTerm\benjaminterm-gui.exe",
    "${env:ProgramFiles(x86)}\BenjaminTerm\benjaminterm-gui.exe"
)
$btExe = $btExePaths | Where-Object { Test-Path $_ } | Select-Object -First 1
if ($btExe) {
    Write-Host "  Launching BenjaminTerm..." -ForegroundColor DarkGray
    Start-Process $btExe
}

Write-Host ""
Write-Host "  Done. Enjoy yapping." -ForegroundColor Cyan
Write-Host ""
