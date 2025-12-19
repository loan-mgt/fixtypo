// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

use enigo::{Enigo, Key, Keyboard, Settings};
use serde_json::json;
use tauri::menu::{Menu, MenuItem};
use tauri::{AppHandle, Manager, Runtime, WindowEvent};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_store::StoreExt;

#[tauri::command]
async fn run_ai_fix<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    println!("Starting AI fix...");
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

    println!("Turbo Mode: {}", turbo_mode);

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
    println!("Sending request to Gemini 2.5 Flash...");
    let response = client
        .post(format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}", api_key))
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
    app.notification()
        .builder()
        .title("Typo Fixed")
        .body("Text corrected and copied to clipboard.")
        .show()
        .map_err(|e| e.to_string())?;

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

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet, run_ai_fix])
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
