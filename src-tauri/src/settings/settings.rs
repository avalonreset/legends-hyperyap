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
    // Try the resource resolver first (looks in resources/ subdirs)
    if let Some(preset_path) =
        crate::utils::resources::resolve_resource_path(app, "settings.json")
    {
        if let Ok(content) = fs::read_to_string(&preset_path) {
            if let Ok(settings) = serde_json::from_str::<AppSettings>(&content) {
                info!("Loaded bundled HyperYap preset settings");
                return settings;
            }
        }
    }
    // Fallback: check resource root directly (Tauri places individual resource files here)
    if let Ok(preset_path) = app
        .path()
        .resolve("settings.json", tauri::path::BaseDirectory::Resource)
    {
        if preset_path.exists() {
            if let Ok(content) = fs::read_to_string(&preset_path) {
                if let Ok(settings) = serde_json::from_str::<AppSettings>(&content) {
                    info!(
                        "Loaded bundled HyperYap preset from resource root: {}",
                        preset_path.display()
                    );
                    return settings;
                }
            }
        }
    }
    info!("Bundled preset not found, using defaults");
    AppSettings::default()
}

pub fn load_settings(app: &AppHandle) -> AppSettings {
    let path = match settings_path(app) {
        Ok(p) => p,
        Err(_) => return AppSettings::default(),
    };

    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str::<AppSettings>(&content).unwrap_or_default(),
        Err(_) => {
            let defaults = load_bundled_preset(app);
            let _ = save_settings(app, &defaults);
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
