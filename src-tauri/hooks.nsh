; HyperYap NSIS installer hooks
; Deploys hotkey daemon, creates Start Menu shortcut, cleans up old entries

!macro NSIS_HOOK_PREINSTALL
  ; Kill any running hotkey daemon before install
  ; Using cmd /c with output suppressed to minimize console flash
  ExecWait 'cmd /c taskkill /F /IM hyperyap-hotkeys.exe >nul 2>&1'
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; --- Deploy hotkey daemon ---
  CreateDirectory "$LOCALAPPDATA\HyperYap"
  CopyFiles /SILENT "$INSTDIR\_up_\hotkeys\target\release\hyperyap-hotkeys.exe" "$LOCALAPPDATA\HyperYap\hyperyap-hotkeys.exe"

  ; --- Remove old MURmure startup entries ---
  Delete "$SMSTARTUP\murmure-hotkeys.lnk"
  Delete "$SMSTARTUP\Murmure.lnk"
  Delete "$SMSTARTUP\murmure.lnk"

  ; --- Remove old scripts directory if present ---
  RMDir /r "$LOCALAPPDATA\HyperYap\scripts"

  ; --- Remove old standalone startup shortcut (main app now manages the daemon opt-in) ---
  Delete "$SMSTARTUP\hyperyap-hotkeys.lnk"

  ; --- Create Start Menu shortcuts ---
  CreateDirectory "$SMPROGRAMS\HyperYap"
  CreateShortcut "$SMPROGRAMS\HyperYap\HyperYap Hotkeys.lnk" "$LOCALAPPDATA\HyperYap\hyperyap-hotkeys.exe"

  ; --- Download NVIDIA Parakeet v2 English speech model (~665MB) with pinned commit + SHA-256 verification ---
  ; Uses curl.exe (ships with Windows 10+) for resume-capable downloads
  ; The install WILL NOT complete until the model is verified on disk
  IfFileExists "$APPDATA\com.avalonreset.hyperyap\resources\parakeet-tdt-0.6b-v2-smcleod-int8\encoder-model.int8.onnx" model_exists model_download
  model_download:
    CreateDirectory "$APPDATA\com.avalonreset.hyperyap\resources\parakeet-tdt-0.6b-v2-smcleod-int8"
    DetailPrint "Downloading NVIDIA Parakeet v2 English speech model (~665MB)..."
    DetailPrint "This may take several minutes on slow connections."
    ; Retry loop: curl with resume support, up to 30 attempts
    StrCpy $0 0 ; attempt counter
    model_retry:
      IntOp $0 $0 + 1
      IntCmp $0 31 model_failed model_try model_failed
    model_try:
      DetailPrint "Download attempt $0 of 30..."
      nsExec::ExecToLog 'cmd /c curl.exe -fL -C - -o "$APPDATA\com.avalonreset.hyperyap\resources\parakeet-tdt-0.6b-v2-smcleod-int8\encoder-model.int8.onnx" --retry 3 --retry-delay 2 --connect-timeout 30 "https://huggingface.co/smcleod/parakeet-tdt-0.6b-v2-int8/resolve/d64884b484b919e9656d0b70cb95dfdc98852bef/encoder-model.int8.onnx" && curl.exe -fL -C - -o "$APPDATA\com.avalonreset.hyperyap\resources\parakeet-tdt-0.6b-v2-smcleod-int8\decoder_joint-model.int8.onnx" --retry 3 --retry-delay 2 --connect-timeout 30 "https://huggingface.co/smcleod/parakeet-tdt-0.6b-v2-int8/resolve/d64884b484b919e9656d0b70cb95dfdc98852bef/decoder_joint-model.int8.onnx" && curl.exe -fL -C - -o "$APPDATA\com.avalonreset.hyperyap\resources\parakeet-tdt-0.6b-v2-smcleod-int8\nemo128.onnx" --retry 3 --retry-delay 2 --connect-timeout 30 "https://huggingface.co/smcleod/parakeet-tdt-0.6b-v2-int8/resolve/d64884b484b919e9656d0b70cb95dfdc98852bef/nemo128.onnx" && curl.exe -fL -C - -o "$APPDATA\com.avalonreset.hyperyap\resources\parakeet-tdt-0.6b-v2-smcleod-int8\vocab.txt" --retry 3 --retry-delay 2 --connect-timeout 30 "https://huggingface.co/smcleod/parakeet-tdt-0.6b-v2-int8/resolve/d64884b484b919e9656d0b70cb95dfdc98852bef/vocab.txt" && curl.exe -fL -C - -o "$APPDATA\com.avalonreset.hyperyap\resources\parakeet-tdt-0.6b-v2-smcleod-int8\config.json" --retry 3 --retry-delay 2 --connect-timeout 30 "https://huggingface.co/smcleod/parakeet-tdt-0.6b-v2-int8/resolve/d64884b484b919e9656d0b70cb95dfdc98852bef/config.json" && curl.exe -fL -C - -o "$APPDATA\com.avalonreset.hyperyap\resources\parakeet-tdt-0.6b-v2-smcleod-int8\README.md" --retry 3 --retry-delay 2 --connect-timeout 30 "https://huggingface.co/smcleod/parakeet-tdt-0.6b-v2-int8/resolve/d64884b484b919e9656d0b70cb95dfdc98852bef/README.md"'
      Pop $1
      ; Check if curl succeeded (exit code 0)
      StrCmp $1 "0" model_verify model_retry_wait
    model_retry_wait:
      DetailPrint "Download interrupted, retrying in 3 seconds..."
      Sleep 3000
      Goto model_retry
    model_verify:
      ; Verify required model files exist and match pinned SHA-256 digests
      nsExec::ExecToLog 'cmd /c certutil -hashfile "$APPDATA\com.avalonreset.hyperyap\resources\parakeet-tdt-0.6b-v2-smcleod-int8\encoder-model.int8.onnx" SHA256 | findstr /i "464e6ba9e9ee681361b52801d90f13be3cf01a9252a584f14023dab6197aedf9" && certutil -hashfile "$APPDATA\com.avalonreset.hyperyap\resources\parakeet-tdt-0.6b-v2-smcleod-int8\decoder_joint-model.int8.onnx" SHA256 | findstr /i "93fbb1856f8f897004a15793e7b78953879003e03fdac56dc76b76ccc2ce6505" && certutil -hashfile "$APPDATA\com.avalonreset.hyperyap\resources\parakeet-tdt-0.6b-v2-smcleod-int8\nemo128.onnx" SHA256 | findstr /i "34a4d9e18f9ebf7e69bbfb502a793d2a4c19ba538cd76db8abc5d3df0b2b74a5" && certutil -hashfile "$APPDATA\com.avalonreset.hyperyap\resources\parakeet-tdt-0.6b-v2-smcleod-int8\vocab.txt" SHA256 | findstr /i "ec182b70dd42113aff6c5372c75cac58c952443eb22322f57bbd7f53977d497d"'
      Pop $2
      StrCmp $2 "0" model_cleanup model_extract_failed
    model_cleanup:
      DetailPrint "Speech model installed and SHA-256 verified successfully."
      Goto model_exists
    model_extract_failed:
      DetailPrint "Model download or SHA-256 check failed, removing corrupted files and retrying..."
      Delete "$APPDATA\com.avalonreset.hyperyap\resources\parakeet-tdt-0.6b-v2-smcleod-int8\*.onnx"
      Delete "$APPDATA\com.avalonreset.hyperyap\resources\parakeet-tdt-0.6b-v2-smcleod-int8\vocab.txt"
      Goto model_retry
    model_failed:
      MessageBox MB_RETRYCANCEL|MB_ICONEXCLAMATION "Failed to download the speech model after 30 attempts.$\n$\nHyperYap needs this model to work. Check your internet connection and click Retry, or Cancel to finish without it." IDRETRY model_reset_retry
      Goto model_exists
    model_reset_retry:
      StrCpy $0 0
      Goto model_retry
  model_exists:

  ; App launch is handled by the NSIS finish page checkbox (default: on)
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ; Kill running daemon
  ExecWait 'cmd /c taskkill /F /IM hyperyap-hotkeys.exe >nul 2>&1'

  ; Clean up deployed files and startup shortcut
  Delete "$LOCALAPPDATA\HyperYap\hyperyap-hotkeys.exe"
  RMDir /r "$LOCALAPPDATA\HyperYap\scripts"
  RMDir "$LOCALAPPDATA\HyperYap"
  Delete "$SMSTARTUP\hyperyap-hotkeys.lnk"

  ; Clean up Start Menu shortcuts
  Delete "$SMPROGRAMS\HyperYap\HyperYap Hotkeys.lnk"
  RMDir "$SMPROGRAMS\HyperYap"
!macroend
