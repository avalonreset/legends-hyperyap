use crate::audio::helpers::read_wav_samples;
use crate::audio::types::{AudioState, RecordingMode};
use crate::dictionary::{Dictionary, fix_transcription_with_dictionary, get_cc_rules_path};
use crate::engine::ParakeetModelParams;
use crate::engine::transcription_engine::TranscriptionEngine;
use crate::formatting_rules;
use crate::history;
use crate::model::Model;
use crate::stats;
use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use serde_json::json;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};

const EVAL_CAPTURE_DIR_ENV: &str = "HYPERYAP_EVAL_CAPTURE_DIR";

pub fn process_recording(app: &AppHandle, file_path: &Path) -> Result<String> {
    // 1. Transcribe
    let raw_text = transcribe_audio(app, file_path)?;
    debug!("Raw transcription: {}", raw_text);

    if raw_text.trim().is_empty() {
        debug!("Transcription is empty, skipping further processing.");
        capture_eval_sample(app, file_path, &raw_text, &raw_text);
        return Ok(raw_text);
    }

    // 2. Deduplicate repeated words (transcription artifact cleanup)
    let text = deduplicate_repeated_words(&raw_text);

    // 3. Dictionary & CC Rules
    let text = apply_dictionary_and_rules(app, text)?;
    debug!("Transcription fixed with dictionary: {}", text);

    // 4. LLM Post-processing
    let llm_text = apply_llm_processing(app, text)?;

    // 5. Apply formatting rules
    let final_text = apply_formatting_rules(app, llm_text);
    debug!("Transcription with formatting rules: {}", final_text);

    capture_eval_sample(app, file_path, &raw_text, &final_text);

    // 6. Save Stats & History
    save_stats_and_history(app, file_path, &final_text)?;

    Ok(final_text)
}

fn capture_eval_sample(app: &AppHandle, file_path: &Path, raw_text: &str, final_text: &str) {
    let Ok(capture_dir) = std::env::var(EVAL_CAPTURE_DIR_ENV) else {
        return;
    };

    let capture_dir = capture_dir.trim();
    if capture_dir.is_empty() {
        return;
    }

    if let Err(e) = capture_eval_sample_inner(app, file_path, raw_text, final_text, capture_dir) {
        warn!("Failed to capture eval sample: {}", e);
    }
}

fn capture_eval_sample_inner(
    app: &AppHandle,
    file_path: &Path,
    raw_text: &str,
    final_text: &str,
    capture_dir: &str,
) -> Result<()> {
    let capture_root = Path::new(capture_dir);
    std::fs::create_dir_all(capture_root).with_context(|| {
        format!(
            "failed to create eval capture dir {}",
            capture_root.display()
        )
    })?;

    let timestamp = chrono::Utc::now().to_rfc3339();
    let id = format!("hyperyap-real-{}", uuid::Uuid::new_v4().simple());
    let audio_name = format!("{id}.wav");
    let captured_audio = capture_root.join(&audio_name);
    std::fs::copy(file_path, &captured_audio).with_context(|| {
        format!(
            "failed to copy eval audio {} to {}",
            file_path.display(),
            captured_audio.display()
        )
    })?;

    let model = app.state::<Arc<Model>>();
    let model_id = model.selected_model_id();
    let model_path = model
        .get_model_path()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let mode = app.state::<AudioState>().get_recording_mode();

    let payload = json!({
        "id": id,
        "audio": captured_audio.to_string_lossy(),
        "reference": "",
        "raw_transcript": raw_text,
        "final_transcript": final_text,
        "model_id": model_id,
        "model_path": model_path,
        "recording_mode": format!("{mode:?}"),
        "captured_at": timestamp
    });

    let manifest_path = capture_root.join("capture-manifest.template.jsonl");
    let mut manifest = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&manifest_path)
        .with_context(|| format!("failed to open {}", manifest_path.display()))?;
    writeln!(manifest, "{}", serde_json::to_string(&payload)?)?;

    info!(
        "Captured eval sample '{}' to {}",
        payload["id"].as_str().unwrap_or_default(),
        capture_root.display()
    );

    Ok(())
}

pub fn transcribe_audio(app: &AppHandle, audio_path: &Path) -> Result<String> {
    let _ = app.emit("llm-processing-start", ());

    let state = app.state::<AudioState>();

    // Ensure engine is loaded
    {
        let mut engine_guard = state.engine.lock();
        if engine_guard.is_none() {
            let model = app.state::<Arc<Model>>();
            let model_path = model
                .get_model_path()
                .map_err(|e| anyhow::anyhow!("Failed to get model path: {}", e))?;

            let mut new_engine = crate::engine::ParakeetEngine::new();
            new_engine
                .load_model_with_params(&model_path, ParakeetModelParams::int8())
                .map_err(|e| anyhow::anyhow!("Failed to load model: {}", e))?;

            *engine_guard = Some(new_engine);
            info!("Model loaded and cached in memory");
        }
    }

    let samples = read_wav_samples(audio_path)?;

    let mut engine_guard = state.engine.lock();
    let engine = engine_guard
        .as_mut()
        .ok_or_else(|| anyhow::anyhow!("Engine not loaded"))?;

    let result = engine.transcribe_samples(samples, None).map_err(|e| {
        let _ = app.emit("llm-processing-end", ());
        anyhow::anyhow!("Transcription failed: {}", e)
    })?;
    let _ = app.emit("llm-processing-end", ());

    Ok(result.text)
}

fn apply_dictionary_and_rules(app: &AppHandle, text: String) -> Result<String> {
    let cc_rules_path = get_cc_rules_path(app).context("Failed to get CC rules path")?;
    let dictionary = app.state::<Dictionary>().get();

    Ok(fix_transcription_with_dictionary(
        text,
        dictionary,
        cc_rules_path,
    ))
}

fn apply_llm_processing(app: &AppHandle, text: String) -> Result<String> {
    let state = app.state::<AudioState>();
    let recording_mode = state.get_recording_mode();

    let rt = tokio::runtime::Runtime::new().context("Failed to create tokio runtime")?;

    match recording_mode {
        RecordingMode::Command => {
            debug!("Processing audio in Command mode");

            let selected_text = match crate::clipboard::get_selected_text(app) {
                Ok(s) if !s.trim().is_empty() => {
                    debug!("Captured selected text for command mode successfully");
                    Some(s)
                }
                Ok(_) => {
                    warn!("Selected text was empty in command mode");
                    None
                }
                Err(e) => {
                    error!("Failed to capture selected text in command mode: {}", e);
                    None
                }
            };

            let system_prompt = format!(
                r#"You are a text transformation tool, not a conversational assistant.
Your ONLY job: apply the user instruction to the input text and return the result.
DO NOT explain, comment, or add any text beyond the transformation output.

Rules:
- Return ONLY the transformed text
- NO explanations, NO commentary, NO markdown formatting
- If the instruction is unclear or cannot be applied: return the input text UNCHANGED
- Never wrap the output in quotes, code blocks, or additional formatting

User instruction: {}"#,
                text
            );

            let user_prompt = match &selected_text {
                Some(s) => s.clone(),
                None => text.clone(),
            };

            match rt.block_on(crate::llm::process_command_with_llm(
                app,
                system_prompt,
                user_prompt,
            )) {
                Ok(response) => {
                    debug!("Command processed with LLM: {}", response);
                    Ok(response)
                }
                Err(e) => {
                    warn!(
                        "Command LLM processing failed: {}. Using original transcription.",
                        e
                    );
                    let _ = app.emit("llm-error", e.to_string());
                    Ok(text)
                }
            }
        }
        RecordingMode::Llm => {
            match rt.block_on(crate::llm::post_process_with_llm(
                app,
                text.clone(),
                false, // force_bypass
            )) {
                Ok(llm_text) => {
                    debug!("Transcription post-processed with LLM: {}", llm_text);
                    Ok(llm_text)
                }
                Err(e) => {
                    warn!(
                        "LLM post-processing failed: {}. Using original transcription.",
                        e
                    );
                    let _ = app.emit("llm-error", e.to_string());
                    Ok(text)
                }
            }
        }
        RecordingMode::Standard => {
            // Standard mode bypasses LLM processing
            Ok(text)
        }
    }
}

fn apply_formatting_rules(app: &AppHandle, text: String) -> String {
    match formatting_rules::load(app) {
        Ok(settings) => formatting_rules::apply_formatting(text, &settings),
        Err(e) => {
            warn!("Failed to load formatting rules: {}. Skipping.", e);
            text
        }
    }
}

fn deduplicate_repeated_words(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return String::new();
    }

    let mut result: Vec<&str> = Vec::with_capacity(words.len());
    let mut i = 0;

    while i < words.len() {
        let current_lower = words[i].to_lowercase();
        let mut count = 1;

        while i + count < words.len() && words[i + count].to_lowercase() == current_lower {
            count += 1;
        }

        if count >= 3 {
            result.push(words[i]);
            result.push(words[i + 1]);
        } else {
            for j in 0..count {
                result.push(words[i + j]);
            }
        }

        i += count;
    }

    result.join(" ")
}

fn save_stats_and_history(app: &AppHandle, file_path: &Path, text: &str) -> Result<()> {
    // Calculate duration and size
    let (duration_seconds, wav_size_bytes) = match hound::WavReader::open(file_path) {
        Ok(reader) => {
            let spec = reader.spec();
            let total_samples = reader.duration() as f64;
            let seconds = if spec.sample_rate > 0 {
                total_samples / (spec.sample_rate as f64)
            } else {
                0.0
            };
            let size = std::fs::metadata(file_path).map(|m| m.len()).unwrap_or(0);
            (seconds, size)
        }
        Err(_) => (0.0, 0),
    };

    let word_count: u64 = text.split_whitespace().filter(|s| !s.is_empty()).count() as u64;

    if let Err(e) = history::add_transcription(app, text.to_string()) {
        error!("Failed to save to history: {}", e);
    }

    if let Err(e) =
        stats::add_transcription_session(app, word_count, duration_seconds, wav_size_bytes)
    {
        error!("Failed to save stats session: {}", e);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedup_four_to_two() {
        assert_eq!(deduplicate_repeated_words("je je je je vais"), "je je vais");
    }

    #[test]
    fn dedup_two_kept_unchanged() {
        assert_eq!(deduplicate_repeated_words("oui oui"), "oui oui");
    }

    #[test]
    fn dedup_five_to_two() {
        assert_eq!(
            deduplicate_repeated_words("the the the the the cat"),
            "the the cat"
        );
    }

    #[test]
    fn dedup_three_to_two_case_insensitive() {
        assert_eq!(
            deduplicate_repeated_words("Hello HELLO hello world"),
            "Hello HELLO world"
        );
    }

    #[test]
    fn dedup_no_repetition() {
        assert_eq!(
            deduplicate_repeated_words("normal sentence"),
            "normal sentence"
        );
    }

    #[test]
    fn dedup_empty_string() {
        assert_eq!(deduplicate_repeated_words(""), "");
    }

    #[test]
    fn dedup_single_word() {
        assert_eq!(deduplicate_repeated_words("word"), "word");
    }

    #[test]
    fn dedup_multiple_groups() {
        assert_eq!(
            deduplicate_repeated_words("the the the cat the the the dog"),
            "the the cat the the dog"
        );
    }

    #[test]
    fn dedup_exactly_three_to_two() {
        assert_eq!(
            deduplicate_repeated_words("hello hello hello world"),
            "hello hello world"
        );
    }

    #[test]
    fn dedup_one_occurrence_unchanged() {
        assert_eq!(deduplicate_repeated_words("hello world"), "hello world");
    }
}
