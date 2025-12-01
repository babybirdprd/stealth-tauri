use tauri::{AppHandle, Manager, State, WebviewWindow, WebviewWindowBuilder, WebviewUrl, Url};
use crate::state::{AppState, Profile};
use crate::scripting;
use crate::fingerprint;
use crate::proxy;
use std::sync::{Arc, Mutex};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

fn get_profiles_path() -> PathBuf {
    Path::new("profiles.json").to_path_buf()
}

fn load_profiles_from_disk() -> Vec<Profile> {
    let path = get_profiles_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(profiles) = serde_json::from_str::<Vec<Profile>>(&content) {
                return profiles;
            }
        }
    }

    // Default profiles
    let defaults = vec![
        Profile {
            name: "Desktop Chrome".into(),
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".into(),
            seed: 12345,
            proxy: None,
        },
        Profile {
            name: "Mobile iPhone".into(),
            user_agent: "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1".into(),
            seed: 67890,
            proxy: None,
        },
        Profile {
             name: "Linux Firefox".into(),
             user_agent: "Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/115.0".into(),
             seed: 112233,
             proxy: None,
        }
    ];

    save_profiles_to_disk(&defaults);
    defaults
}

fn save_profiles_to_disk(profiles: &[Profile]) {
    let path = get_profiles_path();
    let json = serde_json::to_string_pretty(profiles).unwrap_or_default();
    let _ = fs::write(path, json);
}

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

    // Apply Profile Settings
    if let Some(profile) = &state.current_profile {
        // 1. User Agent
        let ua_script = format!(r#"
            Object.defineProperty(navigator, 'userAgent', {{
                get: function () {{ return "{}"; }}
            }});
        "#, profile.user_agent);
        builder = builder.initialization_script(&ua_script);

        // 2. Fingerprint
        let fingerprint_script = fingerprint::generate_injection_script(profile.seed);
        builder = builder.initialization_script(&fingerprint_script);
    }

    // 3. Proxy Configuration
    if state.proxy_port > 0 {
        let proxy_url = Url::parse(&format!("http://127.0.0.1:{}", state.proxy_port)).unwrap();
        builder = builder.proxy_url(proxy_url);

        // Ignore certificate errors
        builder = builder.additional_browser_args("--ignore-certificate-errors");
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
pub async fn execute_script(script: String, state: State<'_, Arc<Mutex<AppState>>>, app: AppHandle) -> Result<(), String> {
    // Ensure proxy is running if not already
    let needs_start = {
        let s = state.lock().unwrap();
        s.proxy_port == 0
    };

    if needs_start {
        proxy::restart_proxy(app.clone(), state.inner().clone()).await;
    }

    if ensure_target_window(&app).is_some() {
        scripting::run_script(script, app, state.inner().clone());
        Ok(())
    } else {
        Err("Could not create target window".into())
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
    load_profiles_from_disk()
}

#[tauri::command]
pub async fn set_profile(profile_name: String, state: State<'_, Arc<Mutex<AppState>>>, app: AppHandle) -> Result<(), String> {
    let profiles = load_profiles_from_disk();
    let profile = profiles.into_iter().find(|p| p.name == profile_name);

    if let Some(p) = profile {
        {
            let mut state = state.lock().unwrap();
            state.current_profile = Some(p.clone());
        }

        // Restart Proxy for new profile
        proxy::restart_proxy(app.clone(), state.inner().clone()).await;

        // Close target window if open, so it gets recreated with new UA/Proxy on next run
        if let Some(w) = app.get_webview_window("target") {
            let _ = w.close();
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn save_profile_config(profile: Profile, state: State<'_, Arc<Mutex<AppState>>>, app: AppHandle) -> Result<(), String> {
    let mut profiles = load_profiles_from_disk();
    if let Some(idx) = profiles.iter().position(|p| p.name == profile.name) {
        profiles[idx] = profile.clone();
    } else {
        profiles.push(profile.clone());
    }

    save_profiles_to_disk(&profiles);

    // Update current state if it matches
    let mut is_current = false;
    {
        let mut state = state.lock().unwrap();
        if let Some(current) = &state.current_profile {
            if current.name == profile.name {
                state.current_profile = Some(profile.clone());
                is_current = true;
            }
        }
    }

    if is_current {
        // Restart Proxy to apply new settings
        proxy::restart_proxy(app.clone(), state.inner().clone()).await;

        // Close window to force refresh
        if let Some(w) = app.get_webview_window("target") {
            let _ = w.close();
        }
    }

    Ok(())
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
