use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, WebviewWindowBuilder, WebviewUrl, Url, State};
use crate::state::AppState;
use crate::scripting;
use std::fs;
use std::path::{Path, PathBuf};
use chrono::Utc;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PhantomJob {
    pub id: String,
    pub script_path: String,
    pub cron: String,
    pub profile: Option<String>,
    pub last_run: Option<String>,
    pub status: String, // "active", "paused"
}

fn get_jobs_path() -> PathBuf {
    Path::new("jobs.json").to_path_buf()
}

fn load_jobs_from_disk() -> Vec<PhantomJob> {
    let path = get_jobs_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(jobs) = serde_json::from_str::<Vec<PhantomJob>>(&content) {
                return jobs;
            }
        }
    }
    Vec::new()
}

fn save_jobs_to_disk(jobs: &[PhantomJob]) {
    let path = get_jobs_path();
    let json = serde_json::to_string_pretty(jobs).unwrap_or_default();
    let _ = fs::write(path, json);
}

pub async fn init(app: AppHandle) {
    let sched = JobScheduler::new().await.unwrap();
    let jobs = load_jobs_from_disk();

    for job in jobs {
        if job.status == "active" {
             schedule_job(&sched, job, app.clone()).await;
        }
    }

    sched.start().await.unwrap();

    let state = app.state::<Arc<Mutex<AppState>>>();
    let mut state = state.lock().unwrap();
    state.scheduler = Some(Arc::new(sched));
}

async fn schedule_job(sched: &JobScheduler, job: PhantomJob, app: AppHandle) {
    let job_id = job.id.clone();
    let cron = job.cron.clone();
    let app_clone = app.clone();
    let job_clone = job.clone();

    let job_runner = Job::new_async(cron.as_str(), move |uuid, mut _l| {
        let app = app_clone.clone();
        let job = job_clone.clone();
        Box::pin(async move {
            run_job(app, job).await;
        })
    });

    if let Ok(j) = job_runner {
        sched.add(j).await.ok();
    } else {
        eprintln!("Failed to create job for cron: {}", cron);
    }
}

async fn run_job(app: AppHandle, job: PhantomJob) {
    println!("Executing Job: {}", job.id);

    let script_content = match fs::read_to_string(Path::new("scripts").join(&job.script_path)) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Job {} failed to read script: {}", job.id, e);
            return;
        }
    };

    let run_id = uuid::Uuid::new_v4();
    let label = format!("job-{}-{}", job.id, run_id);

    let url = Url::parse("about:blank").unwrap();
    let builder = WebviewWindowBuilder::new(
        &app,
        &label,
        WebviewUrl::External(url)
    )
    .inner_size(1920.0, 1080.0)
    .visible(false);

    let window = match builder.build() {
        Ok(w) => w,
        Err(e) => {
             eprintln!("Job {} failed to create window: {}", job.id, e);
             return;
        }
    };

    let state_handle = app.state::<Arc<Mutex<AppState>>>();
    let state = state_handle.inner().clone();

    let result = tokio::task::spawn_blocking(move || {
        scripting::execute(script_content, window, state, None)
    }).await;

    match result {
        Ok(exec_res) => {
             match exec_res {
                 Ok(val) => println!("Job {} finished: {:?}", job.id, val),
                 Err(e) => println!("Job {} error: {}", job.id, e),
             }
        },
        Err(e) => println!("Job {} join error: {}", job.id, e),
    }

    if let Some(w) = app.get_webview_window(&label) {
        let _ = w.close();
    }
}

#[tauri::command]
pub fn list_jobs() -> Vec<PhantomJob> {
    load_jobs_from_disk()
}

#[tauri::command]
pub async fn save_job(job: PhantomJob, app: AppHandle, state: State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    let mut jobs = load_jobs_from_disk();
    if let Some(idx) = jobs.iter().position(|j| j.id == job.id) {
        jobs[idx] = job.clone();
    } else {
        jobs.push(job.clone());
    }
    save_jobs_to_disk(&jobs);
    Ok(())
}

#[tauri::command]
pub fn delete_job(job_id: String) -> Result<(), String> {
     let mut jobs = load_jobs_from_disk();
     jobs.retain(|j| j.id != job_id);
     save_jobs_to_disk(&jobs);
     Ok(())
}
