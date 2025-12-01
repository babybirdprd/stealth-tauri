use tauri::{AppHandle, Manager, State, Emitter};
use std::sync::{Arc, Mutex};
use crate::state::AppState;
use crate::commands::ensure_target_window;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RecorderEvent {
    pub event_type: String,
    pub selector: String,
    pub value: Option<String>,
}

#[tauri::command]
pub async fn start_recording(app: AppHandle, state: State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    {
        let mut s = state.lock().unwrap();
        s.is_recording = true;
    }

    // 1. Ensure target window exists
    let label = "target-studio";
    let window = ensure_target_window(&app, label).ok_or("Could not find target window")?;

    // 2. Inject recorder.js
    let js = include_str!("recorder.js");
    window.eval(js).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn stop_recording(state: State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    let mut s = state.lock().unwrap();
    s.is_recording = false;
    Ok(())
}

#[tauri::command]
pub async fn recorder_event(event: RecorderEvent, state: State<'_, Arc<Mutex<AppState>>>, app: AppHandle) -> Result<(), String> {
    {
        let s = state.lock().unwrap();
        if !s.is_recording {
            return Ok(());
        }
    }

    let mut script_line = String::new();

    match event.event_type.as_str() {
        "click" => {
            script_line = format!(
                "browser.wait_for_selector(\"{}\");\nbrowser.click(\"{}\");\n",
                event.selector, event.selector
            );
        },
        "type" => {
            if let Some(val) = event.value {
                 script_line = format!(
                    "browser.wait_for_selector(\"{}\");\nbrowser.type(\"{}\", \"{}\");\n",
                    event.selector, event.selector, val
                );
            }
        },
        _ => {}
    }

    if !script_line.is_empty() {
        let mut state = state.lock().unwrap();
        state.recorded_script.push_str(&script_line);
        let full_script = state.recorded_script.clone();

        // Emit update
        let _ = app.emit("script_update", full_script);
    }

    Ok(())
}
