// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, Modifiers, Code};
use tauri_plugin_store::StoreExt;
use serde_json::json;

#[tauri::command]
async fn run_ai_fix<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    // 1. Get Config from Store
    let stores = app.store("settings.json").map_err(|e| e.to_string())?;
    let api_key = stores.get("api_key").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let preprompt = stores.get("preprompt").and_then(|v| v.as_str()).unwrap_or("Fix typos:").to_string();

    // 2. Get Clipboard Text
    let clipboard_text = app.clipboard().read_text().map_err(|e| e.to_string())?
        .unwrap_or_default();

    // 3. Call Google Gemini API (using reqwest)
    let client = reqwest::Client::new();
    let response = client
        .post(format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent?key={}", api_key))
        .json(&json!({
            "contents": [{ "parts": [{ "text": format!("{} \n\n {}", preprompt, clipboard_text) }] }]
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let json_res: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    let fixed_text = json_res["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("Error parsing response");

    // 4. Write back to Clipboard
    app.clipboard().write_text(fixed_text.to_string()).map_err(|e| e.to_string())?;
    
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
            let tray = tauri::tray::TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { .. } = event {
                        let window = tray.app_handle().get_webview_window("main").unwrap();
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                })
                .build(app)?;

            // Register Alt+V Shortcut
            let fix_shortcut = Shortcut::new(Some(Modifiers::ALT), Code::KeyV);
            app.global_shortcut().on_shortcut(fix_shortcut, |app, _shortcut, event| {
                if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                    let handle = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = run_ai_fix(handle).await;
                    });
                }
            })?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![run_ai_fix])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
