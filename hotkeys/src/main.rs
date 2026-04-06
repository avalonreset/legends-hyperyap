#![windows_subsystem = "windows"]

use std::mem::{size_of, zeroed};
use std::os::windows::process::CommandExt;
use std::sync::atomic::{AtomicBool, Ordering};
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Threading::*;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
use windows_sys::Win32::UI::Shell::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

// -- Constants --

const WM_TRAYICON: u32 = WM_APP + 1;
const WM_SMART_PASTE: u32 = WM_APP + 2;
const IDM_EXIT: u16 = 1;
const VK_F13: u16 = 0x7C;
const CREATE_NO_WINDOW: u32 = 0x08000000;

// Terminal process names (lowercase) that get smart image paste
const TERMINALS: &[&str] = &[
    "benjaminterm-gui.exe",
    "wezterm-gui.exe",
    "windowsterminal.exe",
    "powershell.exe",
    "pwsh.exe",
    "cmd.exe",
    "alacritty.exe",
    "conemu.exe",
    "conemu64.exe",
    "hyper.exe",
    "mintty.exe",
    "tabby.exe",
    "warp.exe",
    "mobaxterm.exe",
];

// -- Globals (required for hook callbacks) --

static SUPPRESS_V_UP: AtomicBool = AtomicBool::new(false);
static mut HWND_MAIN: HWND = null_mut();
static mut KB_HOOK: HHOOK = null_mut();
static mut MOUSE_HOOK: HHOOK = null_mut();

// -- Entry point --

fn main() {
    unsafe {
        // Turn off CapsLock if it's currently on
        if GetKeyState(VK_CAPITAL as i32) & 1 != 0 {
            send_key(VK_CAPITAL, false);
            send_key(VK_CAPITAL, true);
        }

        let hinstance = GetModuleHandleW(std::ptr::null());

        // Register window class for our hidden message window
        let class_name = wide("HyperYapHotkeys");
        let wc = WNDCLASSW {
            lpfnWndProc: Some(wnd_proc),
            hInstance: hinstance,
            lpszClassName: class_name.as_ptr(),
            style: 0,
            cbClsExtra: 0,
            cbWndExtra: 0,
            hIcon: LoadIconW(hinstance, 1 as *const u16), // embedded icon resource ID 1
            hCursor: null_mut(),
            hbrBackground: null_mut(),
            lpszMenuName: std::ptr::null(),
        };
        RegisterClassW(&wc);

        // Create hidden window
        HWND_MAIN = CreateWindowExW(
            0,
            class_name.as_ptr(),
            wide("HyperYap Hotkeys").as_ptr(),
            0,
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            null_mut(),
            hinstance,
            null_mut(),
        );

        // Add tray icon
        add_tray_icon(HWND_MAIN, hinstance);

        // Install low-level hooks
        KB_HOOK = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook), hinstance, 0);
        MOUSE_HOOK = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook), hinstance, 0);

        // Message loop
        let mut msg: MSG = zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Cleanup
        UnhookWindowsHookEx(KB_HOOK);
        UnhookWindowsHookEx(MOUSE_HOOK);
        remove_tray_icon(HWND_MAIN);
    }
}

// -- Keyboard hook --

unsafe extern "system" fn keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let kb = &*(lparam as *const KBDLLHOOKSTRUCT);

        // Skip injected events (our own SendInput calls)
        if kb.flags & LLKHF_INJECTED != 0 {
            return CallNextHookEx(KB_HOOK, code, wparam, lparam);
        }

        let is_down = wparam == WM_KEYDOWN as usize || wparam == WM_SYSKEYDOWN as usize;
        let is_up = wparam == WM_KEYUP as usize || wparam == WM_SYSKEYUP as usize;

        match kb.vkCode as u16 {
            // CapsLock -> F13
            VK_CAPITAL => {
                if is_down {
                    send_key(VK_F13, false);
                } else if is_up {
                    send_key(VK_F13, true);
                }
                return 1; // suppress original CapsLock
            }

            // Ctrl+V smart paste intercept
            0x56 /* VK_V */ if is_ctrl_held() => {
                if is_down {
                    // Suppress this V keydown, handle asynchronously
                    SUPPRESS_V_UP.store(true, Ordering::SeqCst);
                    PostMessageW(HWND_MAIN, WM_SMART_PASTE, 0, 0);
                    return 1;
                }
                if is_up && SUPPRESS_V_UP.load(Ordering::SeqCst) {
                    SUPPRESS_V_UP.store(false, Ordering::SeqCst);
                    return 1; // suppress matching keyup
                }
            }

            _ => {}
        }
    }
    CallNextHookEx(KB_HOOK, code, wparam, lparam)
}

// -- Mouse hook --

unsafe extern "system" fn mouse_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let ms = &*(lparam as *const MSLLHOOKSTRUCT);

        // Skip injected
        if ms.flags & 1 != 0 {
            return CallNextHookEx(MOUSE_HOOK, code, wparam, lparam);
        }

        match wparam as u32 {
            // XButton1 (back) -> F13
            WM_XBUTTONDOWN if hiword(ms.mouseData) == 1 => {
                send_key(VK_F13, false);
                return 1;
            }
            WM_XBUTTONUP if hiword(ms.mouseData) == 1 => {
                send_key(VK_F13, true);
                return 1;
            }

            // XButton2 (forward) -> Enter
            WM_XBUTTONDOWN if hiword(ms.mouseData) == 2 => {
                send_key(VK_RETURN, false);
                return 1;
            }
            WM_XBUTTONUP if hiword(ms.mouseData) == 2 => {
                send_key(VK_RETURN, true);
                return 1;
            }

            _ => {}
        }
    }
    CallNextHookEx(MOUSE_HOOK, code, wparam, lparam)
}

// -- Window procedure --

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_SMART_PASTE => {
            handle_smart_paste();
            0
        }

        WM_TRAYICON => {
            let event = (lparam & 0xFFFF) as u32;
            match event {
                WM_RBUTTONUP | WM_CONTEXTMENU => {
                    show_tray_menu(hwnd);
                }
                _ => {}
            }
            0
        }

        WM_COMMAND => {
            if (wparam & 0xFFFF) as u16 == IDM_EXIT {
                remove_tray_icon(hwnd);
                PostQuitMessage(0);
            }
            0
        }

        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

// -- Smart paste logic --

fn handle_smart_paste() {
    unsafe {
        // Check if clipboard has a bitmap (CF_BITMAP = 2)
        let has_image = IsClipboardFormatAvailable(2) != 0;

        if has_image {
            if let Some(proc_name) = get_foreground_process_name() {
                let lower = proc_name.to_lowercase();
                if TERMINALS.iter().any(|t| lower == *t) {
                    // Run PowerShell to save clipboard image and replace with file path
                    run_clipboard_image_save();
                }
            }
        }

        // Send Ctrl+V (our injected input will pass through the hook)
        send_key_combo(VK_CONTROL, 0x56 /* V */);
    }
}

fn run_clipboard_image_save() {
    // Embedded PowerShell script (same logic as clipboard-image-paste.ps1)
    let script = r#"
Add-Type -AssemblyName System.Windows.Forms
$img = [System.Windows.Forms.Clipboard]::GetImage()
if ($img) {
    $dir = "$env:USERPROFILE\screenshots"
    if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Path $dir | Out-Null }
    $timestamp = Get-Date -Format "yyyy-MM-dd_HH-mm-ss"
    $path = "$dir\screenshot_$timestamp.png"
    $img.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
    $img.Dispose()
    [System.Windows.Forms.Clipboard]::SetText($path.Replace('\', '/'))
}
"#;

    let _ = std::process::Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-WindowStyle",
            "Hidden",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .status();
}

// -- Tray icon --

unsafe fn add_tray_icon(hwnd: HWND, hinstance: HINSTANCE) {
    let mut nid: NOTIFYICONDATAW = zeroed();
    nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
    nid.uCallbackMessage = WM_TRAYICON;
    nid.hIcon = LoadIconW(hinstance, 1 as *const u16);

    // If embedded icon failed, use default application icon
    if nid.hIcon.is_null() {
        nid.hIcon = LoadIconW(null_mut(), IDI_APPLICATION);
    }

    let tip = wide("HyperYap Hotkeys");
    let len = tip.len().min(128);
    nid.szTip[..len].copy_from_slice(&tip[..len]);

    Shell_NotifyIconW(NIM_ADD, &nid);
}

unsafe fn remove_tray_icon(hwnd: HWND) {
    let mut nid: NOTIFYICONDATAW = zeroed();
    nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    Shell_NotifyIconW(NIM_DELETE, &nid);
}

unsafe fn show_tray_menu(hwnd: HWND) {
    let menu = CreatePopupMenu();
    let exit_text = wide("Exit");
    AppendMenuW(menu, MF_STRING, IDM_EXIT as usize, exit_text.as_ptr());

    let mut pt: POINT = zeroed();
    GetCursorPos(&mut pt);

    // Required for the menu to dismiss when clicking elsewhere
    SetForegroundWindow(hwnd);
    TrackPopupMenu(menu, TPM_RIGHTALIGN | TPM_BOTTOMALIGN, pt.x, pt.y, 0, hwnd, std::ptr::null());
    DestroyMenu(menu);
}

// -- Input helpers --

unsafe fn send_key(vk: u16, key_up: bool) {
    let mut input: INPUT = zeroed();
    input.r#type = INPUT_KEYBOARD;
    input.Anonymous.ki.wVk = vk;
    input.Anonymous.ki.dwFlags = if key_up { KEYEVENTF_KEYUP } else { 0 };
    SendInput(1, &input, size_of::<INPUT>() as i32);
}

unsafe fn send_key_combo(modifier: u16, key: u16) {
    let mut inputs: [INPUT; 4] = zeroed();

    // Modifier down
    inputs[0].r#type = INPUT_KEYBOARD;
    inputs[0].Anonymous.ki.wVk = modifier;

    // Key down
    inputs[1].r#type = INPUT_KEYBOARD;
    inputs[1].Anonymous.ki.wVk = key;

    // Key up
    inputs[2].r#type = INPUT_KEYBOARD;
    inputs[2].Anonymous.ki.wVk = key;
    inputs[2].Anonymous.ki.dwFlags = KEYEVENTF_KEYUP;

    // Modifier up
    inputs[3].r#type = INPUT_KEYBOARD;
    inputs[3].Anonymous.ki.wVk = modifier;
    inputs[3].Anonymous.ki.dwFlags = KEYEVENTF_KEYUP;

    SendInput(4, inputs.as_ptr(), size_of::<INPUT>() as i32);
}

unsafe fn is_ctrl_held() -> bool {
    GetAsyncKeyState(VK_CONTROL as i32) < 0
}

// -- Process detection --

unsafe fn get_foreground_process_name() -> Option<String> {
    let fg = GetForegroundWindow();
    if fg.is_null() {
        return None;
    }

    let mut pid: u32 = 0;
    GetWindowThreadProcessId(fg, &mut pid);
    if pid == 0 {
        return None;
    }

    let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid);
    if handle.is_null() {
        return None;
    }

    let mut buf = [0u16; 260];
    let mut len = buf.len() as u32;
    let ok = QueryFullProcessImageNameW(handle, 0, buf.as_mut_ptr(), &mut len);
    CloseHandle(handle);

    if ok == 0 || len == 0 {
        return None;
    }

    let path = String::from_utf16_lossy(&buf[..len as usize]);
    path.rsplit('\\').next().map(|s| s.to_string())
}

// -- Win32 clipboard check --

#[link(name = "user32")]
extern "system" {
    fn IsClipboardFormatAvailable(format: u32) -> BOOL;
}

// -- Utility --

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn hiword(dword: u32) -> u16 {
    (dword >> 16) as u16
}

// null_mut helper for statics
const fn null_mut() -> *mut std::ffi::c_void {
    std::ptr::null_mut()
}
