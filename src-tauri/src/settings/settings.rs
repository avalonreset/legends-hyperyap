use log::info;
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager};

use super::types::AppSettings;

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    if let Err(e) = fs::create_dir_all(&dir) {
        return Err(format!("create_dir_all failed: {}", e));
    }
    Ok(dir.join("settings.json"))
}

fn load_bundled_preset(app: &AppHandle) -> AppSettings {
    // Tauri bundles ../presets/settings.json into _up_/presets/settings.json
    let search_paths = vec![
        app.path().resolve(
            "../presets/settings.json",
            tauri::path::BaseDirectory::Resource,
        ),
        app.path().resolve(
            "presets/settings.json",
            tauri::path::BaseDirectory::Resource,
        ),
        app.path().resolve(
            "../resources/settings.json",
            tauri::path::BaseDirectory::Resource,
        ),
        app.path().resolve(
            "settings.json",
            tauri::path::BaseDirectory::Resource,
        ),
    ];

    for path_result in search_paths {
        if let Ok(preset_path) = path_result {
            if preset_path.exists() {
                if let Ok(content) = fs::read_to_string(&preset_path) {
                    if let Ok(settings) = serde_json::from_str::<AppSettings>(&content) {
                        info!(
                            "Loaded bundled HyperYap preset from: {}",
                            preset_path.display()
                        );
                        return settings;
                    }
                }
            }
        }
    }

    info!("Bundled preset not found, using defaults");
    AppSettings::default()
}

fn version_marker_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(dir.join(".settings-version"))
}

fn should_deploy_presets(app: &AppHandle) -> bool {
    let current_version = env!("CARGO_PKG_VERSION");
    let marker_path = match version_marker_path(app) {
        Ok(p) => p,
        Err(_) => return true,
    };
    match fs::read_to_string(&marker_path) {
        Ok(stored_version) => stored_version.trim() != current_version,
        Err(_) => true,
    }
}

fn mark_presets_deployed(app: &AppHandle) {
    let current_version = env!("CARGO_PKG_VERSION");
    if let Ok(marker_path) = version_marker_path(app) {
        let _ = fs::write(marker_path, current_version);
    }
}

pub fn load_settings(app: &AppHandle) -> AppSettings {
    let path = match settings_path(app) {
        Ok(p) => p,
        Err(_) => return AppSettings::default(),
    };

    if should_deploy_presets(app) {
        info!("New install or upgrade detected, deploying HyperYap preset settings");
        let presets = load_bundled_preset(app);
        let _ = save_settings(app, &presets);
        mark_presets_deployed(app);
        return presets;
    }

    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str::<AppSettings>(&content).unwrap_or_default(),
        Err(_) => {
            let defaults = load_bundled_preset(app);
            let _ = save_settings(app, &defaults);
            mark_presets_deployed(app);
            defaults
        }
    }
}

pub fn save_settings(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    let path = settings_path(app)?;
    let content = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())
}

pub fn remove_dictionary_from_settings(
    app: &AppHandle,
    mut settings: AppSettings,
) -> Result<AppSettings, String> {
    settings.dictionary = Vec::new();
    save_settings(app, &settings)?;
    Ok(settings)
}
