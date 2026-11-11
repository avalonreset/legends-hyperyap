use log::debug;
use std::collections::{HashMap, HashSet};
use std::mem::zeroed;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL,
    MOD_NOREPEAT, MOD_SHIFT, MOD_WIN,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE, WM_HOTKEY,
};

use crate::shortcuts::registry::ShortcutRegistryState;
use crate::shortcuts::types::{
    ActivationMode, KeyEventType, ShortcutAction, ShortcutBinding, ShortcutState,
};

static SHORTCUT_WORKER_STARTED: AtomicBool = AtomicBool::new(false);

fn is_recording_action(action: &ShortcutAction) -> bool {
    matches!(
        action,
        ShortcutAction::StartRecording
            | ShortcutAction::StartRecordingLLM
            | ShortcutAction::StartRecordingCommand
    )
}

fn split_hotkey_chord(keys: &[i32]) -> Option<(HOT_KEY_MODIFIERS, u32)> {
    let mut modifiers: HOT_KEY_MODIFIERS = MOD_NOREPEAT;
    let mut primary_vk: Option<u32> = None;

    for &vk in keys {
        match vk {
            0x11 => modifiers |= MOD_CONTROL,
            0x10 => modifiers |= MOD_SHIFT,
            0x12 => modifiers |= MOD_ALT,
            0x5B => modifiers |= MOD_WIN,
            other if other > 0 => {
                if primary_vk.is_some() {
                    return None;
                }
                primary_vk = Some(other as u32);
            }
            _ => return None,
        }
    }

    primary_vk.map(|vk| (modifiers, vk))
}

unsafe fn sync_registered_hotkeys(
    bindings: &[ShortcutBinding],
    registered: &mut HashMap<i32, (HOT_KEY_MODIFIERS, u32)>,
) {
    for &id in registered.keys() {
        let _ = UnregisterHotKey(null_mut(), id);
    }
    registered.clear();

    for (idx, binding) in bindings.iter().enumerate() {
        if let Some((mods, vk)) = split_hotkey_chord(&binding.keys) {
            let hotkey_id = (idx as i32) + 1;
            if RegisterHotKey(null_mut(), hotkey_id, mods, vk) != 0 {
                registered.insert(hotkey_id, (mods, vk));
            }
        }
    }
}

fn check_keys_pressed_opt_in(keys: &[i32]) -> bool {
    keys.iter().all(|&vk| {
        if vk == 0x5B {
            ((unsafe { GetAsyncKeyState(0x5B) } as u16
                | unsafe { GetAsyncKeyState(0x5C) } as u16)
                & 0x8000)
                != 0
        } else {
            (unsafe { GetAsyncKeyState(vk) } as u16 & 0x8000) != 0
        }
    })
}

pub fn init(app: AppHandle) {
    if SHORTCUT_WORKER_STARTED
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        debug!("Windows shortcut worker already running; skipping duplicate spawn");
        return;
    }

    std::thread::spawn(move || {
        debug!("Starting Windows event-driven hotkey listener (RegisterHotKey)");

        let mut registered: HashMap<i32, (HOT_KEY_MODIFIERS, u32)> = HashMap::new();
        let mut last_signature: Vec<Vec<i32>> = Vec::new();
        let mut last_press_times: HashMap<usize, Instant> = HashMap::new();
        let mut held_hotkeys: HashMap<usize, ShortcutBinding> = HashMap::new();
        let mut active_fallback: HashMap<usize, ShortcutBinding> = HashMap::new();

        loop {
            let shortcut_state = app.state::<ShortcutState>();
            if shortcut_state.is_suspended() {
                if !registered.is_empty() {
                    unsafe {
                        for &id in registered.keys() {
                            let _ = UnregisterHotKey(null_mut(), id);
                        }
                    }
                    registered.clear();
                    last_signature.clear();
                }
                held_hotkeys.clear();
                active_fallback.clear();
                std::thread::sleep(Duration::from_millis(50));
                continue;
            }

            let (bindings_snapshot, polling_enabled): (Vec<ShortcutBinding>, bool) = {
                let registry_state = app.state::<ShortcutRegistryState>();
                let registry = registry_state.0.read();
                (
                    registry.bindings.clone(),
                    registry.enable_key_polling_fallback,
                )
            };

            let current_signature: Vec<Vec<i32>> =
                bindings_snapshot.iter().map(|b| b.keys.clone()).collect();
            if current_signature != last_signature {
                unsafe {
                    sync_registered_hotkeys(&bindings_snapshot, &mut registered);
                }
                last_signature = current_signature;
            }

            unsafe {
                let mut msg: MSG = zeroed();
                while PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) != 0 {
                    if msg.message == WM_HOTKEY {
                        let hotkey_id = msg.wParam as i32;
                        let idx = (hotkey_id - 1) as usize;
                        if let Some(binding) = bindings_snapshot.get(idx) {
                            let is_ptt_recording = binding.activation_mode
                                == ActivationMode::PushToTalk
                                && is_recording_action(&binding.action);
                            if is_ptt_recording {
                                if !held_hotkeys.contains_key(&idx) {
                                    held_hotkeys.insert(idx, binding.clone());
                                    crate::shortcuts::handle_shortcut_event(
                                        &app,
                                        &binding.action,
                                        &binding.activation_mode,
                                        KeyEventType::Pressed,
                                    );
                                }
                            } else {
                                let now = Instant::now();
                                let debounced = last_press_times
                                    .get(&idx)
                                    .map(|t| now.duration_since(*t) < Duration::from_millis(150))
                                    .unwrap_or(false);
                                if !debounced {
                                    last_press_times.insert(idx, now);
                                    crate::shortcuts::handle_shortcut_event(
                                        &app,
                                        &binding.action,
                                        &binding.activation_mode,
                                        KeyEventType::Pressed,
                                    );
                                    crate::shortcuts::handle_shortcut_event(
                                        &app,
                                        &binding.action,
                                        &binding.activation_mode,
                                        KeyEventType::Released,
                                    );
                                }
                            }
                        }
                    } else {
                        TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }
                }
            }

            // Strictly bounded release check ONLY while an active PushToTalk WM_HOTKEY is held
            if !held_hotkeys.is_empty() {
                let released_indices: Vec<usize> = held_hotkeys
                    .iter()
                    .filter(|(_, binding)| !check_keys_pressed_opt_in(&binding.keys))
                    .map(|(&idx, _)| idx)
                    .collect();
                for idx in released_indices {
                    if let Some(binding) = held_hotkeys.remove(&idx) {
                        crate::shortcuts::handle_shortcut_event(
                            &app,
                            &binding.action,
                            &binding.activation_mode,
                            KeyEventType::Released,
                        );
                    }
                }
            }

            // Optional key-state fallback is strictly disabled unless explicitly enabled by user
            if polling_enabled {
                let mut currently_matched = HashSet::new();
                for (idx, binding) in bindings_snapshot.iter().enumerate() {
                    let hotkey_id = (idx as i32) + 1;
                    if registered.contains_key(&hotkey_id) || binding.keys.is_empty() {
                        continue;
                    }
                    if check_keys_pressed_opt_in(&binding.keys) {
                        currently_matched.insert(idx);
                        if !active_fallback.contains_key(&idx) {
                            active_fallback.insert(idx, binding.clone());
                            crate::shortcuts::handle_shortcut_event(
                                &app,
                                &binding.action,
                                &binding.activation_mode,
                                KeyEventType::Pressed,
                            );
                        }
                    }
                }
                let released_fallback: Vec<usize> = active_fallback
                    .keys()
                    .filter(|idx| !currently_matched.contains(idx))
                    .copied()
                    .collect();
                for idx in released_fallback {
                    if let Some(binding) = active_fallback.remove(&idx) {
                        crate::shortcuts::handle_shortcut_event(
                            &app,
                            &binding.action,
                            &binding.activation_mode,
                            KeyEventType::Released,
                        );
                    }
                }
            } else if !active_fallback.is_empty() {
                active_fallback.clear();
            }

            std::thread::sleep(Duration::from_millis(25));
        }
    });
}
