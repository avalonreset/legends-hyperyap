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
const IDM_PAUSE: u16 = 1;
const IDM_EXIT: u16 = 2;
const VK_F13: u16 = 0x7C;
const CREATE_NO_WINDOW: u32 = 0x08000000;
const CF_BITMAP: u32 = 2;
const CF_DIB: u32 = 8;
const CF_DIBV5: u32 = 17;
const SMART_PASTE_IMAGE_WAIT_MS: u64 = 1_500;
const SMART_PASTE_RETRY_INTERVAL_MS: u64 = 40;
const SMART_PASTE_RESTORE_DELAY_MS: u64 = 750;

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
static PAUSED: AtomicBool = AtomicBool::new(false);
static mut HWND_MAIN: HWND = null_mut();
static mut KB_HOOK: HHOOK = null_mut();
static mut MOUSE_HOOK: HHOOK = null_mut();

// -- Entry point --

static NO_TRAY: AtomicBool = AtomicBool::new(false);

fn main() {
    // Check for --no-tray flag (headless mode, controlled by main HyperYap app)
    let no_tray = std::env::args().any(|arg| arg == "--no-tray");
    NO_TRAY.store(no_tray, Ordering::SeqCst);

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
            hIcon: LoadIconW(hinstance, int_resource(1)), // embedded icon resource ID 1
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

        // Add tray icon only if not in headless mode
        if !no_tray {
            add_tray_icon(HWND_MAIN, hinstance);
        }

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
        if !no_tray {
            remove_tray_icon(HWND_MAIN);
        }
    }
}

// -- Keyboard hook --

unsafe extern "system" fn keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 && !PAUSED.load(Ordering::Relaxed) {
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
    if code >= 0 && !PAUSED.load(Ordering::Relaxed) {
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

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
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
            match (wparam & 0xFFFF) as u16 {
                IDM_PAUSE => {
                    let was_paused = PAUSED.load(Ordering::SeqCst);
                    PAUSED.store(!was_paused, Ordering::SeqCst);
                    let hinstance = GetModuleHandleW(std::ptr::null());
                    if was_paused {
                        update_tray_tip(hwnd, "HYPERYAP HOTKEYS");
                        update_tray_icon(hwnd, LoadIconW(hinstance, int_resource(1)));
                    } else {
                        update_tray_tip(hwnd, "HYPERYAP HOTKEYS (PAUSED)");
                        update_tray_icon(hwnd, LoadIconW(hinstance, int_resource(2)));
                    }
                }
                IDM_EXIT => {
                    remove_tray_icon(hwnd);
                    PostQuitMessage(0);
                }
                _ => {}
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
        if let Some(proc_name) = get_foreground_process_name() {
            let lower = proc_name.to_lowercase();
            if TERMINALS.iter().any(|t| lower == *t)
                && wait_for_clipboard_image(std::time::Duration::from_millis(
                    SMART_PASTE_IMAGE_WAIT_MS,
                ))
            {
                if let Ok(image_path) = run_clipboard_image_save() {
                    send_key_combo(VK_CONTROL, 0x56 /* V */);
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_millis(
                            SMART_PASTE_RESTORE_DELAY_MS,
                        ));
                        let _ = restore_clipboard_image(&image_path);
                    });
                    return;
                }
            }
        }

        // Non-terminal or no image: just send Ctrl+V normally
        send_key_combo(VK_CONTROL, 0x56 /* V */);
    }
}

fn wait_for_clipboard_image(timeout: std::time::Duration) -> bool {
    let started_at = std::time::Instant::now();

    while started_at.elapsed() <= timeout {
        if clipboard_has_image() {
            return true;
        }

        std::thread::sleep(std::time::Duration::from_millis(
            SMART_PASTE_RETRY_INTERVAL_MS,
        ));
    }

    false
}

fn clipboard_has_image() -> bool {
    unsafe {
        IsClipboardFormatAvailable(CF_BITMAP) != 0
            || IsClipboardFormatAvailable(CF_DIB) != 0
            || IsClipboardFormatAvailable(CF_DIBV5) != 0
            || IsClipboardFormatAvailable(RegisterClipboardFormatW(wide("PNG").as_ptr())) != 0
    }
}

fn run_clipboard_image_save() -> Result<String, String> {
    let script = r#"
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

$img = $null
$deadline = (Get-Date).AddMilliseconds(2500)

while ((Get-Date) -lt $deadline -and -not $img) {
    try {
        $img = [System.Windows.Forms.Clipboard]::GetImage()
    } catch {
        $img = $null
    }

    if (-not $img) {
        Start-Sleep -Milliseconds 50
    }
}

if (-not $img) {
    exit 1
}

try {
    $dir = "$env:USERPROFILE\screenshots"
    if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Path $dir | Out-Null }
    $timestamp = Get-Date -Format "yyyy-MM-dd_HH-mm-ss_fff"
    $path = "$dir\screenshot_$timestamp.png"
    $img.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
    [System.Windows.Forms.Clipboard]::SetText($path.Replace('\', '/'))
    Write-Output $path
    exit 0
} catch {
    exit 1
} finally {
    $img.Dispose()
}
"#;

    let output = std::process::Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-Sta",
            "-WindowStyle",
            "Hidden",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("Failed to run clipboard image save script: {}", e))?;

    if !output.status.success() {
        return Err("Clipboard image save script did not find an image".to_string());
    }

    let image_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if image_path.is_empty() {
        return Err("Clipboard image save script returned an empty path".to_string());
    }

    Ok(image_path)
}

fn restore_clipboard_image(image_path: &str) -> Result<(), String> {
    let script = r#"
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

$path = $env:HYPERYAP_SMART_PASTE_IMAGE_PATH
if ([string]::IsNullOrWhiteSpace($path) -or -not (Test-Path -LiteralPath $path)) {
    exit 1
}

$img = $null
try {
    $img = [System.Drawing.Image]::FromFile($path)
    [System.Windows.Forms.Clipboard]::SetImage($img)
    exit 0
} catch {
    exit 1
} finally {
    $img.Dispose()
}
"#;

    let status = std::process::Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-Sta",
            "-WindowStyle",
            "Hidden",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .env("HYPERYAP_SMART_PASTE_IMAGE_PATH", image_path)
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map_err(|e| format!("Failed to run clipboard image restore script: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err("Clipboard image restore script failed".to_string())
    }
}

// -- Tray icon --

unsafe fn add_tray_icon(hwnd: HWND, hinstance: HINSTANCE) {
    let mut nid: NOTIFYICONDATAW = zeroed();
    nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
    nid.uCallbackMessage = WM_TRAYICON;
    nid.hIcon = LoadIconW(hinstance, int_resource(1));

    // If embedded icon failed, use default application icon
    if nid.hIcon.is_null() {
        nid.hIcon = LoadIconW(null_mut(), IDI_APPLICATION);
    }

    let tip = wide("HYPERYAP HOTKEYS");
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

    let is_paused = PAUSED.load(Ordering::SeqCst);
    let pause_text = if is_paused {
        wide("Resume Hotkeys")
    } else {
        wide("Pause Hotkeys")
    };
    AppendMenuW(menu, MF_STRING, IDM_PAUSE as usize, pause_text.as_ptr());

    // Separator
    AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());

    let quit_text = wide("Quit");
    AppendMenuW(menu, MF_STRING, IDM_EXIT as usize, quit_text.as_ptr());

    let mut pt: POINT = zeroed();
    GetCursorPos(&mut pt);

    // Required for the menu to dismiss when clicking elsewhere
    SetForegroundWindow(hwnd);
    TrackPopupMenu(
        menu,
        TPM_RIGHTALIGN | TPM_BOTTOMALIGN,
        pt.x,
        pt.y,
        0,
        hwnd,
        std::ptr::null(),
    );
    DestroyMenu(menu);
}

unsafe fn update_tray_icon(hwnd: HWND, hicon: HICON) {
    let mut nid: NOTIFYICONDATAW = zeroed();
    nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    nid.uFlags = NIF_ICON;
    nid.hIcon = hicon;
    Shell_NotifyIconW(NIM_MODIFY, &nid);
}

unsafe fn update_tray_tip(hwnd: HWND, tip_str: &str) {
    let mut nid: NOTIFYICONDATAW = zeroed();
    nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    nid.uFlags = NIF_TIP;

    let tip = wide(tip_str);
    let len = tip.len().min(128);
    nid.szTip[..len].copy_from_slice(&tip[..len]);

    Shell_NotifyIconW(NIM_MODIFY, &nid);
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
    fn RegisterClipboardFormatW(lpszFormat: *const u16) -> u32;
}

// -- Utility --

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn hiword(dword: u32) -> u16 {
    (dword >> 16) as u16
}

fn int_resource(resource_id: u16) -> *const u16 {
    std::ptr::without_provenance(resource_id as usize)
}

// null_mut helper for statics
const fn null_mut() -> *mut std::ffi::c_void {
    std::ptr::null_mut()
}
