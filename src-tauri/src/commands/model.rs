use crate::audio::types::AudioState;
use crate::model::{Model, ModelInfo};
use std::sync::Arc;
use tauri::{AppHandle, State, command};

#[command]
pub fn is_model_available(model: State<Arc<Model>>) -> bool {
    model.is_available()
}

#[command]
pub fn get_model_path(model: State<Arc<Model>>) -> Result<String, String> {
    let path = model.get_model_path().map_err(|e| format!("{:#}", e))?;

    Ok(path.to_string_lossy().to_string())
}

#[command]
pub fn get_asr_model(model: State<Arc<Model>>) -> String {
    model.selected_model_id()
}

#[command]
pub fn list_asr_models(model: State<Arc<Model>>) -> Vec<ModelInfo> {
    model.list_available_models()
}

#[command]
pub fn set_asr_model(
    app: AppHandle,
    audio_state: State<AudioState>,
    model: State<Arc<Model>>,
    model_id: String,
) -> Result<(), String> {
    if audio_state.recorder.lock().is_some() {
        return Err("Cannot switch ASR models while recording".to_string());
    }

    let model_id = model_id.trim().to_string();
    if model_id.is_empty() {
        return Err("ASR model id cannot be empty".to_string());
    }

    let model_path = model
        .resolve_model_path_for_id(&model_id)
        .ok_or_else(|| format!("ASR model '{}' is not installed or is incomplete", model_id))?;

    let mut settings = crate::settings::load_settings(&app);
    settings.asr_model = model_id.clone();
    crate::settings::save_settings(&app, &settings)?;

    *audio_state.engine.lock() = None;
    log::info!(
        "ASR model changed to '{}' at {}; cached engine cleared",
        model_id,
        model_path.display()
    );

    Ok(())
}
