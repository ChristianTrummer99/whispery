mod audio;
mod transcribe;
mod transform;

use audio::AudioRecorder;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{Emitter, Manager};

struct AppState {
    recorder: Mutex<AudioRecorder>,
}

#[derive(Clone, Serialize, Deserialize)]
struct RecordingStatus {
    state: String,
    message: Option<String>,
}

#[tauri::command]
fn list_audio_devices(state: tauri::State<AppState>) -> Vec<String> {
    let recorder = state.recorder.lock().unwrap();
    recorder.list_devices()
}

#[tauri::command]
fn get_audio_level(state: tauri::State<AppState>) -> f32 {
    let recorder = state.recorder.lock().unwrap();
    recorder.get_audio_level()
}

fn show_overlay(app: &tauri::AppHandle) {
    #[cfg(target_os = "macos")]
    {
        use tauri_nspanel::ManagerExt;
        if let Ok(panel) = app.get_webview_panel("overlay") {
            panel.show();
            panel.order_front_regardless();
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        if let Some(overlay) = app.get_webview_window("overlay") {
            let _ = overlay.show();
        }
    }
}

fn hide_overlay(app: &tauri::AppHandle) {
    #[cfg(target_os = "macos")]
    {
        use tauri_nspanel::ManagerExt;
        if let Ok(panel) = app.get_webview_panel("overlay") {
            panel.hide();
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        if let Some(overlay) = app.get_webview_window("overlay") {
            let _ = overlay.hide();
        }
    }
}

fn emit_recording_status(app: &tauri::AppHandle, state: &str, message: Option<String>) {
    let _ = app.emit(
        "recording-status",
        RecordingStatus {
            state: state.into(),
            message,
        },
    );
}

#[tauri::command]
fn start_recording(
    state: tauri::State<AppState>,
    app: tauri::AppHandle,
    device_name: Option<String>,
) -> Result<(), String> {
    eprintln!("[whispery] Recording started (device={:?})", device_name);
    let mut recorder = state.recorder.lock().unwrap();
    recorder.start(device_name.as_deref())?;

    let _ = app.emit(
        "recording-status",
        RecordingStatus {
            state: "listening".into(),
            message: None,
        },
    );

    show_overlay(&app);
    Ok(())
}

#[tauri::command]
fn cancel_recording(state: tauri::State<AppState>, app: tauri::AppHandle) {
    eprintln!("[whispery] Recording cancelled by user");
    let mut recorder = state.recorder.lock().unwrap();
    recorder.cancel();

    let _ = app.emit(
        "recording-status",
        RecordingStatus {
            state: "idle".into(),
            message: None,
        },
    );

    hide_overlay(&app);
}

#[tauri::command]
fn hide_overlay_cmd(app: tauri::AppHandle) {
    hide_overlay(&app);
}

#[tauri::command]
fn paste_to_input(text: String) -> Result<String, String> {
    eprintln!("[whispery] Pasting to input: \"{}\"", text);
    use arboard::Clipboard;
    use enigo::{Direction, Enigo, Key, Keyboard, Settings};

    let mut clipboard =
        Clipboard::new().map_err(|e| format!("Clipboard::new failed: {e}"))?;
    clipboard
        .set_text(&text)
        .map_err(|e| format!("set_text failed: {e}"))?;

    std::thread::sleep(std::time::Duration::from_millis(100));

    let mut enigo =
        Enigo::new(&Settings::default()).map_err(|e| format!("Enigo::new failed: {e}"))?;

    #[cfg(target_os = "macos")]
    {
        enigo
            .key(Key::Meta, Direction::Press)
            .map_err(|e| format!("Meta press: {e}"))?;
        enigo
            .key(Key::Unicode('v'), Direction::Click)
            .map_err(|e| format!("v click: {e}"))?;
        enigo
            .key(Key::Meta, Direction::Release)
            .map_err(|e| format!("Meta release: {e}"))?;
    }

    #[cfg(not(target_os = "macos"))]
    {
        enigo
            .key(Key::Control, Direction::Press)
            .map_err(|e| format!("Ctrl press: {e}"))?;
        enigo
            .key(Key::Unicode('v'), Direction::Click)
            .map_err(|e| format!("v click: {e}"))?;
        enigo
            .key(Key::Control, Direction::Release)
            .map_err(|e| format!("Ctrl release: {e}"))?;
    }

    Ok("ok".into())
}

#[tauri::command]
async fn stop_recording_and_process(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    openai_api_key: String,
    llm_api_key: String,
    llm_api_url: String,
    llm_model: String,
    prompt_template: String,
    skip_transform: bool,
    whisper_language: Option<String>,
    glossary_prompt: Option<String>,
    save_debug_audio: Option<bool>,
) -> Result<String, String> {
    let wav_data = {
        let mut recorder = state.recorder.lock().unwrap();
        let samples = recorder.stop();
        let sample_count = samples.len();
        let capture_rate = recorder.capture_rate();
        eprintln!("[whispery] Recording stopped — {sample_count} samples, capture_rate={capture_rate}Hz, duration={:.2}s", sample_count as f64 / capture_rate as f64);
        if samples.is_empty() {
            eprintln!("[whispery] ERROR: No audio samples");
            let _ = app.emit(
                "recording-status",
                RecordingStatus {
                    state: "error".into(),
                    message: Some("No audio recorded".into()),
                },
            );
            return Err("No audio captured".into());
        }
        let wav = recorder.encode_wav(&samples);
        eprintln!("[whispery] WAV encoded — {} bytes", wav.len());

        if save_debug_audio.unwrap_or(false) {
            if let Some(mut path) = dirs::desktop_dir() {
                path.push("whispery_debug.wav");
                match std::fs::write(&path, &wav) {
                    Ok(_) => eprintln!("[whispery] Debug audio saved to {}", path.display()),
                    Err(e) => eprintln!("[whispery] Failed to save debug audio: {e}"),
                }
            }
        }

        wav
    };

    let _ = app.emit(
        "recording-status",
        RecordingStatus {
            state: "processing".into(),
            message: Some("Transcribing...".into()),
        },
    );

    if openai_api_key.trim().is_empty() {
        let message = "OpenAI API key is missing".to_string();
        emit_recording_status(&app, "error", Some(message.clone()));
        return Err(message);
    }

    let lang = whisper_language.as_deref().unwrap_or("en");
    let glossary = glossary_prompt.as_deref();
    eprintln!("[whispery] Transcription started (lang={lang}, glossary={})", glossary.unwrap_or("none"));
    let transcribed = match transcribe::transcribe_whisper(&openai_api_key, wav_data, lang, glossary).await {
        Ok(value) => value,
        Err(e) => {
            let message = format!("Transcription failed: {e}");
            emit_recording_status(&app, "error", Some(message.clone()));
            return Err(message);
        }
    };
    eprintln!("[whispery] Transcription result: \"{}\"", transcribed);

    if transcribed.is_empty() {
        eprintln!("[whispery] ERROR: Empty transcription");
        let _ = app.emit(
            "recording-status",
            RecordingStatus {
                state: "error".into(),
                message: Some("No speech detected".into()),
            },
        );
        return Err("Empty transcription".into());
    }

    let final_text = if skip_transform || prompt_template.is_empty() {
        eprintln!("[whispery] No transformation — using raw transcription");
        transcribed
    } else {
        let _ = app.emit(
            "recording-status",
            RecordingStatus {
                state: "processing".into(),
                message: Some("Transforming...".into()),
            },
        );
        eprintln!("[whispery] Transformation started (model={llm_model})");
        let result = match transform::transform_text(
            &llm_api_key,
            &llm_api_url,
            &llm_model,
            &prompt_template,
            &transcribed,
        )
        .await {
            Ok(value) => value,
            Err(e) => {
                let message = format!("Transformation failed: {e}");
                emit_recording_status(&app, "error", Some(message.clone()));
                return Err(message);
            }
        };
        eprintln!("[whispery] Transformation result: \"{}\"", result);
        result
    };

    eprintln!("[whispery] Final output: \"{}\"", final_text);
    let _ = app.emit(
        "recording-status",
        RecordingStatus {
            state: "success".into(),
            message: Some(final_text.clone()),
        },
    );

    Ok(final_text)
}

#[cfg(target_os = "macos")]
mod macos_panel {
    use tauri::Manager;
    use tauri_nspanel::{
        objc2_app_kit::NSWindowCollectionBehavior, tauri_panel, StyleMask, WebviewWindowExt,
    };

    tauri_panel! {
        panel!(OverlayPanel {
            config: {
                can_become_key_window: false,
                is_floating_panel: true
            }
        })
    }

    pub fn setup_overlay(app: &tauri::AppHandle) {
        if let Some(overlay_window) = app.get_webview_window("overlay") {
            match overlay_window.to_panel::<OverlayPanel>() {
                Ok(panel) => {
                    panel.set_level(8);

                    let style = StyleMask::empty().nonactivating_panel().resizable();
                    panel.set_style_mask(style.value());

                    panel.set_collection_behavior(
                        NSWindowCollectionBehavior::CanJoinAllSpaces
                            | NSWindowCollectionBehavior::FullScreenAuxiliary,
                    );

                    panel.set_hides_on_deactivate(false);
                    panel.set_has_shadow(false);
                    panel.set_transparent(true);
                    panel.set_movable_by_window_background(true);
                }
                Err(e) => {
                    eprintln!("Failed to convert overlay to panel: {e}");
                }
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }));

    #[cfg(target_os = "macos")]
    {
        builder = builder.plugin(tauri_nspanel::init());
    }

    builder
        .manage(AppState {
            recorder: Mutex::new(AudioRecorder::new()),
        })
        .invoke_handler(tauri::generate_handler![
            list_audio_devices,
            get_audio_level,
            start_recording,
            cancel_recording,
            stop_recording_and_process,
            hide_overlay_cmd,
            paste_to_input,
        ])
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { .. } = event {
                    window.app_handle().exit(0);
                }
            }
        })
        .setup(|app| {
            #[cfg(desktop)]
            {
                use tauri::menu::{MenuBuilder, MenuItemBuilder};
                use tauri::tray::TrayIconBuilder;

                #[cfg(target_os = "macos")]
                {
                    macos_panel::setup_overlay(&app.handle());
                }

                #[cfg(not(target_os = "macos"))]
                {
                    if let Some(overlay) = app.get_webview_window("overlay") {
                        let _ = overlay.set_always_on_top(true);
                    }
                }

                let show = MenuItemBuilder::with_id("show", "Show Whispery").build(app)?;
                let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
                let menu = MenuBuilder::new(app)
                    .item(&show)
                    .separator()
                    .item(&quit)
                    .build()?;

                let _tray = TrayIconBuilder::new()
                    .menu(&menu)
                    .tooltip("Whispery")
                    .on_menu_event(|app, event| match event.id().as_ref() {
                        "show" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    })
                    .build(app)?;
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
