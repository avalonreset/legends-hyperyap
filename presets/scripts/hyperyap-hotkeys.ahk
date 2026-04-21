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
    exe := ""
    try {
        exe := StrLower(WinGetProcessName("A"))
    }

    if (IsSmartPasteTerminal(exe)) {
        if (WaitForClipboardImage(120)) {
            scriptDir := A_ScriptDir
            RunWait('powershell.exe -NoProfile -Sta -WindowStyle Hidden -ExecutionPolicy Bypass -File "' scriptDir '\clipboard-image-paste.ps1"',, "Hide")
            Send "^v"
            return
        }

        if (HasClipboardText()) {
            Send "^v"
            return
        }

        if (WaitForClipboardImage()) {
            scriptDir := A_ScriptDir
            RunWait('powershell.exe -NoProfile -Sta -WindowStyle Hidden -ExecutionPolicy Bypass -File "' scriptDir '\clipboard-image-paste.ps1"',, "Hide")
            Send "^v"
        }

        return
    }

    Send "^v"
}

IsSmartPasteTerminal(exe) {
    return exe = "benjaminterm-gui.exe"
        || exe = "wezterm-gui.exe"
        || exe = "windowsterminal.exe"
        || exe = "powershell.exe"
        || exe = "pwsh.exe"
        || exe = "cmd.exe"
        || exe = "alacritty.exe"
        || exe = "conemu.exe"
        || exe = "conemu64.exe"
        || exe = "hyper.exe"
        || exe = "mintty.exe"
        || exe = "tabby.exe"
        || exe = "warp.exe"
        || exe = "mobaxterm.exe"
}

HasClipboardImage() {
    return DllCall("IsClipboardFormatAvailable", "UInt", 2)
        || DllCall("IsClipboardFormatAvailable", "UInt", 8)
        || DllCall("IsClipboardFormatAvailable", "UInt", 17)
}

HasClipboardText() {
    return DllCall("IsClipboardFormatAvailable", "UInt", 13)
        || DllCall("IsClipboardFormatAvailable", "UInt", 1)
}

WaitForClipboardImage(timeoutMs := 5000) {
    start := A_TickCount
    while (A_TickCount - start <= timeoutMs) {
        if HasClipboardImage() {
            return true
        }
        Sleep 40
    }
    return false
}
