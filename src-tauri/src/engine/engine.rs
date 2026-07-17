use ndarray::{Array, Array1, Array2, ArrayD, ArrayViewD, IxDyn};
use once_cell::sync::Lazy;
use ort::execution_providers::CPUExecutionProvider;
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
use ort::execution_providers::CUDAExecutionProvider;
use ort::execution_providers::ExecutionProviderDispatch;
use ort::inputs;
use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::TensorRef;
use regex::Regex;

use std::fs;
use std::path::Path;

use super::types::{DecoderState, ParakeetError, ParakeetModel, TimestampedResult};

const SUBSAMPLING_FACTOR: usize = 8;
const WINDOW_SIZE: f32 = 0.01;
const MAX_TOKENS_PER_STEP: usize = 10;

static DECODE_SPACE_RE: Lazy<Result<Regex, regex::Error>> =
    Lazy::new(|| Regex::new(r"\A\s|\s\B|(\s)\b"));

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
fn preferred_execution_provider_names() -> [&'static str; 2] {
    ["CUDAExecutionProvider", "CPUExecutionProvider"]
}

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
fn find_cudnn_runtime_dir(program_files: &std::path::Path) -> Option<std::path::PathBuf> {
    let cudnn_root = program_files.join("NVIDIA").join("CUDNN");
    let mut version_dirs = std::fs::read_dir(cudnn_root)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    version_dirs.sort_by(|left, right| right.cmp(left));

    for version_dir in version_dirs {
        let bin_dir = version_dir.join("bin");
        let mut runtime_dirs = vec![bin_dir.clone()];
        if let Ok(entries) = std::fs::read_dir(&bin_dir) {
            runtime_dirs.extend(
                entries
                    .filter_map(Result::ok)
                    .map(|entry| entry.path())
                    .filter(|path| path.is_dir()),
            );
        }
        runtime_dirs.sort_by(|left, right| right.cmp(left));

        if let Some(runtime_dir) = runtime_dirs
            .into_iter()
            .find(|path| path.join("cudnn64_9.dll").is_file())
        {
            return Some(runtime_dir);
        }
    }

    None
}

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
fn find_packaged_cuda_provider_dir(executable_dir: &std::path::Path) -> Option<std::path::PathBuf> {
    [
        executable_dir.to_path_buf(),
        executable_dir.join("target").join("release"),
        executable_dir.join("_up_").join("target").join("release"),
        executable_dir.join("runtime").join("windows-x64"),
        executable_dir
            .join("_up_")
            .join("runtime")
            .join("windows-x64"),
    ]
    .into_iter()
    .find(|path| {
        path.join("onnxruntime_providers_cuda.dll").is_file()
            && path.join("onnxruntime_providers_shared.dll").is_file()
    })
}

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
fn ensure_cuda_runtime_on_path() {
    static INIT: std::sync::Once = std::sync::Once::new();

    INIT.call_once(|| {
        let packaged_provider_dir = std::env::current_exe()
            .ok()
            .and_then(|executable| executable.parent().map(std::path::Path::to_path_buf))
            .and_then(|executable_dir| find_packaged_cuda_provider_dir(&executable_dir));

        let cudnn_runtime_dir = ["ProgramW6432", "ProgramFiles"]
            .into_iter()
            .filter_map(std::env::var_os)
            .map(std::path::PathBuf::from)
            .find_map(|program_files| find_cudnn_runtime_dir(&program_files));

        if packaged_provider_dir.is_none() {
            log::warn!("Packaged ONNX Runtime CUDA provider directory was not found");
        }
        if cudnn_runtime_dir.is_none() {
            log::warn!("cuDNN 9 runtime directory was not found under Program Files");
        }

        let current_path = std::env::var_os("PATH").unwrap_or_default();
        let mut path_entries = std::env::split_paths(&current_path).collect::<Vec<_>>();
        let mut added_runtime_dirs = Vec::new();

        for runtime_dir in packaged_provider_dir.into_iter().chain(cudnn_runtime_dir) {
            let runtime_text = runtime_dir.to_string_lossy();
            if path_entries
                .iter()
                .any(|path| path.to_string_lossy().eq_ignore_ascii_case(&runtime_text))
            {
                continue;
            }

            path_entries.insert(0, runtime_dir.clone());
            added_runtime_dirs.push(runtime_dir);
        }

        match std::env::join_paths(path_entries) {
            Ok(path) => {
                std::env::set_var("PATH", path);
                for runtime_dir in added_runtime_dirs {
                    log::info!(
                        "Added CUDA runtime directory to the HyperYap process path: {}",
                        runtime_dir.display()
                    );
                }
            }
            Err(error) => log::warn!("Failed to add CUDA runtime directories to PATH: {error}"),
        }
    });
}

#[cfg(not(all(target_os = "windows", target_arch = "x86_64")))]
fn preferred_execution_provider_names() -> [&'static str; 1] {
    ["CPUExecutionProvider"]
}

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
fn preferred_execution_providers() -> Vec<ExecutionProviderDispatch> {
    ensure_cuda_runtime_on_path();
    vec![
        CUDAExecutionProvider::default()
            .with_device_id(0)
            .build()
            // NVIDIA acceleration is preferred, but HyperYap must still start on
            // Windows machines without a compatible CUDA/cuDNN installation.
            .fail_silently(),
        CPUExecutionProvider::default().build(),
    ]
}

#[cfg(not(all(target_os = "windows", target_arch = "x86_64")))]
fn preferred_execution_providers() -> Vec<ExecutionProviderDispatch> {
    vec![CPUExecutionProvider::default().build()]
}

fn decode_wordpiece_tokens(tokens: &[String]) -> String {
    let mut text = String::new();
    let mut pending_space = false;

    for token in tokens {
        if token == "\u{2581}" {
            pending_space = true;
            continue;
        }

        let token = token.replace('\u{2581}', "");
        if token.is_empty() || token == "<unk>" || (token.starts_with('<') && token.ends_with('>'))
        {
            continue;
        }

        if let Some(piece) = token.strip_prefix("##") {
            if pending_space && !text.is_empty() {
                text.push(' ');
            }
            text.push_str(piece);
        } else {
            if !text.is_empty() && !is_attach_left(&token) {
                text.push(' ');
            }
            text.push_str(&token);
        }
        pending_space = false;
    }

    text
}

fn is_attach_left(token: &str) -> bool {
    matches!(
        token,
        "." | "," | "!" | "?" | ":" | ";" | ")" | "]" | "}" | "'" | "\"" | "%"
    )
}

impl Drop for ParakeetModel {
    fn drop(&mut self) {
        log::debug!(
            "Dropping ParakeetModel with {} vocab tokens",
            self.vocab.len()
        );
    }
}

impl ParakeetModel {
    pub fn new<P: AsRef<Path>>(model_dir: P, quantized: bool) -> Result<Self, ParakeetError> {
        let encoder = Self::init_session(&model_dir, "encoder-model", None, quantized)?;
        let decoder_joint = Self::init_session(&model_dir, "decoder_joint-model", None, quantized)?;
        let preprocessor = Self::init_session(&model_dir, "nemo128", None, false)?;

        let (vocab, blank_idx) = Self::load_vocab(&model_dir)?;
        let vocab_size = vocab.len();

        log::trace!(
            "Loaded vocabulary with {} tokens, blank_idx={}",
            vocab_size,
            blank_idx
        );

        Ok(Self {
            encoder,
            decoder_joint,
            preprocessor,
            vocab,
            blank_idx,
            vocab_size,
        })
    }

    fn init_session<P: AsRef<Path>>(
        model_dir: P,
        model_name: &str,
        intra_threads: Option<usize>,
        try_quantized: bool,
    ) -> Result<Session, ParakeetError> {
        let providers = preferred_execution_providers();
        log::info!(
            "Loading ONNX model '{}' with execution providers: {}",
            model_name,
            preferred_execution_provider_names().join(" -> ")
        );

        // Try quantized version first if requested, fallback to regular version
        let model_filename = if try_quantized {
            let quantized_name = format!("{}.int8.onnx", model_name);
            let quantized_path = model_dir.as_ref().join(&quantized_name);
            if quantized_path.exists() {
                log::trace!("Loading quantized model from {}...", quantized_name);
                quantized_name
            } else {
                let regular_name = format!("{}.onnx", model_name);
                log::trace!(
                    "Quantized model not found, loading regular model from {}...",
                    regular_name
                );
                regular_name
            }
        } else {
            let regular_name = format!("{}.onnx", model_name);
            log::trace!("Loading model from {}...", regular_name);
            regular_name
        };

        let mut builder = Session::builder()?
            .with_config_entry("session.log_severity_level", "3")?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_execution_providers(providers)?
            .with_memory_pattern(false)?
            .with_parallel_execution(false)?;

        if let Some(threads) = intra_threads {
            builder = builder
                .with_intra_threads(threads)?
                .with_inter_threads(threads)?;
        }

        let session = builder.commit_from_file(model_dir.as_ref().join(&model_filename))?;

        for input in &session.inputs {
            log::trace!(
                "Model '{}' input: name={}, type={:?}",
                model_filename,
                input.name,
                input.input_type
            );
        }

        Ok(session)
    }

    fn load_vocab<P: AsRef<Path>>(model_dir: P) -> Result<(Vec<String>, i32), ParakeetError> {
        let vocab_path = model_dir.as_ref().join("vocab.txt");
        let content = fs::read_to_string(vocab_path)?;

        let mut max_id = 0;
        let mut tokens_with_ids: Vec<(String, usize)> = Vec::new();
        let mut blank_idx: Option<usize> = None;
        let mut plain_tokens: Vec<String> = Vec::new();
        let mut saw_indexed_vocab = false;

        for line in content.lines() {
            let trimmed = line.trim_end();
            if trimmed.is_empty() {
                continue;
            }

            let parts: Vec<&str> = trimmed.split(' ').collect();
            if parts.len() >= 2 {
                let token = parts[0].to_string();
                if let Ok(id) = parts[1].parse::<usize>() {
                    saw_indexed_vocab = true;
                    if token == "<blk>" {
                        blank_idx = Some(id);
                    }
                    tokens_with_ids.push((token, id));
                    max_id = max_id.max(id);
                    continue;
                }
            }

            plain_tokens.push(trimmed.to_string());
        }

        if saw_indexed_vocab {
            // Create vocab vector with \u2581 replaced with space
            let mut vocab = vec![String::new(); max_id + 1];
            for (token, id) in tokens_with_ids {
                vocab[id] = token.replace('\u{2581}', " ");
            }

            let blank_idx = blank_idx.ok_or_else(|| {
                ParakeetError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Missing <blk> token in vocabulary",
                ))
            })? as i32;

            Ok((vocab, blank_idx))
        } else {
            // Some Unified EN ONNX exports ship tokenizer.model plus a plain ordered vocab.txt
            // that omits the SentencePiece <unk> entry. Reinsert it so token ids match the
            // tokenizer/model space, and use the next id as the RNNT blank.
            let mut vocab = Vec::with_capacity(plain_tokens.len() + 1);
            vocab.push("<unk>".to_string());
            vocab.extend(plain_tokens);
            let blank_idx = vocab.len() as i32;
            Ok((vocab, blank_idx))
        }
    }

    pub fn preprocess(
        &mut self,
        waveforms: &ArrayViewD<f32>,
        waveforms_lens: &ArrayViewD<i64>,
    ) -> Result<(ArrayD<f32>, ArrayD<i64>), ParakeetError> {
        log::trace!("Running preprocessor inference...");
        let inputs = inputs![
            "waveforms" => TensorRef::from_array_view(waveforms.view())?,
            "waveforms_lens" => TensorRef::from_array_view(waveforms_lens.view())?,
        ];
        let outputs = self.preprocessor.run(inputs)?;

        let features = outputs
            .get("features")
            .ok_or_else(|| ParakeetError::OutputNotFound("features".to_string()))?
            .try_extract_array()?;
        let features_lens = outputs
            .get("features_lens")
            .ok_or_else(|| ParakeetError::OutputNotFound("features_lens".to_string()))?
            .try_extract_array()?;

        Ok((features.to_owned(), features_lens.to_owned()))
    }

    pub fn encode(
        &mut self,
        audio_signal: &ArrayViewD<f32>,
        length: &ArrayViewD<i64>,
    ) -> Result<(ArrayD<f32>, ArrayD<i64>), ParakeetError> {
        log::trace!("Running encoder inference...");
        let inputs = inputs![
            "audio_signal" => TensorRef::from_array_view(audio_signal.view())?,
            "length" => TensorRef::from_array_view(length.view())?,
        ];
        let outputs = self.encoder.run(inputs)?;

        let encoder_output = outputs
            .get("outputs")
            .ok_or_else(|| ParakeetError::OutputNotFound("outputs".to_string()))?
            .try_extract_array()?;
        let encoded_lengths = outputs
            .get("encoded_lengths")
            .ok_or_else(|| ParakeetError::OutputNotFound("encoded_lengths".to_string()))?
            .try_extract_array()?;

        let encoder_output = encoder_output.permuted_axes(IxDyn(&[0, 2, 1]));

        Ok((encoder_output.to_owned(), encoded_lengths.to_owned()))
    }

    pub fn create_decoder_state(&self) -> Result<DecoderState, ParakeetError> {
        // Get input shapes from decoder model
        let inputs = &self.decoder_joint.inputs;

        let state1_shape = inputs
            .iter()
            .find(|input| input.name == "input_states_1")
            .ok_or_else(|| ParakeetError::InputNotFound("input_states_1".to_string()))?
            .input_type
            .tensor_shape()
            .ok_or_else(|| ParakeetError::TensorShape("input_states_1".to_string()))?;

        let state2_shape = inputs
            .iter()
            .find(|input| input.name == "input_states_2")
            .ok_or_else(|| ParakeetError::InputNotFound("input_states_2".to_string()))?
            .input_type
            .tensor_shape()
            .ok_or_else(|| ParakeetError::TensorShape("input_states_2".to_string()))?;

        // Create zero states with batch_size=1
        // Shape is [2, -1, 640] so we use [2, 1, 640] for batch_size=1
        let state1 = Array::zeros((
            state1_shape[0] as usize,
            1, // batch_size = 1
            state1_shape[2] as usize,
        ));

        let state2 = Array::zeros((
            state2_shape[0] as usize,
            1, // batch_size = 1
            state2_shape[2] as usize,
        ));

        Ok((state1, state2))
    }

    pub fn decode_step(
        &mut self,
        prev_tokens: &[i32],
        prev_state: &DecoderState,
        encoder_out: &ArrayViewD<f32>, // [time_steps, 1024]
    ) -> Result<(ArrayD<f32>, DecoderState), ParakeetError> {
        log::trace!("Running decoder inference...");

        // Get last token or blank_idx if empty
        let target_token = prev_tokens.last().copied().unwrap_or(self.blank_idx);

        // Prepare inputs matching Python: encoder_out[None, :, None] -> [1, time_steps, 1]
        let encoder_outputs = encoder_out
            .to_owned()
            .insert_axis(ndarray::Axis(0))
            .insert_axis(ndarray::Axis(2));
        let targets = Array2::from_shape_vec((1, 1), vec![target_token])?;
        let target_length = Array1::from_vec(vec![1]);

        let inputs = inputs![
            "encoder_outputs" => TensorRef::from_array_view(encoder_outputs.view())?,
            "targets" => TensorRef::from_array_view(targets.view())?,
            "target_length" => TensorRef::from_array_view(target_length.view())?,
            "input_states_1" => TensorRef::from_array_view(prev_state.0.view())?,
            "input_states_2" => TensorRef::from_array_view(prev_state.1.view())?,
        ];

        let outputs = self.decoder_joint.run(inputs)?;

        let logits = outputs
            .get("outputs")
            .ok_or_else(|| ParakeetError::OutputNotFound("outputs".to_string()))?
            .try_extract_array()?;
        log::trace!(
            "Logits shape: {:?}, vocab_size: {}",
            logits.shape(),
            self.vocab_size
        );
        let state1 = outputs
            .get("output_states_1")
            .ok_or_else(|| ParakeetError::OutputNotFound("output_states_1".to_string()))?
            .try_extract_array()?;
        let state2 = outputs
            .get("output_states_2")
            .ok_or_else(|| ParakeetError::OutputNotFound("output_states_2".to_string()))?
            .try_extract_array()?;

        // Squeeze outputs like Python (remove batch dimension)
        let logits = logits.remove_axis(ndarray::Axis(0));

        // Convert ArrayD back to Array3 to match expected return type
        let state1_3d = state1.to_owned().into_dimensionality::<ndarray::Ix3>()?;
        let state2_3d = state2.to_owned().into_dimensionality::<ndarray::Ix3>()?;

        Ok((logits.to_owned(), (state1_3d, state2_3d)))
    }

    pub fn recognize_batch(
        &mut self,
        waveforms: &ArrayViewD<f32>,
        waveforms_len: &ArrayViewD<i64>,
    ) -> Result<Vec<TimestampedResult>, ParakeetError> {
        // Preprocess and encode
        let (features, features_lens) = self.preprocess(waveforms, waveforms_len)?;
        let (encoder_out, encoder_out_lens) =
            self.encode(&features.view(), &features_lens.view())?;

        // Decode for each batch item
        let mut results = Vec::new();
        for (encodings, &encodings_len) in encoder_out.outer_iter().zip(encoder_out_lens.iter()) {
            let (tokens, timestamps) =
                self.decode_sequence(&encodings.view(), encodings_len as usize)?;
            let result = self.decode_tokens(tokens, timestamps);
            results.push(result);
        }

        Ok(results)
    }

    fn decode_sequence(
        &mut self,
        encodings: &ArrayViewD<f32>, // [time_steps, 1024]
        encodings_len: usize,
    ) -> Result<(Vec<i32>, Vec<usize>), ParakeetError> {
        let mut prev_state = self.create_decoder_state()?;
        let mut tokens = Vec::new();
        let mut timestamps = Vec::new();

        let mut t = 0;
        let mut emitted_tokens = 0;

        while t < encodings_len {
            let encoder_step = encodings.slice(ndarray::s![t, ..]);
            // Convert to dynamic dimension to match decode_step parameter type
            let encoder_step_dyn = encoder_step.to_owned().into_dyn();
            let (probs, new_state) =
                self.decode_step(&tokens, &prev_state, &encoder_step_dyn.view())?;

            // For TDT models, split output into vocab logits and duration logits
            // output[:vocab_size] = vocabulary logits
            // output[vocab_size:] = duration logits
            let vocab_logits_slice = probs.as_slice().ok_or_else(|| {
                ParakeetError::Shape(ndarray::ShapeError::from_kind(
                    ndarray::ErrorKind::IncompatibleShape,
                ))
            })?;

            let vocab_logits = if probs.len() > self.vocab_size + 1 {
                // TDT model - extract only vocabulary logits
                log::trace!(
                    "TDT model detected: splitting {} logits into vocab({}) + duration",
                    probs.len(),
                    self.vocab_size
                );
                &vocab_logits_slice[..self.vocab_size]
            } else {
                // Regular RNN-T model. Keep the trailing RNNT blank logit when present.
                vocab_logits_slice
            };

            // Get argmax token from vocabulary logits only
            let token = vocab_logits
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(idx, _)| idx as i32)
                .unwrap_or(self.blank_idx);

            if token != self.blank_idx {
                prev_state = new_state;
                tokens.push(token);
                timestamps.push(t);
                emitted_tokens += 1;
            }

            // Step logic from Python - simplified since step is always -1
            if token == self.blank_idx || emitted_tokens == MAX_TOKENS_PER_STEP {
                t += 1;
                emitted_tokens = 0;
            }
        }

        Ok((tokens, timestamps))
    }

    fn decode_tokens(&self, ids: Vec<i32>, timestamps: Vec<usize>) -> TimestampedResult {
        let tokens: Vec<String> = ids
            .iter()
            .filter_map(|&id| {
                let idx = id as usize;
                if idx < self.vocab.len() {
                    Some(self.vocab[idx].clone())
                } else {
                    None
                }
            })
            .collect();

        let text = if tokens.iter().any(|token| token.starts_with("##")) {
            decode_wordpiece_tokens(&tokens)
        } else {
            match &*DECODE_SPACE_RE {
                Ok(regex) => regex
                    .replace_all(&tokens.join(""), |caps: &regex::Captures| {
                        if caps.get(1).is_some() {
                            " "
                        } else {
                            ""
                        }
                    })
                    .to_string(),
                Err(_) => tokens.join(""), // Fallback if regex failed to compile
            }
        };

        let float_timestamps: Vec<f32> = timestamps
            .iter()
            .map(|&t| WINDOW_SIZE * SUBSAMPLING_FACTOR as f32 * t as f32)
            .collect();

        TimestampedResult {
            text,
            timestamps: float_timestamps,
            tokens,
        }
    }

    pub fn transcribe_samples(
        &mut self,
        samples: Vec<f32>,
    ) -> Result<TimestampedResult, ParakeetError> {
        let batch_size = 1;
        let samples_len = samples.len();

        // Create waveforms array [batch_size, samples_len]
        let waveforms = Array2::from_shape_vec((batch_size, samples_len), samples)?.into_dyn();

        // Create waveforms_lens array [batch_size] with the actual length
        let waveforms_lens = Array1::from_vec(vec![samples_len as i64]).into_dyn();

        // Run recognition to get detailed results
        let results = self.recognize_batch(&waveforms.view(), &waveforms_lens.view())?;

        // Extract the first (and only) result
        let timestamped_result = results.into_iter().next().ok_or_else(|| {
            ParakeetError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "No transcription result returned",
            ))
        })?;

        Ok(timestamped_result)
    }
}

// TranscriptionEngine trait implementation
use super::helpers::convert_timestamps;
use super::transcription_engine::{TranscriptionEngine, TranscriptionResult};
use super::types::{
    ParakeetEngine, ParakeetInferenceParams, ParakeetModelParams, QuantizationType,
};
use std::path::Path as StdPath;

impl TranscriptionEngine for ParakeetEngine {
    type InferenceParams = ParakeetInferenceParams;
    type ModelParams = ParakeetModelParams;

    fn load_model_with_params(
        &mut self,
        model_path: &StdPath,
        params: Self::ModelParams,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let quantized = match params.quantization {
            QuantizationType::FP32 => false,
            QuantizationType::Int8 => true,
        };
        let model = ParakeetModel::new(model_path, quantized)?;

        self.model = Some(model);
        self.loaded_model_path = Some(model_path.to_path_buf());
        Ok(())
    }

    fn transcribe_samples(
        &mut self,
        samples: Vec<f32>,
        params: Option<Self::InferenceParams>,
    ) -> Result<TranscriptionResult, Box<dyn std::error::Error>> {
        let model: &mut ParakeetModel = self
            .model
            .as_mut()
            .ok_or("Model not loaded. Call load_model_with_params() first.")?;

        let parakeet_params = params.unwrap_or_default();

        // Get the timestamped result from the model
        let timestamped_result = model.transcribe_samples(samples)?;

        // Convert timestamps based on requested granularity
        let segments =
            convert_timestamps(&timestamped_result, parakeet_params.timestamp_granularity);

        Ok(TranscriptionResult {
            text: timestamped_result.text,
            segments,
        })
    }
}

#[cfg(test)]
mod execution_provider_tests {
    use super::preferred_execution_provider_names;
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    use super::{find_cudnn_runtime_dir, find_packaged_cuda_provider_dir};

    #[test]
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    fn windows_prefers_cuda_before_cpu_fallback() {
        assert_eq!(
            preferred_execution_provider_names(),
            ["CUDAExecutionProvider", "CPUExecutionProvider"]
        );
    }

    #[test]
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    fn finds_versioned_cudnn_runtime_for_normal_app_launches() {
        let test_root =
            std::env::temp_dir().join(format!("hyperyap-cudnn-discovery-{}", uuid::Uuid::new_v4()));
        let runtime_dir = test_root
            .join("NVIDIA")
            .join("CUDNN")
            .join("v9.10")
            .join("bin")
            .join("12.9");
        std::fs::create_dir_all(&runtime_dir).expect("create fake cuDNN layout");
        std::fs::write(runtime_dir.join("cudnn64_9.dll"), []).expect("write cuDNN marker");

        let discovered = find_cudnn_runtime_dir(&test_root);

        assert_eq!(discovered.as_deref(), Some(runtime_dir.as_path()));
        std::fs::remove_dir_all(test_root).expect("clean fake cuDNN layout");
    }

    #[test]
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    fn finds_cuda_provider_bundled_below_the_installed_executable() {
        let test_root = std::env::temp_dir().join(format!(
            "hyperyap-cuda-provider-discovery-{}",
            uuid::Uuid::new_v4()
        ));
        let provider_dir = test_root.join("target").join("release");
        std::fs::create_dir_all(&provider_dir).expect("create fake provider layout");
        std::fs::write(provider_dir.join("onnxruntime_providers_cuda.dll"), [])
            .expect("write CUDA provider marker");
        std::fs::write(provider_dir.join("onnxruntime_providers_shared.dll"), [])
            .expect("write shared provider marker");

        let discovered = find_packaged_cuda_provider_dir(&test_root);

        assert_eq!(discovered.as_deref(), Some(provider_dir.as_path()));
        std::fs::remove_dir_all(test_root).expect("clean fake provider layout");
    }

    #[test]
    #[cfg(not(all(target_os = "windows", target_arch = "x86_64")))]
    fn non_windows_uses_cpu_provider() {
        assert_eq!(
            preferred_execution_provider_names(),
            ["CPUExecutionProvider"]
        );
    }
}
