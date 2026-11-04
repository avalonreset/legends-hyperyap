# ============================================================
# legends-hyperyap installer
# One-command setup: voice-to-text + hotkeys
# Install everything. No options. That's the point.
# ============================================================
#Requires -RunAsAdministrator

param(
    [switch]$KeepConfig
)

$ErrorActionPreference = "Stop"

$repo = "avalonreset/legends-hyperyap"
$modelId = "parakeet-tdt-0.6b-v2-smcleod-int8"
$modelBaseUrl = "https://huggingface.co/smcleod/parakeet-tdt-0.6b-v2-int8/resolve/main"
$modelFiles = @(
    "encoder-model.int8.onnx",
    "decoder_joint-model.int8.onnx",
    "nemo128.onnx",
    "vocab.txt",
    "config.json",
    "README.md"
)
$appDataDir = "$env:APPDATA\com.avalonreset.hyperyap"
$startupDir = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Startup"
$installDir = "$env:LOCALAPPDATA\Programs\HyperYap"
$hotkeysExe = "$env:LOCALAPPDATA\HyperYap\hyperyap-hotkeys.exe"

Write-Host ""
Write-Host "  __ __                     __  __          " -ForegroundColor Cyan
Write-Host " / // /_ _____  ___ ____   / / / /__ ____  " -ForegroundColor Cyan
Write-Host "/ _  / // / _ \/ -_) __/  / /_/ / _ '/ _ \ " -ForegroundColor Cyan
Write-Host "/_//_/\_, / .__/\__/_/     \____/\_,_/ .__/ " -ForegroundColor Cyan
Write-Host "     /___/_/                        /_/     " -ForegroundColor Cyan
Write-Host ""
Write-Host "  The complete vibe coding system." -ForegroundColor DarkGray
Write-Host "  Voice-to-text + hotkeys. Windows only." -ForegroundColor DarkGray
Write-Host ""

Write-Host "  This will:" -ForegroundColor White
Write-Host "    - Uninstall MURmure (if installed)" -ForegroundColor DarkGray
Write-Host "    - Install legends-hyperyap and its native Rust hotkey helper" -ForegroundColor DarkGray
Write-Host "    - Download NVIDIA Parakeet v2 English speech model (~665MB)" -ForegroundColor DarkGray
Write-Host "    - Deploy legends-hyperyap configs (overwrites existing)" -ForegroundColor DarkGray
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
@("$startupDir\murmure.lnk", "$startupDir\Murmure.lnk", "$startupDir\murmure-hotkeys.lnk") | ForEach-Object {
    if (Test-Path $_) {
        Remove-Item $_ -Force -ErrorAction SilentlyContinue
        Write-Host "  Removed old startup entry: $(Split-Path $_ -Leaf)" -ForegroundColor DarkGray
    }
}

# -----------------------------------------------------------
# 1. Install legends-hyperyap voice engine
# -----------------------------------------------------------
Write-Host "[1/4] Installing legends-hyperyap voice engine..." -ForegroundColor Yellow

# Kill any running legends-hyperyap before installing (NSIS upgrades in-place, no uninstall needed)
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
        Write-Host "  legends-hyperyap installed." -ForegroundColor Green
    } else {
        Write-Host "  No release found yet. Build from source with: pnpm tauri build" -ForegroundColor DarkYellow
    }
} catch {
    Write-Host "  No release available yet. Continuing with setup..." -ForegroundColor DarkYellow
}

# -----------------------------------------------------------
# 2. Download speech recognition model
# -----------------------------------------------------------
Write-Host "[2/4] Setting up speech recognition model (~665MB)..." -ForegroundColor Yellow

$resourceDirs = @(
    "$installDir\resources",
    "$env:LOCALAPPDATA\com.avalonreset.hyperyap\resources",
    "$env:APPDATA\com.avalonreset.hyperyap\resources"
)
$targetResourceDir = $resourceDirs | Where-Object { Test-Path (Split-Path $_ -Parent) } | Select-Object -First 1
if (-not $targetResourceDir) { $targetResourceDir = $resourceDirs[0] }
$modelDir = "$targetResourceDir\$modelId"

if (Test-Path "$modelDir\encoder-model.int8.onnx") {
    Write-Host "  Model already downloaded." -ForegroundColor Green
} else {
    $maxRetries = 30
    $downloaded = $false

    for ($attempt = 1; $attempt -le $maxRetries; $attempt++) {
        Write-Host "  Downloading from Hugging Face (attempt $attempt/$maxRetries)..." -ForegroundColor DarkGray
        New-Item -ItemType Directory -Path $modelDir -Force | Out-Null

        $attemptOk = $true
        foreach ($file in $modelFiles) {
            $out = Join-Path $modelDir $file
            $url = "$modelBaseUrl/$file"
            Write-Host "    $file" -ForegroundColor DarkGray
            & curl.exe -L -C - -o $out --retry 3 --retry-delay 2 --connect-timeout 30 $url 2>&1 | Write-Host
            if ($LASTEXITCODE -ne 0) {
                $attemptOk = $false
                break
            }
        }

        if ($attemptOk -and
            (Test-Path "$modelDir\encoder-model.int8.onnx") -and
            (Test-Path "$modelDir\decoder_joint-model.int8.onnx") -and
            (Test-Path "$modelDir\nemo128.onnx") -and
            (Test-Path "$modelDir\vocab.txt")) {
            $downloaded = $true
            break
        }

        Write-Host "  Download interrupted, retrying in 3 seconds..." -ForegroundColor Yellow
        Start-Sleep -Seconds 3
    }

    if (-not $downloaded) {
        throw "Failed to download speech model after $maxRetries attempts. Check your internet connection."
    }

    Write-Host "  Model installed." -ForegroundColor Green
}

# -----------------------------------------------------------
# 3. Verify native hotkey helper + fetch presets
# -----------------------------------------------------------
Write-Host "[3/4] Setting up native hotkey helper..." -ForegroundColor Yellow

# Remove any legacy startup links from pre-native releases
@("$startupDir\hyperyap-hotkeys.lnk") | ForEach-Object {
    if (Test-Path $_) {
        Remove-Item $_ -Force -ErrorAction SilentlyContinue
    }
}

$scriptRoot = $PSScriptRoot
$presetsDir = "$scriptRoot\presets"

if (-not (Test-Path $presetsDir)) {
    Write-Host "  Downloading presets from GitHub..." -ForegroundColor DarkGray
    $presetsDir = "$env:TEMP\hyperyap-presets"
    New-Item -ItemType Directory -Path $presetsDir -Force | Out-Null
    @("settings.json", "llm_connect.json") | ForEach-Object {
        Invoke-WebRequest -Uri "https://raw.githubusercontent.com/$repo/main/presets/$_" -OutFile "$presetsDir\$_" -UseBasicParsing
    }
}

if (Test-Path $hotkeysExe) {
    Write-Host "  Native hotkey helper found at $hotkeysExe." -ForegroundColor Green
} else {
    Write-Host "  Native hotkey helper will be deployed by the application installer." -ForegroundColor DarkGray
}

# -----------------------------------------------------------
# 4. Deploy configs + auto-start
# -----------------------------------------------------------
Write-Host "[4/4] Deploying configs and auto-start..." -ForegroundColor Yellow

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

# HyperYap app auto-start (handled by app itself on first launch; also manages hyperyap-hotkeys.exe)
Write-Host "  Voice engine and hotkey helper auto-start will activate on first launch." -ForegroundColor Green

# -----------------------------------------------------------
# Launch everything
# -----------------------------------------------------------
Write-Host ""
Write-Host "  legends-hyperyap is ready!" -ForegroundColor Green
Write-Host ""
Write-Host "  - Press F13, CapsLock, or Mouse Back to record" -ForegroundColor DarkGray
Write-Host "  - Press again to stop and auto-paste transcription" -ForegroundColor DarkGray
Write-Host "  - Mouse Forward = Enter" -ForegroundColor DarkGray
Write-Host ""

# Start legends-hyperyap (which also launches hyperyap-hotkeys.exe --no-tray)
$appExePaths = @("$installDir\hyperyap.exe", "$installDir\HyperYap.exe", "$env:ProgramFiles\HyperYap\hyperyap.exe")
$appExe = $appExePaths | Where-Object { Test-Path $_ } | Select-Object -First 1
if ($appExe) {
    Write-Host "  Launching legends-hyperyap..." -ForegroundColor DarkGray
    Start-Process $appExe
}

Write-Host ""
Write-Host "  Done. Enjoy yapping." -ForegroundColor Cyan
Write-Host ""
