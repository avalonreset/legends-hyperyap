use std::sync::atomic::{AtomicBool, Ordering};
use tauri::menu::{CheckMenuItem, Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

static HOTKEYS_PAUSED: AtomicBool = AtomicBool::new(false);

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let show_i = MenuItem::with_id(app, "show", "Open HyperYap", true, None::<&str>)?;
    let hotkeys_i =
        CheckMenuItem::with_id(app, "hotkeys", "Hotkeys Enabled", true, true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_i, &hotkeys_i, &quit_i])?;

    let builder = TrayIconBuilder::new()
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "hotkeys" => {
                let was_paused = HOTKEYS_PAUSED.load(Ordering::SeqCst);
                HOTKEYS_PAUSED.store(!was_paused, Ordering::SeqCst);
                toggle_hotkeys_daemon(!was_paused);
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        });

    #[cfg(target_os = "linux")]
    let builder = builder.show_menu_on_left_click(true).icon_as_template(true);
    #[cfg(target_os = "windows")]
    let builder = builder;

    let _tray = builder
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("HYPERYAP")
        .build(app)?;

    Ok(())
}

/// Toggle the hotkey daemon: kill it to pause, relaunch to resume.
#[cfg(target_os = "windows")]
fn toggle_hotkeys_daemon(pause: bool) {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    if pause {
        let _ = Command::new("cmd")
            .args(["/C", "taskkill /F /IM hyperyap-hotkeys.exe >nul 2>&1"])
            .creation_flags(0x08000000)
            .status();
    } else {
        let hotkeys_path =
            std::path::PathBuf::from(std::env::var("LOCALAPPDATA").unwrap_or_default())
                .join("HyperYap")
                .join("hyperyap-hotkeys.exe");
        if hotkeys_path.exists() {
            let _ = Command::new(&hotkeys_path).arg("--no-tray").spawn();
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn toggle_hotkeys_daemon(_pause: bool) {}
