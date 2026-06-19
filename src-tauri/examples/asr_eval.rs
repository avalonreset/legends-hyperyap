use anyhow::{anyhow, bail, Context, Result};
use hyperyap_lib::engine::transcription_engine::TranscriptionEngine;
use hyperyap_lib::engine::{ParakeetEngine, ParakeetModelParams};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug, Deserialize)]
struct ManifestItem {
    id: String,
    audio: PathBuf,
    reference: String,
}

#[derive(Debug, Serialize)]
struct EvalItem {
    id: String,
    audio: PathBuf,
    duration_sec: f64,
    elapsed_sec: f64,
    realtime_factor: f64,
    reference: String,
    hypothesis: String,
    wer: f64,
    cer: f64,
    reference_words: usize,
    reference_chars: usize,
    character_errors: usize,
    substitutions: usize,
    insertions: usize,
    deletions: usize,
}

#[derive(Debug, Serialize)]
struct EvalReport {
    label: String,
    model_dir: PathBuf,
    manifest: PathBuf,
    samples: usize,
    total_duration_sec: f64,
    total_elapsed_sec: f64,
    realtime_factor: f64,
    weighted_wer: f64,
    weighted_cer: f64,
    mean_wer: f64,
    mean_cer: f64,
    total_reference_words: usize,
    total_reference_chars: usize,
    total_character_errors: usize,
    total_substitutions: usize,
    total_insertions: usize,
    total_deletions: usize,
    items: Vec<EvalItem>,
}

struct Args {
    model_dir: PathBuf,
    manifest: PathBuf,
    out_dir: PathBuf,
    label: String,
    fp32: bool,
}

fn main() -> Result<()> {
    let args = parse_args()?;
    fs::create_dir_all(&args.out_dir)
        .with_context(|| format!("failed to create {}", args.out_dir.display()))?;

    let items = read_manifest(&args.manifest)?;
    if items.is_empty() {
        bail!("manifest has no eval items: {}", args.manifest.display());
    }

    let mut engine = ParakeetEngine::new();
    let params = if args.fp32 {
        ParakeetModelParams::default()
    } else {
        ParakeetModelParams::int8()
    };
    engine
        .load_model_with_params(&args.model_dir, params)
        .map_err(|e| anyhow!("failed to load model: {e}"))?;

    let mut eval_items = Vec::new();
    let total_timer = Instant::now();

    for item in items {
        let (samples, duration_sec) = read_wav_samples(&item.audio)
            .with_context(|| format!("failed to read {}", item.audio.display()))?;
        let timer = Instant::now();
        let result = engine
            .transcribe_samples(samples, None)
            .map_err(|e| anyhow!("transcription failed for {}: {e}", item.id))?;
        let elapsed_sec = timer.elapsed().as_secs_f64();
        let wer = word_error_stats(&item.reference, &result.text);
        let reference_words = normalize_words(&item.reference).len();
        let cer = character_error_stats(&item.reference, &result.text);
        let reference_chars = normalize_chars(&item.reference).len();

        eval_items.push(EvalItem {
            id: item.id,
            audio: item.audio,
            duration_sec,
            elapsed_sec,
            realtime_factor: if duration_sec > 0.0 {
                elapsed_sec / duration_sec
            } else {
                0.0
            },
            reference: item.reference,
            hypothesis: result.text,
            wer: wer.rate,
            cer: cer.rate,
            reference_words,
            reference_chars,
            character_errors: cer.errors,
            substitutions: wer.substitutions,
            insertions: wer.insertions,
            deletions: wer.deletions,
        });
    }

    let total_elapsed_sec = total_timer.elapsed().as_secs_f64();
    let total_duration_sec: f64 = eval_items.iter().map(|item| item.duration_sec).sum();
    let total_reference_words: usize = eval_items.iter().map(|item| item.reference_words).sum();
    let total_reference_chars: usize = eval_items.iter().map(|item| item.reference_chars).sum();
    let total_character_errors: usize = eval_items.iter().map(|item| item.character_errors).sum();
    let total_substitutions: usize = eval_items.iter().map(|item| item.substitutions).sum();
    let total_insertions: usize = eval_items.iter().map(|item| item.insertions).sum();
    let total_deletions: usize = eval_items.iter().map(|item| item.deletions).sum();
    let total_errors = total_substitutions + total_insertions + total_deletions;
    let weighted_wer = if total_reference_words > 0 {
        total_errors as f64 / total_reference_words as f64
    } else {
        0.0
    };
    let weighted_cer = if total_reference_chars > 0 {
        total_character_errors as f64 / total_reference_chars as f64
    } else {
        0.0
    };
    let mean_wer = eval_items.iter().map(|item| item.wer).sum::<f64>() / eval_items.len() as f64;
    let mean_cer = eval_items.iter().map(|item| item.cer).sum::<f64>() / eval_items.len() as f64;

    let report = EvalReport {
        label: args.label.clone(),
        model_dir: args.model_dir.clone(),
        manifest: args.manifest.clone(),
        samples: eval_items.len(),
        total_duration_sec,
        total_elapsed_sec,
        realtime_factor: if total_duration_sec > 0.0 {
            total_elapsed_sec / total_duration_sec
        } else {
            0.0
        },
        weighted_wer,
        weighted_cer,
        mean_wer,
        mean_cer,
        total_reference_words,
        total_reference_chars,
        total_character_errors,
        total_substitutions,
        total_insertions,
        total_deletions,
        items: eval_items,
    };

    let json_path = args.out_dir.join(format!("{}.json", args.label));
    let md_path = args.out_dir.join(format!("{}.md", args.label));
    fs::write(&json_path, serde_json::to_string_pretty(&report)?)?;
    fs::write(&md_path, render_markdown(&report))?;

    println!("{}", serde_json::to_string_pretty(&report)?);
    println!("wrote {}", json_path.display());
    println!("wrote {}", md_path.display());

    Ok(())
}

fn parse_args() -> Result<Args> {
    let mut model_dir = None;
    let mut manifest = None;
    let mut out_dir = None;
    let mut label = None;
    let mut fp32 = false;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--model-dir" => model_dir = args.next().map(PathBuf::from),
            "--manifest" => manifest = args.next().map(PathBuf::from),
            "--out-dir" => out_dir = args.next().map(PathBuf::from),
            "--label" => label = args.next(),
            "--fp32" => fp32 = true,
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            other => bail!("unknown argument: {other}"),
        }
    }

    Ok(Args {
        model_dir: model_dir.ok_or_else(|| anyhow!("missing --model-dir"))?,
        manifest: manifest.ok_or_else(|| anyhow!("missing --manifest"))?,
        out_dir: out_dir.ok_or_else(|| anyhow!("missing --out-dir"))?,
        label: label.unwrap_or_else(|| "asr-eval".to_string()),
        fp32,
    })
}

fn print_help() {
    println!(
        "\
asr_eval

USAGE:
  cargo run --example asr_eval -- --model-dir <DIR> --manifest <JSONL> --out-dir <DIR> [--label NAME] [--fp32]

Manifest JSONL rows:
  {{\"id\":\"sample-01\",\"audio\":\"C:\\\\path\\\\sample.wav\",\"reference\":\"known transcript\"}}
"
    );
}

fn read_manifest(path: &Path) -> Result<Vec<ManifestItem>> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    content
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(idx, line)| {
            serde_json::from_str::<ManifestItem>(line)
                .with_context(|| format!("invalid manifest JSON on line {}", idx + 1))
        })
        .collect()
}

fn read_wav_samples(wav_path: &Path) -> Result<(Vec<f32>, f64)> {
    let mut reader = hound::WavReader::open(wav_path)?;
    let spec = reader.spec();

    if spec.bits_per_sample != 16 {
        bail!(
            "expected 16 bits per sample, found {} in {}",
            spec.bits_per_sample,
            wav_path.display()
        );
    }

    if spec.sample_format != hound::SampleFormat::Int {
        bail!(
            "expected Int sample format, found {:?} in {}",
            spec.sample_format,
            wav_path.display()
        );
    }

    let raw_i16: std::result::Result<Vec<i16>, _> = reader.samples::<i16>().collect();
    let mut raw_i16 = raw_i16?;
    let source_frames = if spec.channels > 0 {
        raw_i16.len() / spec.channels as usize
    } else {
        0
    };
    let duration_sec = if spec.sample_rate > 0 {
        source_frames as f64 / spec.sample_rate as f64
    } else {
        0.0
    };

    if spec.channels > 1 {
        let channels = spec.channels as usize;
        let mut mono = Vec::with_capacity(raw_i16.len() / channels);
        for frame in raw_i16.chunks_exact(channels) {
            let sum: i32 = frame.iter().map(|sample| *sample as i32).sum();
            let avg = (sum / channels as i32).clamp(i16::MIN as i32, i16::MAX as i32) as i16;
            mono.push(avg);
        }
        raw_i16 = mono;
    }

    let samples: Vec<f32> = raw_i16
        .into_iter()
        .map(|sample| sample as f32 / i16::MAX as f32)
        .collect();

    let samples = if spec.sample_rate != 16000 {
        resample_linear(&samples, spec.sample_rate as usize, 16000)
    } else {
        samples
    };

    Ok((samples, duration_sec))
}

fn resample_linear(input: &[f32], src_hz: usize, dst_hz: usize) -> Vec<f32> {
    if input.is_empty() || src_hz == 0 || dst_hz == 0 {
        return Vec::new();
    }
    if src_hz == dst_hz {
        return input.to_vec();
    }

    let ratio = dst_hz as f64 / src_hz as f64;
    let out_len = ((input.len() as f64) * ratio).ceil() as usize;
    let last_idx = input.len().saturating_sub(1);
    let mut out = Vec::with_capacity(out_len);

    for i in 0..out_len {
        let t = (i as f64) / ratio;
        let idx = t.floor() as usize;
        let frac = (t - idx as f64) as f32;
        let a = input[idx];
        let b = input[std::cmp::min(idx + 1, last_idx)];
        out.push(a + (b - a) * frac);
    }

    out
}

struct WerStats {
    rate: f64,
    substitutions: usize,
    insertions: usize,
    deletions: usize,
}

struct ErrorRate {
    rate: f64,
    errors: usize,
}

#[derive(Clone, Copy)]
struct Cell {
    cost: usize,
    substitutions: usize,
    insertions: usize,
    deletions: usize,
}

fn word_error_stats(reference: &str, hypothesis: &str) -> WerStats {
    let reference = normalize_words(reference);
    let hypothesis = normalize_words(hypothesis);
    let rows = reference.len() + 1;
    let cols = hypothesis.len() + 1;
    let mut dp = vec![
        Cell {
            cost: 0,
            substitutions: 0,
            insertions: 0,
            deletions: 0,
        };
        rows * cols
    ];

    let index = |row: usize, col: usize| row * cols + col;

    for row in 1..rows {
        dp[index(row, 0)] = Cell {
            cost: row,
            substitutions: 0,
            insertions: 0,
            deletions: row,
        };
    }
    for col in 1..cols {
        dp[index(0, col)] = Cell {
            cost: col,
            substitutions: 0,
            insertions: col,
            deletions: 0,
        };
    }

    for row in 1..rows {
        for col in 1..cols {
            if reference[row - 1] == hypothesis[col - 1] {
                dp[index(row, col)] = dp[index(row - 1, col - 1)];
                continue;
            }

            let mut sub = dp[index(row - 1, col - 1)];
            sub.cost += 1;
            sub.substitutions += 1;

            let mut del = dp[index(row - 1, col)];
            del.cost += 1;
            del.deletions += 1;

            let mut ins = dp[index(row, col - 1)];
            ins.cost += 1;
            ins.insertions += 1;

            dp[index(row, col)] = [sub, del, ins]
                .into_iter()
                .min_by_key(|cell| {
                    (
                        cell.cost,
                        cell.insertions + cell.deletions,
                        cell.substitutions,
                    )
                })
                .unwrap();
        }
    }

    let final_cell = dp[index(reference.len(), hypothesis.len())];
    WerStats {
        rate: if reference.is_empty() {
            if hypothesis.is_empty() {
                0.0
            } else {
                1.0
            }
        } else {
            final_cell.cost as f64 / reference.len() as f64
        },
        substitutions: final_cell.substitutions,
        insertions: final_cell.insertions,
        deletions: final_cell.deletions,
    }
}

fn normalize_words(text: &str) -> Vec<String> {
    let mut normalized = String::with_capacity(text.len());
    for ch in text.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() || ch == '\'' {
            normalized.push(ch);
        } else {
            normalized.push(' ');
        }
    }
    normalized
        .split_whitespace()
        .map(|word| word.trim_matches('\'').to_string())
        .filter(|word| !word.is_empty())
        .collect()
}

fn character_error_stats(reference: &str, hypothesis: &str) -> ErrorRate {
    let reference = normalize_chars(reference);
    let hypothesis = normalize_chars(hypothesis);
    let errors = edit_distance(&reference, &hypothesis);
    ErrorRate {
        rate: if reference.is_empty() {
            if hypothesis.is_empty() {
                0.0
            } else {
                1.0
            }
        } else {
            errors as f64 / reference.len() as f64
        },
        errors,
    }
}

fn normalize_chars(text: &str) -> Vec<char> {
    normalize_words(text).join(" ").chars().collect()
}

fn edit_distance<T: Eq>(reference: &[T], hypothesis: &[T]) -> usize {
    let rows = reference.len() + 1;
    let cols = hypothesis.len() + 1;
    let mut dp = vec![0usize; rows * cols];
    let index = |row: usize, col: usize| row * cols + col;

    for row in 1..rows {
        dp[index(row, 0)] = row;
    }
    for col in 1..cols {
        dp[index(0, col)] = col;
    }

    for row in 1..rows {
        for col in 1..cols {
            let substitution_cost = if reference[row - 1] == hypothesis[col - 1] {
                0
            } else {
                1
            };
            dp[index(row, col)] = [
                dp[index(row - 1, col)] + 1,
                dp[index(row, col - 1)] + 1,
                dp[index(row - 1, col - 1)] + substitution_cost,
            ]
            .into_iter()
            .min()
            .unwrap();
        }
    }

    dp[index(reference.len(), hypothesis.len())]
}

fn render_markdown(report: &EvalReport) -> String {
    let mut markdown = String::new();
    markdown.push_str(&format!("# ASR Eval: {}\n\n", report.label));
    markdown.push_str(&format!("- Model: `{}`\n", report.model_dir.display()));
    markdown.push_str(&format!("- Manifest: `{}`\n", report.manifest.display()));
    markdown.push_str(&format!("- Samples: `{}`\n", report.samples));
    markdown.push_str(&format!(
        "- Weighted WER: `{:.2}%`\n",
        report.weighted_wer * 100.0
    ));
    markdown.push_str(&format!(
        "- Weighted CER: `{:.2}%`\n",
        report.weighted_cer * 100.0
    ));
    markdown.push_str(&format!("- Mean WER: `{:.2}%`\n", report.mean_wer * 100.0));
    markdown.push_str(&format!("- Mean CER: `{:.2}%`\n", report.mean_cer * 100.0));
    markdown.push_str(&format!(
        "- Realtime factor: `{:.2}x`\n\n",
        report.realtime_factor
    ));
    markdown.push_str("| ID | WER | CER | RTF | Reference | Hypothesis |\n");
    markdown.push_str("| --- | ---: | ---: | ---: | --- | --- |\n");

    for item in &report.items {
        markdown.push_str(&format!(
            "| {} | {:.2}% | {:.2}% | {:.2}x | {} | {} |\n",
            escape_markdown_cell(&item.id),
            item.wer * 100.0,
            item.cer * 100.0,
            item.realtime_factor,
            escape_markdown_cell(&item.reference),
            escape_markdown_cell(&item.hypothesis)
        ));
    }

    markdown
}

fn escape_markdown_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}
