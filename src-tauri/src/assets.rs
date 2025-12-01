use include_dir::{include_dir, Dir};
use std::fs;
use std::path::Path;
use tauri::AppHandle;

static EXAMPLES_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/examples");

pub fn extract_examples(_app: &AppHandle) {
    // In a real production app, we should use app.path().app_data_dir().
    // For now, we match the existing behavior in commands.rs which uses "scripts" in the CWD.
    let scripts_path = Path::new("scripts");
    let examples_path = scripts_path.join("examples");

    if !scripts_path.exists() {
        if let Err(e) = fs::create_dir_all(scripts_path) {
            eprintln!("Failed to create scripts dir: {}", e);
            return;
        }
    }

    if !examples_path.exists() {
        if let Err(e) = fs::create_dir(&examples_path) {
             eprintln!("Failed to create examples dir: {}", e);
             return;
        }
    }

    for file in EXAMPLES_DIR.files() {
        let path = examples_path.join(file.path());
        if !path.exists() {
            if let Err(e) = fs::write(&path, file.contents()) {
                eprintln!("Failed to write example {}: {}", path.display(), e);
            } else {
                println!("Extracted example: {}", path.display());
            }
        }
    }
}
