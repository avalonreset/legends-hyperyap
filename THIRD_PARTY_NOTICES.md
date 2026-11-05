# Third-Party Notices

`legends-hyperyap` is licensed under the GNU Affero General Public License v3.0 (`AGPL-3.0-only`). See [LICENSE](LICENSE) and [NOTICE](NOTICE) for full project attribution.

## Upstream & Model Attribution

| Component | License | Notes |
| --- | --- | --- |
| [MURmure](https://github.com/Kieirra/murmure) | AGPL-3.0 | Upstream local speech-to-text Tauri/Rust application by Kieirra |
| [nvidia/parakeet-tdt-0.6b-v2](https://huggingface.co/nvidia/parakeet-tdt-0.6b-v2) | CC-BY-4.0 | NVIDIA Parakeet TDT 0.6B v2 English speech recognition model |
| [smcleod/parakeet-tdt-0.6b-v2-int8](https://huggingface.co/smcleod/parakeet-tdt-0.6b-v2-int8) | CC-BY-4.0 | ONNX INT8 model conversion by Sam McLeod |

## Runtime & Build Dependencies

| Package | License | Purpose |
| --- | --- | --- |
| Tauri (`tauri`, `@tauri-apps/api`) | Apache-2.0 OR MIT | Cross-platform desktop shell |
| React (`react`, `react-dom`) | MIT | Frontend user interface |
| ONNX Runtime (`ort`) | MIT / Apache-2.0 | Local ONNX model inference |
| `windows-sys` | MIT OR Apache-2.0 | Native Win32 bindings for `hyperyap-hotkeys` |
| `image` | MIT OR Apache-2.0 | Native PNG encoding in `hyperyap-hotkeys` |
