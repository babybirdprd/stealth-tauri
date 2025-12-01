use axum::{
    extract::{State, Request},
    routing::{get, post},
    Router, Json, http::StatusCode,
    response::IntoResponse,
    middleware::Next,
};
use tauri::{AppHandle, Manager};
use std::sync::{Arc, Mutex};
use crate::state::AppState;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
struct ServerState {
    app: AppHandle,
    state: Arc<Mutex<AppState>>,
    token: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    uptime: u64,
    active_jobs: usize,
}

#[derive(Serialize)]
struct LogsResponse {
    logs: Vec<String>,
}

#[derive(Deserialize)]
struct JobRequest {
    script: String,
    profile: Option<String>,
}

async fn health(State(_data): State<ServerState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
        uptime: 0,
        active_jobs: 0,
    })
}

async fn get_logs(State(data): State<ServerState>) -> Json<LogsResponse> {
    let logs = {
        let s = data.state.lock().unwrap();
        s.script_logs.clone()
    };
    Json(LogsResponse { logs })
}

async fn submit_job(
    State(data): State<ServerState>,
    Json(payload): Json<JobRequest>
) -> impl IntoResponse {
    if let Some(profile_name) = payload.profile {
        let profiles = crate::commands::get_profiles();
        if let Some(p) = profiles.into_iter().find(|p| p.name == profile_name) {
             {
                let mut s = data.state.lock().unwrap();
                s.current_profile = Some(p.clone());
             }
             crate::proxy::restart_proxy(data.app.clone(), data.state.clone()).await;
        }
    }

    let needs_start = {
        let s = data.state.lock().unwrap();
        s.proxy_port == 0
    };
    if needs_start {
        crate::proxy::restart_proxy(data.app.clone(), data.state.clone()).await;
    }

    let label = "headless-job";

    // We use ensure_target_window from commands.rs
    // This assumes commands.rs is compiled and available (it is)

    if let Some(_w) = crate::commands::ensure_target_window(&data.app, label) {
         crate::scripting::run_script(
             payload.script,
             data.app.clone(),
             data.state.clone(),
             label.to_string()
         );
         (StatusCode::OK, "Job started")
    } else {
         (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create window")
    }
}

async fn stop_jobs(State(data): State<ServerState>) -> impl IntoResponse {
    if let Some(w) = data.app.get_webview_window("target-studio") {
        let _ = w.close();
    }
    if let Some(w) = data.app.get_webview_window("headless-job") {
        let _ = w.close();
    }
    (StatusCode::OK, "Stopped")
}

async fn auth_middleware(
    State(state): State<ServerState>,
    req: Request,
    next: Next,
) -> impl IntoResponse {
    let auth_header = req.headers().get("Authorization");
    let expected = format!("Bearer {}", state.token);

    match auth_header {
        Some(header) if header.to_str().unwrap_or("") == expected => {
            next.run(req).await
        }
        _ => StatusCode::UNAUTHORIZED.into_response(),
    }
}

pub async fn start_server(port: u16, token: String, app: AppHandle, state: Arc<Mutex<AppState>>) {
    let server_state = ServerState {
        app,
        state,
        token: token.clone(),
    };

    let app_router = Router::new()
        .route("/health", get(health))
        .route("/logs", get(get_logs))
        .route("/jobs", post(submit_job))
        .route("/stop", post(stop_jobs))
        .layer(axum::middleware::from_fn_with_state(server_state.clone(), auth_middleware))
        .with_state(server_state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    println!("API Server running on {}", addr);
    println!("API Token: {}", token);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app_router).await.unwrap();
}
