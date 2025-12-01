mod state;
mod scripting;
mod commands;
mod proxy;
mod fingerprint;
mod headless;
mod recorder;
mod scheduler;
mod assets;
mod server;

use state::AppState;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use clap::Parser;
use uuid::Uuid;

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

    #[arg(long)]
    api_port: Option<u16>,

    #[arg(long, env = "PHANTOM_API_TOKEN")]
    api_token: Option<String>,
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

            // Extract example scripts
            assets::extract_examples(app.handle());

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

            // Init API Server
            if let Some(port) = cli.api_port {
                let token = cli.api_token.unwrap_or_else(|| Uuid::new_v4().to_string());
                let handle = app.handle().clone();
                let state = app.state::<Arc<Mutex<AppState>>>().inner().clone();
                tauri::async_runtime::spawn(async move {
                    server::start_server(port, token, handle, state).await;
                });
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
