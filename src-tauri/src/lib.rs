mod state;
mod scripting;
mod commands;
mod proxy;
mod fingerprint;
mod headless;
mod recorder;
mod scheduler;

use state::AppState;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long)]
    headless: bool,

    #[arg(long)]
    script: Option<String>,

    #[arg(long)]
    output: Option<String>,

    #[arg(long)]
    profile: Option<String>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Parse CLI args
    // We use try_parse so that in dev mode (without args) it doesn't fail if tauri adds something,
    // though usually tauri doesn't.
    let cli = Cli::parse();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .setup(move |app| {
            app.manage(Arc::new(Mutex::new(AppState::default())));

            if !cli.headless {
                let _ = tauri::WebviewWindowBuilder::new(
                    app,
                    "main",
                    tauri::WebviewUrl::App("index.html".into())
                )
                .title("stealth-tauri")
                .inner_size(1024.0, 768.0)
                .build()?;
            } else {
                 // Headless mode
                 if let Some(script_path) = &cli.script {
                     println!("Running script in headless mode: {}", script_path);
                     headless::run_headless_script(app.handle().clone(), script_path.clone(), cli.output.clone());
                 }
            }

            // Init Scheduler
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                scheduler::init(handle).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::execute_script,
            commands::script_callback,
            commands::get_profiles,
            commands::set_profile,
            commands::save_profile_config,
            commands::list_scripts,
            commands::save_script,
            commands::read_script,
            recorder::start_recording,
            recorder::stop_recording,
            recorder::recorder_event,
            scheduler::list_jobs,
            scheduler::save_job,
            scheduler::delete_job
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
