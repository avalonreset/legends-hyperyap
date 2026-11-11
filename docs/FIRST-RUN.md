# first dictation

## install and prepare

1. `v0.1.4` is published as a source release. Build and package the desktop app
   for your platform using the guided source build in
   [`README.md#install`](../README.md#install), or download prebuilt installers
   when attached to the
   [latest release](https://github.com/avalonreset/legends-hyperyap/releases/latest)
   (Windows x64 setup EXE, macOS Apple Silicon and Intel DMGs, Linux x86_64
   AppImage and Debian package).
2. Launch hyperyap and allow the initial speech-model download to complete.
   The default Parakeet v2 model is English-focused and SHA-256 verified against
   its pinned revision.
3. Check the microphone and record shortcut in Settings. Grant microphone
   access when requested; macOS also needs Accessibility permission for global
   shortcuts. Linux shortcut behavior depends on the desktop session. On
   Windows, standard shortcuts use OS-registered `RegisterHotKey` events; the
   workstation hotkey helper (`enable_hotkeys_helper`) and login autostart are
   opt-in in Settings.
4. Leave LLM Connect disabled for the first check. It is optional and unrelated
   to whether the speech model can transcribe.

## prove the record-and-paste loop

Open a blank document in a text editor and place the cursor in it. Press your
configured recording shortcut, say "this is my first local dictation," and press
the shortcut again. This assumes the default toggle-to-talk mode; push-to-talk
instead records while the shortcut is held.

Success means the spoken words appear in that document. After the model is
downloaded, repeat this with the network disconnected to check the local path.
Do not use a terminal for the first test: pasted text there can become a command.

Then add a frequently used name to the custom dictionary or a deliberate
formatting rule, and repeat the check in your usual app. Keep the test phrase
short so transcription, shortcut, and paste failures are easy to distinguish.

## troubleshooting

| symptom | first check |
|---|---|
| no recording indicator | confirm the shortcut and microphone permission; check desktop global-shortcut restrictions |
| recording starts but the transcript is empty | verify the selected microphone produces input in the operating system; check that model setup completed |
| transcript exists but nothing pastes | focus an editable text field and check platform permissions; use the configured last-transcript paste shortcut |
| CUDA is unavailable | CPU fallback is supported; a compatible NVIDIA GPU/runtime is optional |
| an unfamiliar name is consistently wrong | add the intended spelling to the custom dictionary and retest |
| settings changed after an upgrade | restore a previously exported `.hyperyap` configuration or settings backup |

If it persists, [open an issue](https://github.com/avalonreset/legends-hyperyap/issues)
with the app version, OS, install method, recording mode, and the failing step.
Review logs before sharing them; omit private transcripts, endpoint keys, and
personal paths.

## privacy choices

Core audio transcription is local. A local Ollama endpoint can also process text
locally. Enabling a remote LLM endpoint sends submitted text to that endpoint;
its privacy terms are separate. Internet access is used for initial downloads
and updates. The optional HTTP API is a separate integration surface, disabled
by default and protected by a per-install Bearer token; see
[API usage](API_USAGE.md) before enabling it.

See [configuration](../README.md#configuration) for settings and upgrade backups,
and [platform compatibility](../README.md#platform-compatibility) for the
opt-in Windows hotkey helper versus the cross-platform desktop app.
