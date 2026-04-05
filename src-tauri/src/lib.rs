#![allow(clippy::module_inception)]

mod audio;
pub mod cli;
mod clipboard;
mod commands;
mod dictionary;
mod engine;
mod formatting_rules;
mod history;
mod http_api;
mod llm;
mod model;
mod onboarding;
mod overlay;
mod settings;
mod shortcuts;
mod stats;
mod utils;
mod wake_word;

use crate::shortcuts::init_shortcuts;
use audio::preload_engine;
use audio::types::AudioState;
use commands::*;
use dictionary::Dictionary;
use http_api::HttpApiState;
use log::{error, info, warn};
use model::Model;
use overlay::tray::setup_tray;
use std::str::FromStr;
use std::sync::Arc;
use tauri::{DeviceEventFilter, Listener, Manager};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_log::{Target, TargetKind};
use wake_word::types::WakeWordState;

#[cfg(target_os = "windows")]
fn deploy_hotkey_scripts(app: &tauri::AppHandle) {
    use std::fs;
    use std::os::windows::process::CommandExt;
    use std::path::PathBuf;

    let local_app_data = match std::env::var("LOCALAPPDATA") {
        Ok(v) => PathBuf::from(v),
        Err(_) => return,
    };
    let scripts_dir = local_app_data.join("HyperYap").join("scripts");
    let _ = fs::create_dir_all(&scripts_dir);

    // Find bundled scripts directory (Tauri NSIS puts them at _up_/presets/scripts/ relative to exe)
    let mut bundle_dir: Option<PathBuf> = None;
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let candidate = exe_dir.join("_up_").join("presets").join("scripts");
            if candidate.join("hyperyap-hotkeys.ahk").exists() {
                bundle_dir = Some(candidate);
            }
        }
    }

    // Fallback: Tauri resource resolver
    if bundle_dir.is_none() {
        for prefix in &["../presets/scripts", "presets/scripts"] {
            if let Ok(p) = app.path().resolve(prefix, tauri::path::BaseDirectory::Resource) {
                if p.join("hyperyap-hotkeys.ahk").exists() {
                    bundle_dir = Some(p);
                    break;
                }
            }
        }
    }

    let bundle_dir = match bundle_dir {
        Some(d) => d,
        None => {
            info!("Bundled hotkey scripts not found, skipping deployment");
            return;
        }
    };

    // Deploy scripts to scripts_dir
    for filename in &["hyperyap-hotkeys.ahk", "clipboard-image-paste.ps1"] {
        let source = bundle_dir.join(filename);
        if source.exists() {
            let _ = fs::copy(&source, scripts_dir.join(filename));
        }
    }
    info!("Deployed hotkey scripts to {}", scripts_dir.display());

    let target_ahk = scripts_dir.join("hyperyap-hotkeys.ahk");

    // Remove old murmure startup entries
    let app_data = match std::env::var("APPDATA") {
        Ok(v) => PathBuf::from(v),
        Err(_) => return,
    };
    let startup_dir = app_data.join("Microsoft").join("Windows").join("Start Menu")
        .join("Programs").join("Startup");
    for old_name in &["murmure-hotkeys.ahk", "murmure-hotkeys.lnk", "Murmure.lnk", "murmure.lnk"] {
        let old_path = startup_dir.join(old_name);
        if old_path.exists() {
            let _ = fs::remove_file(&old_path);
            info!("Removed old startup entry: {}", old_name);
        }
    }

    // Find AutoHotkey exe (check system install)
    let program_files = std::env::var("ProgramFiles").unwrap_or_default();
    let ahk_search = vec![
        PathBuf::from(&program_files).join("AutoHotkey").join("v2").join("AutoHotkey64.exe"),
        PathBuf::from(&program_files).join("AutoHotkey").join("v2").join("AutoHotkey32.exe"),
        PathBuf::from(std::env::var("ProgramFiles(x86)").unwrap_or_default())
            .join("AutoHotkey").join("v2").join("AutoHotkey64.exe"),
    ];
    let mut ahk_exe = ahk_search.iter().find(|p| p.exists()).cloned();

    // If AHK not installed, download and run the installer (shows GUI + UAC prompt)
    if ahk_exe.is_none() {
        info!("AutoHotkey v2 not found. Downloading and launching installer...");
        let ahk_installer = std::env::temp_dir().join("ahk-v2-setup.exe");
        // Download and run in a blocking thread so the script launches after install completes
        let ahk_installer_clone = ahk_installer.clone();
        let ahk_search_clone = ahk_search.clone();
        let target_ahk_clone = target_ahk.clone();
        let startup_link = startup_dir.join("hyperyap-hotkeys.lnk");
        let scripts_dir_clone = scripts_dir.clone();
        std::thread::spawn(move || {
            // Download AHK installer
            let dl_result = std::process::Command::new("powershell")
                .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command",
                    &format!("Invoke-WebRequest -Uri 'https://www.autohotkey.com/download/ahk-v2.exe' -OutFile '{}' -UseBasicParsing",
                        ahk_installer_clone.display())])
                .creation_flags(0x08000000)
                .status();

            if dl_result.is_err() || !ahk_installer_clone.exists() {
                error!("Failed to download AutoHotkey installer");
                return;
            }

            // Run installer with GUI (user sees it, approves UAC)
            info!("Launching AutoHotkey v2 installer...");
            let _ = std::process::Command::new(&ahk_installer_clone)
                .status(); // Blocking - waits for install to complete

            // Clean up installer
            let _ = std::fs::remove_file(&ahk_installer_clone);

            // Find the newly installed AHK exe
            let new_ahk = ahk_search_clone.iter().find(|p| p.exists());
            if let Some(ahk) = new_ahk {
                info!("AutoHotkey installed at {}", ahk.display());

                // Create startup shortcut
                let ps_cmd = format!(
                    "$s=(New-Object -ComObject WScript.Shell).CreateShortcut('{link}');\
                    $s.TargetPath='{ahk}';\
                    $s.Arguments='{script}';\
                    $s.WorkingDirectory='{workdir}';\
                    $s.Description='HyperYap Hotkeys';\
                    $s.Save()",
                    link = startup_link.display(),
                    ahk = ahk.display(),
                    script = target_ahk_clone.display(),
                    workdir = scripts_dir_clone.display()
                );
                let _ = std::process::Command::new("powershell")
                    .args(["-NoProfile", "-Command", &ps_cmd])
                    .creation_flags(0x08000000)
                    .status();

                // Launch hotkeys
                let _ = std::process::Command::new(ahk)
                    .arg(&target_ahk_clone)
                    .creation_flags(0x08000000)
                    .spawn();
                info!("Hotkey script launched after AHK install");
            } else {
                error!("AutoHotkey install completed but exe not found");
            }
        });
        return;
    }

    // AHK is installed. Create startup shortcut and launch.
    let ahk_exe = ahk_exe.unwrap();
    let startup_link = startup_dir.join("hyperyap-hotkeys.lnk");
    let ps_cmd = format!(
        "$s=(New-Object -ComObject WScript.Shell).CreateShortcut('{link}');\
        $s.TargetPath='{ahk}';\
        $s.Arguments='{script}';\
        $s.WorkingDirectory='{workdir}';\
        $s.Description='HyperYap Hotkeys';\
        $s.Save()",
        link = startup_link.display(),
        ahk = ahk_exe.display(),
        script = target_ahk.display(),
        workdir = scripts_dir.display()
    );
    let _ = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps_cmd])
        .creation_flags(0x08000000)
        .status();
    info!("Created startup shortcut: {}", startup_link.display());

    let _ = std::process::Command::new(&ahk_exe)
        .arg(&target_ahk)
        .creation_flags(0x08000000)
        .spawn();
    info!("Launched hotkey script with {}", ahk_exe.display());
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(main_window) = app.get_webview_window("main") {
        match main_window.show() {
            Ok(_) => (),
            Err(e) => error!("Failed to show window: {}", e),
        }
        match main_window.set_focus() {
            Ok(_) => (),
            Err(e) => error!("Failed to focus window: {}", e),
        }
    } else {
        warn!("Main window not found");
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::Webview),
                    Target::new(TargetKind::LogDir { file_name: None }),
                ])
                .max_file_size(1024 * 1024) // 1 MB, rotation
                .level(log::LevelFilter::Trace)
                .level_for("ort", log::LevelFilter::Warn)
                .level_for("ort::logging", log::LevelFilter::Warn)
                .level_for("zbus", log::LevelFilter::Warn)
                .level_for("tracing", log::LevelFilter::Warn)
                .level_for("symphonia_core", log::LevelFilter::Warn)
                .level_for("symphonia_bundle_mp3", log::LevelFilter::Warn)
                .level_for("enigo", log::LevelFilter::Info)
                .level_for("reqwest", log::LevelFilter::Info)
                .level_for("hyper_util", log::LevelFilter::Info)
                .level_for("tauri_plugin_updater", log::LevelFilter::Info)
                .level_for("arboard", log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if let Some(cli::CliCommand::Import {
                file_path,
                strategy,
            }) = cli::parse_raw_args(&args)
            {
                match cli::import::execute_import(app, &file_path, &strategy) {
                    Ok(msg) => {
                        info!("CLI import (hot-reload): {}", msg);
                        cli::import::apply_hot_reload_side_effects(app);
                    }
                    Err(msg) => {
                        error!("CLI import failed: {}", msg);
                    }
                }
            } else {
                show_main_window(app);
            }
        }))
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .arg("--autostart")
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_process::init())
        .device_event_filter(DeviceEventFilter::Never)
        .setup(|app| {
            let is_autostart = std::env::args().any(|arg| arg == "--autostart");
            if is_autostart {
                info!("Starting minimized to tray (autostart mode)");
                if let Some(main_window) = app.get_webview_window("main") {
                    let _ = main_window.hide();
                }
            }

            // Re-register autostart with --autostart flag for users who enabled it before this update
            if let Ok(true) = app.autolaunch().is_enabled() {
                let _ = app.autolaunch().enable();
            }

            // HyperYap: enable autostart by default on first launch
            if let Ok(false) = app.autolaunch().is_enabled() {
                let _ = app.autolaunch().enable();
                info!("Autostart enabled by default (first launch)");
            }

            // Deploy AHK hotkey scripts and startup shortcut (Windows only)
            #[cfg(target_os = "windows")]
            deploy_hotkey_scripts(app.handle());

            // Early CLI detection, before heavy initialization
            if let Some(cli::CliCommand::Import {
                file_path,
                strategy,
            }) = cli::parse_cli_matches(app.handle())
            {
                if let Some(main_window) = app.get_webview_window("main") {
                    let _ = main_window.hide();
                }
                match cli::import::execute_import(app.handle(), &file_path, &strategy) {
                    Ok(msg) => {
                        println!("{}", msg);
                        app.handle().exit(0);
                    }
                    Err(msg) => {
                        eprintln!("{}", msg);
                        app.handle().exit(1);
                    }
                }
                return Ok(());
            }

            let model =
                Arc::new(Model::new(app.handle().clone()).expect("Failed to initialize model"));
            app.manage(model);
            app.manage(AudioState::new());
            app.manage(WakeWordState::new());

            let mut s = settings::load_settings(app.handle());

            #[cfg(target_os = "macos")]
            {
                use tauri::ActivationPolicy;
                let policy = if s.show_in_dock {
                    ActivationPolicy::Regular
                } else {
                    ActivationPolicy::Accessory
                };
                app.set_activation_policy(policy);
            }

            if let Ok(level) = log::LevelFilter::from_str(&s.log_level) {
                log::set_max_level(level);
            }

            let dictionary = if !s.dictionary.is_empty() {
                let dictionary_from_settings = s.dictionary.clone();
                s = settings::remove_dictionary_from_settings(app.handle(), s)?;
                dictionary::migrate_and_load(app.handle(), dictionary_from_settings)?
            } else {
                dictionary::load(app.handle())?
            };
            app.manage(Dictionary::new(dictionary.clone()));
            app.manage(HttpApiState::new());

            match preload_engine(app.handle()) {
                Ok(_) => info!("Transcription engine initialized and ready"),
                Err(e) => info!("Transcription engine will be loaded on first use: {}", e),
            }

            setup_tray(app.handle())?;

            overlay::overlay::create_recording_overlay(app.handle());
            if s.overlay_mode.as_str() == "always" {
                overlay::overlay::show_recording_overlay(app.handle());
            }

            init_shortcuts(app.handle().clone());

            audio::sound::init_sound_system(app.handle());

            audio::microphone::init_mic_cache_if_needed(app.handle(), s.mic_id.clone());

            if s.api_enabled {
                let app_handle = app.handle().clone();
                let state = app_handle.state::<HttpApiState>().inner().clone();
                crate::http_api::spawn_http_api_thread(app_handle, s.api_port, state);
            }

            let app_handle = app.handle().clone();
            app.handle().listen("recording-limit-reached", move |_| {
                warn!("Recording limit reached, cancelling...");
                let app = app_handle.clone();
                std::thread::spawn(move || {
                    crate::shortcuts::force_cancel_recording(&app);
                });
            });

            if s.wake_word_enabled {
                let app_handle = app.handle().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    wake_word::start_listener(&app_handle);
                });
            }

            if !is_autostart {
                info!("Showing main window (manual launch)");
                show_main_window(app.handle());
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            is_model_available,
            get_model_path,
            read_hyperyap_file,
            write_hyperyap_file,
            get_all_settings,
            set_show_in_dock,
            get_dictionary_with_languages,
            get_recent_transcriptions,
            clear_history,
            get_record_shortcut,
            set_record_shortcut,
            set_dictionary,
            set_dictionary_with_languages,
            get_dictionary,
            export_dictionary,
            import_dictionary,
            get_last_transcript_shortcut,
            set_last_transcript_shortcut,
            get_llm_record_shortcut,
            set_llm_record_shortcut,
            get_command_shortcut,
            set_command_shortcut,
            get_cancel_shortcut,
            set_cancel_shortcut,
            cancel_recording,
            get_llm_mode_1_shortcut,
            set_llm_mode_1_shortcut,
            get_llm_mode_2_shortcut,
            set_llm_mode_2_shortcut,
            get_llm_mode_3_shortcut,
            set_llm_mode_3_shortcut,
            get_llm_mode_4_shortcut,
            set_llm_mode_4_shortcut,
            set_overlay_mode,
            set_overlay_position,
            suspend_transcription,
            resume_transcription,
            get_api_enabled,
            set_api_enabled,
            get_api_port,
            set_api_port,
            start_http_api_server,
            stop_http_api_server,
            set_copy_to_clipboard,
            set_paste_method,
            get_usage_stats,
            set_persist_history,
            get_current_language,
            set_current_language,
            get_current_mic_id,
            set_current_mic_id,
            get_current_mic_label,
            get_mic_list,
            get_onboarding_state,
            set_onboarding_used_home_shortcut,
            set_onboarding_transcribed_outside_app,
            set_onboarding_added_dictionary_word,
            set_onboarding_congrats_dismissed,
            get_llm_connect_settings,
            set_llm_connect_settings,
            test_llm_connection,
            fetch_ollama_models,
            pull_ollama_model,
            test_remote_connection,
            fetch_remote_models,
            store_remote_api_key,
            has_remote_api_key,
            get_remote_api_key_masked,
            set_sound_enabled,
            set_record_mode,
            get_formatting_settings,
            set_formatting_settings,
            validate_regex,
            set_log_level,
            open_accessibility_settings,
            check_accessibility_permission,
            get_wake_word_enabled,
            set_wake_word_enabled,
            get_wake_word_record,
            set_wake_word_record,
            get_llm_mode_wake_word,
            set_llm_mode_wake_word,
            get_wake_word_command,
            set_wake_word_command,
            get_wake_word_cancel,
            set_wake_word_cancel,
            get_wake_word_validate,
            set_wake_word_validate,
            get_auto_enter_after_wake_word,
            set_auto_enter_after_wake_word,
            get_silence_timeout_ms,
            set_silence_timeout_ms
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
