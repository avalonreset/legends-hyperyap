use anyhow::Result;
use log::{debug, warn};
use serde::Serialize;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

pub const DEFAULT_MODEL_ID: &str = "parakeet-tdt-0.6b-v2-smcleod-int8";
pub const UNIFIED_EN_SMOOTHQUANT_PILOT_MODEL_ID: &str =
    "parakeet-unified-en-0.6b-smoothquant-pilot-int8";

const ENV_MODEL_ID: &str = "HYPERYAP_ASR_MODEL_ID";
const ENV_MODEL_DIR: &str = "HYPERYAP_ASR_MODEL_DIR";
const REQUIRED_MODEL_FILES: &[&str] = &[
    "encoder-model.int8.onnx",
    "decoder_joint-model.int8.onnx",
    "nemo128.onnx",
    "vocab.txt",
];

#[derive(Debug, Clone, Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub path: String,
    pub source: String,
    pub selected: bool,
    pub available: bool,
}

pub struct Model {
    app_handle: AppHandle,
}

impl Model {
    pub fn new(app_handle: AppHandle) -> Result<Self> {
        Ok(Self { app_handle })
    }

    pub fn selected_model_id(&self) -> String {
        if let Ok(model_id) = std::env::var(ENV_MODEL_ID) {
            let model_id = model_id.trim();
            if !model_id.is_empty() {
                return model_id.to_string();
            }
        }

        let model_id = crate::settings::load_settings(&self.app_handle).asr_model;
        if model_id.trim().is_empty() {
            DEFAULT_MODEL_ID.to_string()
        } else {
            model_id
        }
    }

    pub fn get_model_path(&self) -> Result<PathBuf> {
        if let Some(model_path) = self.env_model_dir()? {
            debug!(
                "Using model override from {ENV_MODEL_DIR}: {}",
                model_path.display()
            );
            return Ok(model_path);
        }

        let selected_model_id = self.selected_model_id();
        let mut candidate_ids = vec![selected_model_id.clone()];
        if selected_model_id != DEFAULT_MODEL_ID {
            candidate_ids.push(DEFAULT_MODEL_ID.to_string());
        }

        let mut attempted = Vec::new();
        for model_id in candidate_ids {
            match self.resolve_model_path_for_id(&model_id) {
                Some(model_path) => {
                    if model_id != selected_model_id {
                        warn!(
                            "Selected ASR model '{}' was unavailable; falling back to '{}'",
                            selected_model_id, model_id
                        );
                    }
                    debug!("Model '{}' found at: {}", model_id, model_path.display());
                    return Ok(model_path);
                }
                None => attempted.push(model_id),
            }
        }

        anyhow::bail!(
            "ASR model '{}' not found or incomplete. Attempted: {}. \
            Expected files in each model folder: {}.",
            selected_model_id,
            attempted.join(", "),
            REQUIRED_MODEL_FILES.join(", ")
        )
    }

    pub fn resolve_model_path_for_id(&self, model_id: &str) -> Option<PathBuf> {
        let model_id = model_id.trim();
        if model_id.is_empty() || model_id.contains("..") || model_id.contains(['/', '\\']) {
            return None;
        }

        for root in self.model_roots() {
            let candidate = root.join(model_id);
            if is_valid_model_dir(&candidate) {
                return Some(candidate);
            }
        }

        if let Some(model_path) =
            crate::utils::resources::resolve_resource_path(&self.app_handle, model_id)
        {
            if is_valid_model_dir(&model_path) {
                return Some(model_path);
            }
        }

        None
    }

    pub fn list_available_models(&self) -> Vec<ModelInfo> {
        let selected = self.selected_model_id();
        let mut models = BTreeMap::<String, ModelInfo>::new();

        for root in self.model_roots() {
            let source = root.display().to_string();
            if let Ok(entries) = fs::read_dir(&root) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !is_valid_model_dir(&path) {
                        continue;
                    }

                    let id = entry.file_name().to_string_lossy().to_string();
                    models.entry(id.clone()).or_insert_with(|| ModelInfo {
                        name: display_name_for_model(&id),
                        selected: id == selected,
                        available: true,
                        source: source.clone(),
                        path: path.to_string_lossy().to_string(),
                        id,
                    });
                }
            }
        }

        for id in [DEFAULT_MODEL_ID, UNIFIED_EN_SMOOTHQUANT_PILOT_MODEL_ID] {
            models.entry(id.to_string()).or_insert_with(|| ModelInfo {
                id: id.to_string(),
                name: display_name_for_model(id),
                path: String::new(),
                source: "expected".to_string(),
                selected: id == selected,
                available: false,
            });
        }

        models.into_values().collect()
    }

    pub fn is_available(&self) -> bool {
        self.get_model_path().is_ok()
    }

    fn env_model_dir(&self) -> Result<Option<PathBuf>> {
        let Ok(model_dir) = std::env::var(ENV_MODEL_DIR) else {
            return Ok(None);
        };

        let model_dir = model_dir.trim();
        if model_dir.is_empty() {
            return Ok(None);
        }

        let path = PathBuf::from(model_dir);
        if is_valid_model_dir(&path) {
            Ok(Some(path))
        } else {
            anyhow::bail!(
                "{} points at an incomplete ASR model directory: {}. Expected files: {}",
                ENV_MODEL_DIR,
                path.display(),
                REQUIRED_MODEL_FILES.join(", ")
            )
        }
    }

    fn model_roots(&self) -> Vec<PathBuf> {
        let mut roots = Vec::new();

        for relative_path in ["../resources", "_up_/resources", "resources"] {
            if let Ok(path) = self
                .app_handle
                .path()
                .resolve(relative_path, tauri::path::BaseDirectory::Resource)
            {
                roots.push(path);
            }
        }

        if let Ok(app_data_dir) = self.app_handle.path().app_data_dir() {
            roots.push(app_data_dir.join("resources"));
        }

        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                roots.push(exe_dir.join("_up_").join("resources"));
            }
        }

        dedupe_paths(roots)
    }
}

pub fn is_valid_model_dir(path: &Path) -> bool {
    path.is_dir()
        && REQUIRED_MODEL_FILES
            .iter()
            .all(|file_name| path.join(file_name).is_file())
}

fn display_name_for_model(id: &str) -> String {
    match id {
        DEFAULT_MODEL_ID => "Parakeet TDT 0.6B v2 INT8 (English)".to_string(),
        UNIFIED_EN_SMOOTHQUANT_PILOT_MODEL_ID => {
            "Parakeet Unified EN 0.6B SmoothQuant INT8".to_string()
        }
        other => other
            .split(['-', '_'])
            .filter(|part| !part.is_empty())
            .map(|part| {
                let mut chars = part.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" "),
    }
}

fn dedupe_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    let mut deduped = Vec::new();

    for path in paths {
        let key = path
            .canonicalize()
            .unwrap_or_else(|_| path.clone())
            .to_string_lossy()
            .to_lowercase();
        if seen.insert(key) {
            deduped.push(path);
        }
    }

    deduped
}
