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

  ; --- Download NVIDIA Parakeet speech model (~440MB) with retry ---
  ; Uses curl.exe (ships with Windows 10+) for resume-capable downloads
  ; The install WILL NOT complete until the model is verified on disk
  IfFileExists "$APPDATA\com.avalonreset.hyperyap\resources\parakeet-tdt-0.6b-v3-int8\encoder-model.int8.onnx" model_exists model_download
  model_download:
    CreateDirectory "$APPDATA\com.avalonreset.hyperyap\resources"
    DetailPrint "Downloading NVIDIA Parakeet speech model (~440MB)..."
    DetailPrint "This may take several minutes on slow connections."
    ; Retry loop: curl with resume support, up to 30 attempts
    StrCpy $0 0 ; attempt counter
    model_retry:
      IntOp $0 $0 + 1
      IntCmp $0 31 model_failed model_try model_failed
    model_try:
      DetailPrint "Download attempt $0 of 30..."
      nsExec::ExecToLog 'cmd /c curl.exe -L -C - -o "$TEMP\parakeet-model.zip" --retry 3 --retry-delay 2 --connect-timeout 30 "https://github.com/Kieirra/murmure-model/releases/download/1.0.0/parakeet-tdt-0.6b-v3-int8.zip"'
      Pop $1
      ; Check if curl succeeded (exit code 0)
      StrCmp $1 "0" model_extract model_retry_wait
    model_retry_wait:
      DetailPrint "Download interrupted, retrying in 3 seconds..."
      Sleep 3000
      Goto model_retry
    model_extract:
      DetailPrint "Extracting speech model..."
      nsExec::ExecToLog 'cmd /c powershell.exe -NoProfile -Command "Expand-Archive -Path \"$TEMP\parakeet-model.zip\" -DestinationPath \"$APPDATA\com.avalonreset.hyperyap\resources\" -Force"'
      ; Verify extraction succeeded
      IfFileExists "$APPDATA\com.avalonreset.hyperyap\resources\parakeet-tdt-0.6b-v3-int8\encoder-model.int8.onnx" model_cleanup model_extract_failed
    model_cleanup:
      Delete "$TEMP\parakeet-model.zip"
      DetailPrint "Speech model installed successfully."
      Goto model_exists
    model_extract_failed:
      ; Zip may be corrupt/incomplete — delete and retry download
      Delete "$TEMP\parakeet-model.zip"
      DetailPrint "Extraction failed (incomplete download?), retrying..."
      Goto model_retry
    model_failed:
      Delete "$TEMP\parakeet-model.zip"
      MessageBox MB_RETRYCANCEL|MB_ICONEXCLAMATION "Failed to download the speech model after 30 attempts.$\n$\nHyperYap needs this model to work. Check your internet connection and click Retry, or Cancel to finish without it." IDRETRY model_reset_retry
      Goto model_exists
    model_reset_retry:
      StrCpy $0 0
      Goto model_retry
  model_exists:

  ; --- Launch the hotkey daemon ---
  Exec '"$LOCALAPPDATA\HyperYap\hyperyap-hotkeys.exe"'
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
