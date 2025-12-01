use tauri::{AppHandle, Manager, State, WebviewWindow, WebviewWindowBuilder, WebviewUrl, Url};
use crate::state::{AppState, Profile};
use crate::scripting;
use std::sync::{Arc, Mutex};
use serde_json::Value;
use std::fs;
use std::path::Path;

fn ensure_target_window(app: &AppHandle) -> Option<WebviewWindow> {
    if let Some(w) = app.get_webview_window("target") {
        return Some(w);
    }

    let state_handle = app.state::<Arc<Mutex<AppState>>>();
    let state = state_handle.lock().unwrap();

    let url = Url::parse("about:blank").unwrap();
    let mut builder = WebviewWindowBuilder::new(
        app,
        "target",
        WebviewUrl::External(url)
    )
    .title("Phantom Browser Target")
    .inner_size(1024.0, 768.0);

    if let Some(profile) = &state.current_profile {
        let js = format!(r#"
            Object.defineProperty(navigator, 'userAgent', {{
                get: function () {{ return "{}"; }}
            }});
        "#, profile.user_agent);
        builder = builder.initialization_script(&js);
    }

    match builder.build() {
        Ok(w) => Some(w),
        Err(e) => {
            eprintln!("Failed to create target window: {}", e);
            None
        }
    }
}

#[tauri::command]
pub fn execute_script(script: String, state: State<Arc<Mutex<AppState>>>, app: AppHandle) {
    if ensure_target_window(&app).is_some() {
        scripting::run_script(script, app, state.inner().clone());
    } else {
        eprintln!("Could not create target window");
    }
}

#[tauri::command]
pub fn script_callback(id: String, data: Value, state: State<Arc<Mutex<AppState>>>) {
    let mut state = state.lock().unwrap();
    if let Some(tx) = state.pending_callbacks.remove(&id) {
        let _ = tx.send(data);
    }
}

#[tauri::command]
pub fn get_profiles() -> Vec<Profile> {
    vec![
        Profile {
            name: "Desktop Chrome".into(),
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".into()
        },
        Profile {
            name: "Mobile iPhone".into(),
            user_agent: "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1".into()
        },
        Profile {
             name: "Linux Firefox".into(),
             user_agent: "Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/115.0".into()
        }
    ]
}

#[tauri::command]
pub fn set_profile(profile_name: String, state: State<Arc<Mutex<AppState>>>, app: AppHandle) {
    let profiles = get_profiles();
    let profile = profiles.into_iter().find(|p| p.name == profile_name);

    if let Some(p) = profile {
        {
            let mut state = state.lock().unwrap();
            state.current_profile = Some(p.clone());
        }

        // Close target window if open, so it gets recreated with new UA on next run
        if let Some(w) = app.get_webview_window("target") {
            let _ = w.close();
        }
    }
}

#[tauri::command]
pub fn list_scripts() -> Vec<String> {
    let path = Path::new("scripts");
    if !path.exists() {
        let _ = fs::create_dir(path);
    }

    let mut scripts = Vec::new();
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
             if let Ok(name) = entry.file_name().into_string() {
                 scripts.push(name);
             }
        }
    }
    scripts
}

#[tauri::command]
pub fn save_script(filename: String, content: String) -> Result<(), String> {
    let path = Path::new("scripts");
    if !path.exists() {
        let _ = fs::create_dir(path);
    }

    let safe_filename = Path::new(&filename).file_name().ok_or("Invalid filename")?;
    let target = path.join(safe_filename);

    fs::write(target, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn read_script(filename: String) -> Result<String, String> {
    let path = Path::new("scripts").join(filename);
    fs::read_to_string(path).map_err(|e| e.to_string())
}
