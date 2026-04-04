# ============================================================
# HyperYap Installer
# One-command setup: voice-to-text + terminal + hotkeys
# ============================================================
#Requires -RunAsAdministrator

param(
    [switch]$All,
    [switch]$Force,
    [switch]$SkipModel,
    [switch]$SkipAHK,
    [switch]$SkipAutostart,
    [switch]$SkipTerminal
)

$ErrorActionPreference = "Stop"

$repo = "avalonreset/hyperyap"
$modelUrl = "https://github.com/Kieirra/murmure-model/releases/download/1.0.0/parakeet-tdt-0.6b-v3-int8.zip"
$ahkInstallerUrl = "https://www.autohotkey.com/download/ahk-v2.exe"
$appDataDir = "$env:APPDATA\com.avalonreset.hyperyap"
$startupDir = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Startup"
$installDir = "$env:LOCALAPPDATA\Programs\HyperYap"
$btRepo = "avalonreset/BenjaminTerm"

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

# -----------------------------------------------------------
# Interactive component picker
# -----------------------------------------------------------
$installVoice = $true
$installTerminal = -not $SkipTerminal
$installHotkeys = -not $SkipAHK

if ($Force) {
    Write-Host "  --Force mode: will overwrite existing configs and reinstall components." -ForegroundColor Yellow
    Write-Host ""
}

if (-not $All -and -not $SkipTerminal -and -not $SkipAHK -and [Environment]::UserInteractive) {
    Write-Host "  What would you like to install?" -ForegroundColor White
    Write-Host ""
    Write-Host "    [1] Everything (recommended)" -ForegroundColor Green
    Write-Host "    [2] Voice engine only" -ForegroundColor DarkGray
    Write-Host "    [3] Voice engine + hotkeys (no terminal)" -ForegroundColor DarkGray
    Write-Host "    [4] Voice engine + terminal (no hotkeys)" -ForegroundColor DarkGray
    Write-Host ""
    $choice = Read-Host "  Choice (1-4, default=1)"
    if ([string]::IsNullOrWhiteSpace($choice)) { $choice = "1" }

    switch ($choice) {
        "1" { $installVoice = $true; $installTerminal = $true; $installHotkeys = $true }
        "2" { $installVoice = $true; $installTerminal = $false; $installHotkeys = $false }
        "3" { $installVoice = $true; $installTerminal = $false; $installHotkeys = $true }
        "4" { $installVoice = $true; $installTerminal = $true; $installHotkeys = $false }
        default { $installVoice = $true; $installTerminal = $true; $installHotkeys = $true }
    }
    Write-Host ""
}

# -----------------------------------------------------------
# 1. Download latest HyperYap release
# -----------------------------------------------------------
Write-Host "[1/6] Downloading latest HyperYap release..." -ForegroundColor Yellow

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
# 2. Install BenjaminTerm
# -----------------------------------------------------------
if ($installTerminal) {
    Write-Host "[2/6] Installing BenjaminTerm terminal..." -ForegroundColor Yellow

    $btExePaths = @(
        "$env:ProgramFiles\BenjaminTerm\benjaminterm-gui.exe",
        "$env:LOCALAPPDATA\Programs\BenjaminTerm\benjaminterm-gui.exe",
        "${env:ProgramFiles(x86)}\BenjaminTerm\benjaminterm-gui.exe"
    )
    $btExe = $btExePaths | Where-Object { Test-Path $_ } | Select-Object -First 1

    if ($btExe -and -not $Force) {
        Write-Host "  BenjaminTerm already installed. Use -Force to reinstall." -ForegroundColor Green
    } else {
        try {
            $btReleaseApi = "https://api.github.com/repos/$btRepo/releases"
            $btReleases = Invoke-RestMethod -Uri $btReleaseApi -Headers @{ "User-Agent" = "HyperYap-Installer" }
            # Find the first release that has a Setup.exe asset
            $btSetupAsset = $null
            foreach ($rel in $btReleases) {
                $btSetupAsset = $rel.assets | Where-Object { $_.name -match "Setup\.exe$" } | Select-Object -First 1
                if ($btSetupAsset) { break }
            }
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
                $btZipAsset = $null
                foreach ($rel in $btReleases) {
                    $btZipAsset = $rel.assets | Where-Object { $_.name -match "windows.*\.zip$" -and $_.name -notmatch "sha256" } | Select-Object -First 1
                    if ($btZipAsset) { break }
                }
                if ($btZipAsset) {
                    $btZipPath = "$env:TEMP\BenjaminTerm.zip"
                    $btInstallDir = "$env:LOCALAPPDATA\Programs\BenjaminTerm"
                    Write-Host "  Downloading $($btZipAsset.name) (portable)..." -ForegroundColor DarkGray
                    Invoke-WebRequest -Uri $btZipAsset.browser_download_url -OutFile $btZipPath -UseBasicParsing
                    Write-Host "  Extracting..." -ForegroundColor DarkGray
                    New-Item -ItemType Directory -Path $btInstallDir -Force | Out-Null
                    Expand-Archive -Path $btZipPath -DestinationPath $btInstallDir -Force
                    Remove-Item $btZipPath -Force
                    Write-Host "  BenjaminTerm installed (portable) to $btInstallDir" -ForegroundColor Green
                } else {
                    Write-Host "  No BenjaminTerm installer found. Install manually from:" -ForegroundColor DarkYellow
                    Write-Host "  https://github.com/$btRepo/releases" -ForegroundColor DarkGray
                }
            }
        } catch {
            Write-Host "  Could not fetch BenjaminTerm release." -ForegroundColor DarkYellow
            Write-Host "  Error: $_" -ForegroundColor DarkGray
            Write-Host "  Install manually: https://github.com/$btRepo/releases" -ForegroundColor DarkGray
        }
    }
} else {
    Write-Host "[2/6] Skipping BenjaminTerm." -ForegroundColor DarkGray
}

# -----------------------------------------------------------
# 3. Download Parakeet model
# -----------------------------------------------------------
if (-not $SkipModel) {
    Write-Host "[3/6] Downloading speech recognition model (~440MB)..." -ForegroundColor Yellow

    # Find where HyperYap stores resources
    $resourceDirs = @(
        "$installDir\resources",
        "$env:LOCALAPPDATA\com.avalonreset.hyperyap\resources",
        "$env:APPDATA\com.avalonreset.hyperyap\resources"
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
    Write-Host "[3/6] Skipping model download (--SkipModel)" -ForegroundColor DarkGray
}

# -----------------------------------------------------------
# 4. Install AutoHotkey v2 (if needed)
# -----------------------------------------------------------
if ($installHotkeys) {
    Write-Host "[4/6] Checking AutoHotkey v2..." -ForegroundColor Yellow

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
    Write-Host "[4/6] Skipping AutoHotkey." -ForegroundColor DarkGray
}

# -----------------------------------------------------------
# 5. Deploy preset configs
# -----------------------------------------------------------
Write-Host "[5/6] Deploying HyperYap configs..." -ForegroundColor Yellow

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

# Clean up old MURmure configs if present
$oldMurmureDir = "$env:APPDATA\com.al1x-ai.murmure"
if ((Test-Path $oldMurmureDir) -and $Force) {
    Write-Host "  Removing old MURmure configs..." -ForegroundColor DarkGray
    Remove-Item "$oldMurmureDir\settings.json" -Force -ErrorAction SilentlyContinue
    Remove-Item "$oldMurmureDir\llm_connect.json" -Force -ErrorAction SilentlyContinue
}

# Also remove old murmure startup shortcuts
$oldMurmureStartup = "$startupDir\murmure-hotkeys.ahk"
if (Test-Path $oldMurmureStartup) {
    Remove-Item $oldMurmureStartup -Force -ErrorAction SilentlyContinue
    Write-Host "  Removed old MURmure startup shortcut." -ForegroundColor DarkGray
}

# Copy app configs
New-Item -ItemType Directory -Path $appDataDir -Force | Out-Null

if ($Force -or -not (Test-Path "$appDataDir\settings.json")) {
    Copy-Item "$presetsDir\settings.json" "$appDataDir\settings.json" -Force
    Write-Host "  Settings deployed." -ForegroundColor Green
} else {
    Write-Host "  Settings already exist. Skipping (use -Force to overwrite)." -ForegroundColor DarkYellow
}

if ($Force -or -not (Test-Path "$appDataDir\llm_connect.json")) {
    Copy-Item "$presetsDir\llm_connect.json" "$appDataDir\llm_connect.json" -Force
    Write-Host "  LLM config deployed." -ForegroundColor Green
} else {
    Write-Host "  LLM config already exists. Skipping (use -Force to overwrite)." -ForegroundColor DarkYellow
}

# Copy AHK scripts to a stable location (if hotkeys selected)
$scriptsInstallDir = "$env:LOCALAPPDATA\HyperYap\scripts"
if ($installHotkeys) {
    New-Item -ItemType Directory -Path $scriptsInstallDir -Force | Out-Null
    Copy-Item "$presetsDir\scripts\*" "$scriptsInstallDir\" -Force
    Write-Host "  Hotkey scripts deployed to $scriptsInstallDir" -ForegroundColor Green
}

# -----------------------------------------------------------
# 6. Set up auto-start
# -----------------------------------------------------------
if (-not $SkipAutostart) {
    Write-Host "[6/6] Configuring auto-start..." -ForegroundColor Yellow

    # AHK hotkeys → Startup folder
    if ($installHotkeys) {
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
    Write-Host "[6/6] Skipping auto-start (--SkipAutostart)" -ForegroundColor DarkGray
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
if ($installHotkeys) {
    $ahkScript = "$scriptsInstallDir\hyperyap-hotkeys.ahk"
    if (Test-Path $ahkScript) {
        Write-Host "  Starting hotkeys..." -ForegroundColor DarkGray
        Start-Process $ahkScript
    }
}

# Start HyperYap app now
if ($appExe) {
    Write-Host "  Launching HyperYap..." -ForegroundColor DarkGray
    Start-Process $appExe
}

# Start BenjaminTerm now
if ($installTerminal) {
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
}

Write-Host ""
Write-Host "  Done. Enjoy yapping." -ForegroundColor Cyan
Write-Host ""
