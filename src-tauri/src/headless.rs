use tauri::{AppHandle, Manager, WebviewWindowBuilder, WebviewUrl, Url};
use std::sync::{Arc, Mutex};
use crate::state::AppState;
use crate::scripting;
use std::fs;

pub fn run_headless_script(app: AppHandle, script_path: String, output_path: Option<String>) {
    // 1. Read script
    let content = match fs::read_to_string(&script_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading script '{}': {}", script_path, e);
            app.exit(1);
            return;
        }
    };

    // 2. Create hidden window
    let label = format!("headless-{}", uuid::Uuid::new_v4());
    let url = Url::parse("about:blank").unwrap();
    let builder = WebviewWindowBuilder::new(
        &app,
        &label,
        WebviewUrl::External(url)
    )
    .inner_size(1920.0, 1080.0)
    .visible(false); // Hidden!

    let window = match builder.build() {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Error creating headless window: {}", e);
            app.exit(1);
            return;
        }
    };

    // 3. Execute
    let state_handle = app.state::<Arc<Mutex<AppState>>>();
    let state = state_handle.inner().clone();

    let app_handle = app.clone();
    std::thread::spawn(move || {
        // We pass None for app_handle so scripting::execute uses println! for logging
        let result = scripting::execute(content, window, state, None);

        match result {
            Ok(val) => {
                // Serialize result
                // Rhai Dynamic -> Serde Json
                // We need to ensure Dynamic implements Serialize or convert it.
                // rhai "serde" feature is enabled in Cargo.toml?
                // Let's check. Yes `features = ["sync"]`. "serde" is not explicitly enabled for rhai in Cargo.toml.
                // Wait, I saw `rhai = { version = "1.23.6", features = ["sync"] }` in the read_file output.
                // I need to add "serde" feature to rhai.

                let output = serde_json::to_string_pretty(&val).unwrap_or_else(|_| format!("{:?}", val));

                if let Some(path) = output_path {
                    if let Err(e) = fs::write(&path, output) {
                         eprintln!("Error writing output to '{}': {}", path, e);
                         app_handle.exit(1);
                    } else {
                         println!("Output written to {}", path);
                         app_handle.exit(0);
                    }
                } else {
                    println!("{}", output);
                    app_handle.exit(0);
                }
            },
            Err(e) => {
                eprintln!("Script execution error: {}", e);
                app_handle.exit(1);
            }
        }
    });
}
