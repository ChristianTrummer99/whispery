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


#[tauri::command]
fn start_recording(
    state: tauri::State<AppState>,
    app: tauri::AppHandle,
    device_name: Option<String>,
) -> Result<(), String> {
    let mut recorder = state.recorder.lock().unwrap();
    recorder.start(device_name.as_deref())?;

    let _ = app.emit("recording-status", RecordingStatus {
        state: "listening".into(),
        message: None,
    });

    show_overlay(&app);

    Ok(())
}

#[tauri::command]
fn hide_overlay_cmd(app: tauri::AppHandle) {
    hide_overlay(&app);
}

#[tauri::command]
fn test_clipboard_only(text: String) -> Result<String, String> {
    eprintln!("whispery test: clipboard_only called with: {text}");
    use arboard::Clipboard;
    let mut clipboard = Clipboard::new().map_err(|e| format!("Clipboard::new failed: {e}"))?;
    clipboard.set_text(&text).map_err(|e| format!("set_text failed: {e}"))?;
    eprintln!("whispery test: clipboard set OK");
    Ok("clipboard OK".into())
}

#[tauri::command]
fn test_enigo_only() -> Result<String, String> {
    eprintln!("whispery test: enigo_only called");
    use enigo::{Enigo, Key, Keyboard, Settings, Direction};

    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("Enigo::new failed: {e}"))?;
    eprintln!("whispery test: enigo created OK");

    #[cfg(target_os = "macos")]
    {
        enigo.key(Key::Meta, Direction::Press).map_err(|e| format!("Meta press: {e}"))?;
        enigo.key(Key::Unicode('v'), Direction::Click).map_err(|e| format!("v click: {e}"))?;
        enigo.key(Key::Meta, Direction::Release).map_err(|e| format!("Meta release: {e}"))?;
    }

    #[cfg(not(target_os = "macos"))]
    {
        enigo.key(Key::Control, Direction::Press).map_err(|e| format!("Ctrl press: {e}"))?;
        enigo.key(Key::Unicode('v'), Direction::Click).map_err(|e| format!("v click: {e}"))?;
        enigo.key(Key::Control, Direction::Release).map_err(|e| format!("Ctrl release: {e}"))?;
    }

    eprintln!("whispery test: enigo paste simulated OK");
    Ok("enigo OK".into())
}

#[tauri::command]
fn test_paste_combined(text: String) -> Result<String, String> {
    eprintln!("whispery test: paste_combined called");
    use arboard::Clipboard;
    use enigo::{Enigo, Key, Keyboard, Settings, Direction};

    let mut clipboard = Clipboard::new().map_err(|e| format!("Clipboard::new failed: {e}"))?;
    clipboard.set_text(&text).map_err(|e| format!("set_text failed: {e}"))?;
    eprintln!("whispery test: clipboard set");

    std::thread::sleep(std::time::Duration::from_millis(100));

    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("Enigo::new failed: {e}"))?;
    eprintln!("whispery test: enigo created");

    #[cfg(target_os = "macos")]
    {
        enigo.key(Key::Meta, Direction::Press).map_err(|e| format!("Meta press: {e}"))?;
        enigo.key(Key::Unicode('v'), Direction::Click).map_err(|e| format!("v click: {e}"))?;
        enigo.key(Key::Meta, Direction::Release).map_err(|e| format!("Meta release: {e}"))?;
    }

    #[cfg(not(target_os = "macos"))]
    {
        enigo.key(Key::Control, Direction::Press).map_err(|e| format!("Ctrl press: {e}"))?;
        enigo.key(Key::Unicode('v'), Direction::Click).map_err(|e| format!("v click: {e}"))?;
        enigo.key(Key::Control, Direction::Release).map_err(|e| format!("Ctrl release: {e}"))?;
    }

    eprintln!("whispery test: paste simulated OK");
    Ok("paste OK".into())
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
) -> Result<String, String> {
    let wav_data = {
        let mut recorder = state.recorder.lock().unwrap();
        let samples = recorder.stop();
        if samples.is_empty() {
            let _ = app.emit("recording-status", RecordingStatus {
                state: "error".into(),
                message: Some("No audio recorded".into()),
            });
            return Err("No audio captured".into());
        }
        recorder.encode_wav(&samples)
    };

    let _ = app.emit("recording-status", RecordingStatus {
        state: "processing".into(),
        message: Some("Transcribing...".into()),
    });

    let lang = whisper_language.as_deref().unwrap_or("en");
    let transcribed = transcribe::transcribe_whisper(&openai_api_key, wav_data, lang).await?;

    if transcribed.is_empty() {
        let _ = app.emit("recording-status", RecordingStatus {
            state: "error".into(),
            message: Some("No speech detected".into()),
        });
        return Err("Empty transcription".into());
    }

    let final_text = if skip_transform || prompt_template.is_empty() {
        transcribed
    } else {
        let _ = app.emit("recording-status", RecordingStatus {
            state: "processing".into(),
            message: Some("Transforming...".into()),
        });
        transform::transform_text(
            &llm_api_key,
            &llm_api_url,
            &llm_model,
            &prompt_template,
            &transcribed,
        )
        .await?
    };

    let _ = app.emit("recording-status", RecordingStatus {
        state: "success".into(),
        message: Some(final_text.clone()),
    });

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
                    // NSPanel level: floating (8) is sufficient — the NSPanel
                    // mechanism itself handles appearing over fullscreen apps.
                    panel.set_level(8);

                    let style = StyleMask::empty()
                        .nonactivating_panel()
                        .resizable();
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
        .plugin(tauri_plugin_store::Builder::new().build());

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
            start_recording,
            stop_recording_and_process,
            hide_overlay_cmd,
            test_clipboard_only,
            test_enigo_only,
            test_paste_combined,
        ])
        .setup(|app| {
            #[cfg(desktop)]
            {
                use tauri::tray::TrayIconBuilder;
                use tauri::menu::{MenuBuilder, MenuItemBuilder};

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
                    .on_menu_event(|app, event| {
                        match event.id().as_ref() {
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
                        }
                    })
                    .build(app)?;
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
