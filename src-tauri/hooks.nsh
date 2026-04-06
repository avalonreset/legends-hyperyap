; HyperYap NSIS installer hooks
; Deploys hotkey daemon, creates startup shortcut, cleans up old entries

!macro NSIS_HOOK_PREINSTALL
  ; Kill any running hotkey daemon before install
  ; Using cmd /c with output suppressed to minimize console flash
  ExecWait 'cmd /c taskkill /F /IM hyperyap-hotkeys.exe >nul 2>&1'
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; --- Deploy hotkey daemon ---
  CreateDirectory "$LOCALAPPDATA\HyperYap"
  CopyFiles /SILENT "$INSTDIR\_up_\hotkeys\target\release\hyperyap-hotkeys.exe" "$LOCALAPPDATA\HyperYap\hyperyap-hotkeys.exe"

  ; --- Remove old MURmure and AHK startup entries ---
  Delete "$SMSTARTUP\murmure-hotkeys.ahk"
  Delete "$SMSTARTUP\murmure-hotkeys.lnk"
  Delete "$SMSTARTUP\Murmure.lnk"
  Delete "$SMSTARTUP\murmure.lnk"
  Delete "$SMSTARTUP\hyperyap-hotkeys.ahk"

  ; --- Remove old AHK scripts if present ---
  Delete "$LOCALAPPDATA\HyperYap\scripts\hyperyap-hotkeys.ahk"
  Delete "$LOCALAPPDATA\HyperYap\scripts\clipboard-image-paste.ps1"
  RMDir "$LOCALAPPDATA\HyperYap\scripts"

  ; --- Create startup shortcut for the hotkey daemon ---
  CreateShortcut "$SMSTARTUP\hyperyap-hotkeys.lnk" "$LOCALAPPDATA\HyperYap\hyperyap-hotkeys.exe"

  ; --- Do NOT auto-launch during install. The NSIS finish page
  ; "Run hyperyap" checkbox launches the main app. The hotkey daemon
  ; starts on next login via the startup shortcut. ---
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ; Kill running daemon
  ExecWait 'cmd /c taskkill /F /IM hyperyap-hotkeys.exe >nul 2>&1'

  ; Clean up deployed files and startup shortcut
  Delete "$LOCALAPPDATA\HyperYap\hyperyap-hotkeys.exe"
  Delete "$LOCALAPPDATA\HyperYap\scripts\hyperyap-hotkeys.ahk"
  Delete "$LOCALAPPDATA\HyperYap\scripts\clipboard-image-paste.ps1"
  RMDir "$LOCALAPPDATA\HyperYap\scripts"
  RMDir "$LOCALAPPDATA\HyperYap"
  Delete "$SMSTARTUP\hyperyap-hotkeys.lnk"
!macroend
