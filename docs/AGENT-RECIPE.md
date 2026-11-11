# Agent Recipe: `legends-hyperyap` (`v0.1.4`)

This recipe is loaded via the `cto-legends` router (`cto-legends handoff legends-hyperyap`). Do not register `legends-hyperyap` as a standalone skill.

## Readiness Check

Verify the source build environment and release hygiene gate before building or controlling HyperYap:

```bash
python scripts/check_release.py
cargo build --release --manifest-path hotkeys/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

## Build, CLI & Local API Command Reference

| Command / Surface | Purpose | Key Options / Endpoints |
| --- | --- | --- |
| `bash .github/scripts/download-parakeet-model.sh resources` | Fetch and SHA-256 verify the pinned Parakeet TDT 0.6B v2 INT8 ONNX model (`d64884b484b919e9656d0b70cb95dfdc98852bef`) | Destination directory (`resources`) |
| `cargo build --release --manifest-path hotkeys/Cargo.toml` | Compile the opt-in native Windows Rust hotkey and smart terminal paste helper (`hyperyap-hotkeys.exe`) | `--release` |
| `pnpm tauri dev` | Run HyperYap in development mode (Vite + Rust Tauri shell) | None |
| `pnpm tauri build` | Build production desktop application and installer bundle | `--no-bundle` |
| `hyperyap import <FILE> [-s\|--strategy replace\|merge]` | Import a `.hyperyap` JSON configuration file from the CLI (`<= 2 MiB`, symlinks refused) | `<FILE>`, `-s replace` (default) or `-s merge` |
| `POST http://127.0.0.1:<port>/api/transcribe` | Optional localhost HTTP transcription API when enabled in Settings (see [`docs/API_USAGE.md`](API_USAGE.md)) | Requires `Authorization: Bearer <api_token>` header and multipart `audio=@file.wav` |

## Environment Variables

| Variable | Purpose | Default |
| --- | --- | --- |
| `HYPERYAP_ASR_MODEL_ID` | Override the speech model directory name under `resources/` | `parakeet-tdt-0.6b-v2-smcleod-int8` |
| `HYPERYAP_ASR_MODEL_DIR` | Override the full filesystem path to a validated ONNX Parakeet model directory | None (uses bundled or `%APPDATA%` resources path) |

## EDR and Security Guardrails

- **No script wrappers**: Do not generate or execute `.ps1`, `.bat`, `.cmd`, `.vbs`, or `.ahk` files.
- **No hidden window flags**: Never pass hidden window or execution-policy bypass flags when spawning processes.
- **No persistence mutation without consent**: Autostart (`autostart`) and the Windows workstation hotkey helper (`enable_hotkeys_helper`) are disabled (`false`) by default and require explicit user opt-in.
- **Event-driven shortcuts**: Windows shortcuts use OS-registered `RegisterHotKey` + `WM_HOTKEY` message delivery by default (unconditional key-state polling is prohibited).
- **Explicit argv execution**: Always invoke `pnpm`, `cargo`, and native binaries with explicit argument arrays (`shell=False`).
