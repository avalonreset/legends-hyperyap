# Agent Recipe: `legends-hyperyap`

This recipe is loaded via the `cto-legends` router (`cto-legends handoff legends-hyperyap`). Do not register `legends-hyperyap` as a standalone skill.

## Readiness Check

Verify the source build environment and local HTTP API status before building or controlling HyperYap:

```bash
pnpm run lint
pnpm run clippy
```

## Build & Local API Command Reference

| Command / Surface | Purpose | Key Options / Endpoints |
| --- | --- | --- |
| `bash .github/scripts/download-parakeet-model.sh resources` | Fetch and unpack the default Parakeet TDT 0.6B v2 INT8 ONNX model | Destination directory (`resources`) |
| `cargo build --release --manifest-path hotkeys/Cargo.toml` | Compile the native Windows Rust hotkey and smart terminal paste helper (`hyperyap-hotkeys.exe`) | `--release` |
| `pnpm tauri dev` | Run HyperYap in development mode (Vite + Rust Tauri shell) | None |
| `pnpm tauri build` | Build production desktop application and installer bundle | `--no-bundle` |
| `http://127.0.0.1:<port>/...` | Optional localhost HTTP API when enabled in Settings (see [`docs/API_USAGE.md`](API_USAGE.md)) | Localhost-bound only |

## EDR and Security Guardrails

- **No script wrappers**: Do not generate or execute `.ps1`, `.bat`, `.cmd`, `.vbs`, or `.ahk` files.
- **No hidden window flags**: Never pass hidden window or execution-policy bypass flags when spawning processes.
- **No persistence mutation**: Never modify Windows Registry Run keys (`HKCU\...\Run`, `HKLM\...\Run`), Startup folders, or Scheduled Tasks (`schtasks`).
- **Explicit argv execution**: Always invoke `pnpm`, `cargo`, and native binaries with explicit argument arrays (`shell=False`).
