// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn fetch_models(api_key: String) -> Result<Vec<String>, String> {
    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "https://generativelanguage.googleapis.com/v1beta/models?key={}",
            api_key
        ))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let json_res: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;

    if let Some(error) = json_res.get("error") {
        return Err(format!("API Error: {:?}", error));
    }

    let models: Vec<String> = json_res["models"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|m| {
            // Check if model supports generateContent
            let methods = m["supportedGenerationMethods"]
                .as_array()
                .map(|v| v.iter().filter_map(|s| s.as_str()).collect::<Vec<_>>())
                .unwrap_or_default();

            let supports_generate = methods.contains(&"generateContent");

            let name = m["name"].as_str().unwrap_or("");
            let is_gemini = name.contains("gemini");

            if supports_generate && is_gemini {
                Some(name.replace("models/", ""))
            } else {
                None
            }
        })
        .collect();

    Ok(models)
}

use enigo::{Enigo, Key, Keyboard, Settings};
use serde_json::json;
use tauri::menu::{Menu, MenuItem};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, Runtime, WindowEvent};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_store::StoreExt;

// Animation timing constants (in milliseconds)
const FRAME_DURATION_MS: u64 = 150;
const INTRO_FRAMES: u64 = 5;
const OUTRO_FRAMES: u64 = 4;
const MIN_RUNNING_CYCLES: u64 = 2;
const FRAMES_PER_CYCLE: u64 = 2;

// AI Model configuration
const GEMINI_MODEL: &str = "gemini-2.5-flash";

#[tauri::command]
async fn run_ai_fix<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    println!("Starting AI fix...");

    // Show duck animation overlay
    if let Some(duck_window) = app.get_webview_window("duck") {
        println!("Duck window found!");
        // Position bottom-right above taskbar
        if let Ok(monitors) = duck_window.available_monitors() {
            if let Some(primary) = monitors.first() {
                let screen_size = primary.size();
                let x = screen_size.width as i32 - 120; // window width (100) + margin (20)
                let y = screen_size.height as i32 - 150; // window height (100) + taskbar (~50)
                println!("Positioning duck at ({}, {})", x, y);
                let _ = duck_window.set_position(PhysicalPosition::new(x, y));
            }
        }
        match duck_window.show() {
            Ok(_) => println!("Duck window shown successfully"),
            Err(e) => println!("Failed to show duck window: {:?}", e),
        }
        match app.emit_to("duck", "animation-phase", "start") {
            Ok(_) => println!("Emitted animation-phase start"),
            Err(e) => println!("Failed to emit to duck: {:?}", e),
        }
    } else {
        println!("Duck window NOT found!");
    }

    // Track start time for minimum display duration
    let start_time = std::time::Instant::now();

    // Run the main logic, capturing result
    let result = run_ai_fix_inner(&app).await;

    // Ensure minimum display time: intro + at least MIN_RUNNING_CYCLES
    let min_display_ms = FRAME_DURATION_MS * (INTRO_FRAMES + MIN_RUNNING_CYCLES * FRAMES_PER_CYCLE);
    let min_display_time = std::time::Duration::from_millis(min_display_ms);
    let elapsed = start_time.elapsed();
    if elapsed < min_display_time {
        std::thread::sleep(min_display_time - elapsed);
    }

    // Always hide duck window after completion (success or error)
    if let Some(duck_window) = app.get_webview_window("duck") {
        let _ = app.emit_to("duck", "animation-phase", "finish");
        // Wait for outro animation to complete
        let outro_wait_ms = FRAME_DURATION_MS * OUTRO_FRAMES + 100; // +100ms buffer
        std::thread::sleep(std::time::Duration::from_millis(outro_wait_ms));
        let _ = duck_window.hide();
    }

    result
}

async fn run_ai_fix_inner<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    // 1. Get Config from Store
    let stores = app.store("settings.json").map_err(|e| e.to_string())?;

    let path = app.path().app_data_dir().map(|p| p.join("settings.json"));
    println!("Loading config from: {:?}", path);

    // The store should be loaded. get() returns Option<JsonValue>.
    let api_key = stores
        .get("api_key")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();

    // Mask key for logging
    let masked_key = if api_key.len() > 4 {
        format!("{}...", &api_key[0..4])
    } else {
        "Empty/Short".to_string()
    };
    println!("API Key found: {}", masked_key);

    let preprompt = stores
        .get("preprompt")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "Fix typos:".to_string());

    let turbo_mode = stores
        .get("turbo_mode")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let model = stores
        .get("model")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| GEMINI_MODEL.to_string());

    let show_duck = stores
        .get("show_duck")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let show_notification = stores
        .get("show_notification")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    println!(
        "Turbo: {}, Model: {}, Duck: {}, Notif: {}",
        turbo_mode, model, show_duck, show_notification
    );

    // Show duck animation overlay (if enabled)
    if show_duck {
        if let Some(duck_window) = app.get_webview_window("duck") {
            println!("Duck window found!");
            // Position bottom-right above taskbar
            if let Ok(monitors) = duck_window.available_monitors() {
                if let Some(primary) = monitors.first() {
                    let screen_size = primary.size();
                    let x = screen_size.width as i32 - 120; // window width (100) + margin (20)
                    let y = screen_size.height as i32 - 150; // window height (100) + taskbar (~50)
                    println!("Positioning duck at ({}, {})", x, y);
                    let _ = duck_window.set_position(PhysicalPosition::new(x, y));
                }
            }
            match duck_window.show() {
                Ok(_) => println!("Duck window shown successfully"),
                Err(e) => println!("Failed to show duck window: {:?}", e),
            }
            match app.emit_to("duck", "animation-phase", "start") {
                Ok(_) => println!("Emitted animation-phase start"),
                Err(e) => println!("Failed to emit to duck: {:?}", e),
            }
        } else {
            println!("Duck window NOT found!");
        }
    }

    // Track start time for minimum display duration
    let start_time = std::time::Instant::now();

    // Call API and process result
    let result = run_ai_fix_logic(
        &app,
        &api_key,
        &preprompt,
        &model,
        turbo_mode,
        show_notification,
    )
    .await;

    // Handle duck outro (if enabled)
    if show_duck {
        // Ensure minimum display time only if we showed the duck
        let min_display_ms =
            FRAME_DURATION_MS * (INTRO_FRAMES + MIN_RUNNING_CYCLES * FRAMES_PER_CYCLE);
        let min_display_time = std::time::Duration::from_millis(min_display_ms);
        let elapsed = start_time.elapsed();
        if elapsed < min_display_time {
            std::thread::sleep(min_display_time - elapsed);
        }

        // Always hide duck window after completion (success or error)
        if let Some(duck_window) = app.get_webview_window("duck") {
            let _ = app.emit_to("duck", "animation-phase", "finish");
            // Wait for outro animation to complete
            let outro_wait_ms = FRAME_DURATION_MS * OUTRO_FRAMES + 100; // +100ms buffer
            std::thread::sleep(std::time::Duration::from_millis(outro_wait_ms));
            let _ = duck_window.hide();
        }
    }

    result
}

// Logic separated for cleaner conditional handling
async fn run_ai_fix_logic<R: Runtime>(
    app: &AppHandle<R>,
    api_key: &str,
    preprompt: &str,
    model: &str,
    turbo_mode: bool,
    show_notification: bool,
) -> Result<(), String> {
    println!("Turbo Mode: {}, Model: {}", turbo_mode, model);

    // 2. If Turbo Mode, simulate Ctrl+C to copy selection
    if turbo_mode {
        let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
        // Release Alt key first (since Alt+V triggered us, Alt might still be held)
        enigo
            .key(Key::Alt, enigo::Direction::Release)
            .map_err(|e| e.to_string())?;
        std::thread::sleep(std::time::Duration::from_millis(50));
        // Now simulate Ctrl+C
        enigo
            .key(Key::Control, enigo::Direction::Press)
            .map_err(|e| e.to_string())?;
        enigo
            .key(Key::Unicode('c'), enigo::Direction::Click)
            .map_err(|e| e.to_string())?;
        enigo
            .key(Key::Control, enigo::Direction::Release)
            .map_err(|e| e.to_string())?;
        std::thread::sleep(std::time::Duration::from_millis(200)); // Wait for clipboard
    }

    // 3. Get Clipboard Text
    // Handle clipboard errors gracefully (e.g. empty)
    let clipboard_text = app.clipboard().read_text().unwrap_or_default();
    println!("Clipboard text length: {}", clipboard_text.len());

    // 3. Call Google Gemini API (using reqwest)
    let client = reqwest::Client::new();
    println!("Sending request to {}...", model);
    let response = client
        .post(format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}", model, api_key))
        .json(&json!({
            "contents": [{ "parts": [{ "text": format!("{} \n\n INSTRUCTIONS:\n1. Fix typos and grammar.\n2. STRICTLY PRESERVE all original newlines, paragraph breaks, and indentation.\n3. Do NOT merge lines.\n\nINPUT TEXT:\n```\n{}\n```", preprompt, clipboard_text) }] }],
             "generationConfig": {
                "responseMimeType": "application/json",
                "responseSchema": {
                    "type": "object",
                    "properties": {
                        "fixed_text": { "type": "string" }
                    }
                }
            }
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let json_res: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;

    if let Some(error) = json_res.get("error") {
        println!("API Error: {:?}", error);
        return Err(format!("API Error: {:?}", error));
    }

    // Parse the structured output
    // The model returns a stringified JSON in the text field when responseMimeType is application/json
    let content_text = json_res["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("");

    println!("Raw API Response Text: {}", content_text);

    let fixed_text =
        if let Ok(parsed_inner) = serde_json::from_str::<serde_json::Value>(content_text) {
            parsed_inner["fixed_text"]
                .as_str()
                .unwrap_or(content_text)
                .to_string()
        } else {
            // Fallback if parsing fails (shouldn't happen with valid structured output)
            content_text.to_string()
        };

    // Clean up potentially double-escaped newlines from the model
    // Sometimes models output literal "\n" strings instead of actual newlines in the JSON logic
    let fixed_text = fixed_text.replace("\\n", "\n").replace("\\r", "");

    println!("Received response. Writing to clipboard.");

    // 4. Write back to Clipboard
    app.clipboard()
        .write_text(fixed_text)
        .map_err(|e| e.to_string())?;

    // 6. If Turbo Mode, simulate Ctrl+V to paste
    if turbo_mode {
        let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
        enigo
            .key(Key::Control, enigo::Direction::Press)
            .map_err(|e| e.to_string())?;
        enigo
            .key(Key::Unicode('v'), enigo::Direction::Click)
            .map_err(|e| e.to_string())?;
        enigo
            .key(Key::Control, enigo::Direction::Release)
            .map_err(|e| e.to_string())?;
    }

    // 5. Notification (Show after pasting to avoid stealing focus)
    if show_notification {
        app.notification()
            .builder()
            .title("Typo Fixed")
            .body("Text corrected and copied to clipboard.")
            .show()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // Setup Tray Icon
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let open_i = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open_i, &quit_i])?;

            let handle = app.handle().clone();
            let _tray = tauri::tray::TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |_app, event| match event.id.as_ref() {
                    "quit" => {
                        std::process::exit(0);
                    }
                    "open" => {
                        if let Some(window) = handle.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Register Alt+Z Shortcut
            let fix_shortcut = Shortcut::new(Some(Modifiers::CONTROL), Code::KeyQ);
            app.global_shortcut()
                .on_shortcut(fix_shortcut, |app, _shortcut, event| {
                    if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        println!("Shortcut Ctrl+Q pressed!");
                        let handle = app.clone();
                        tauri::async_runtime::spawn(async move {
                            if let Err(e) = run_ai_fix(handle).await {
                                println!("Error running AI fix: {}", e);
                            }
                        });
                    }
                })?;

            // Show main window only if API key is NOT set (window starts hidden)
            let should_show = if let Ok(store) = app.store("settings.json") {
                if let Some(key) = store.get("api_key") {
                    // Hide if key exists and is not empty
                    !key.as_str().map(|s| !s.is_empty()).unwrap_or(false)
                } else {
                    true // No key, show window
                }
            } else {
                true // No store, show window
            };

            if should_show {
                println!("No API key found, showing main window");
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                }
            } else {
                println!("API key found, staying hidden");
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet, run_ai_fix, fetch_models])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::WindowEvent {
                event: WindowEvent::CloseRequested { api, .. },
                ..
            } = event
            {
                let window = app_handle.get_webview_window("main").unwrap();
                window.hide().unwrap();
                api.prevent_close();
            }
        });
}
