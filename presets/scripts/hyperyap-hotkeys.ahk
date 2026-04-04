#Requires AutoHotkey v2.0
#SingleInstance Force

; =============================================================
; HyperYap Hotkeys
; Maps mouse side buttons and CapsLock to F13 for voice control
; Smart paste: intercepts Ctrl+V in terminals to handle images
; =============================================================

; === Caps Lock disabled (repurposed as F13) ===
SetCapsLockState("AlwaysOff")

; === Mouse side buttons ===
XButton1::F13        ; Mouse back button → start/stop recording
XButton2::Enter      ; Mouse forward button → Enter key
CapsLock::F13        ; CapsLock → start/stop recording

; === Smart Paste: Ctrl+V intercept ===
; In terminals: save clipboard image as PNG, swap clipboard to file path, paste.
; Everywhere else: normal Ctrl+V, clipboard untouched.
$^v:: {
    if DllCall("IsClipboardFormatAvailable", "UInt", 2) {
        exe := ""
        try {
            exe := StrLower(WinGetProcessName("A"))
        }
        ; Add your terminal executables here
        if (exe = "windowsterminal.exe"
            || exe = "powershell.exe"
            || exe = "pwsh.exe"
            || exe = "cmd.exe"
            || exe = "wezterm-gui.exe"
            || exe = "alacritty.exe") {
            scriptDir := A_ScriptDir
            RunWait('powershell.exe -NoProfile -WindowStyle Hidden -ExecutionPolicy Bypass -File "' scriptDir '\clipboard-image-paste.ps1"',, "Hide")
        }
    }
    Send "^v"
}
