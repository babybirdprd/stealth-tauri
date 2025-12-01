use rhai::{Engine, Scope};
use tauri::{AppHandle, Manager, WebviewWindow, Emitter};
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc::channel;
use uuid::Uuid;
use crate::state::AppState;

#[derive(Clone)]
pub struct BrowserApi {
    window: WebviewWindow,
    state: Arc<Mutex<AppState>>,
}

impl BrowserApi {
    pub fn new(window: WebviewWindow, state: Arc<Mutex<AppState>>) -> Self {
        Self { window, state }
    }

    pub fn navigate(&mut self, url: &str) {
        let js = format!("window.location.href = '{}';", url);
        let _ = self.window.eval(&js);
    }

    pub fn click(&mut self, selector: &str) {
        let js = format!("document.querySelector('{}')?.click();", selector);
        let _ = self.window.eval(&js);
    }

    pub fn wait_for_selector(&mut self, selector: &str) {
        let (tx, rx) = channel();
        let id = Uuid::new_v4().to_string();

        {
            let mut state = self.state.lock().unwrap();
            state.pending_callbacks.insert(id.clone(), tx);
        }

        let js = format!(r#"
            (function() {{
                const sel = "{}";
                const id = "{}";
                const check = () => {{
                    if (document.querySelector(sel)) {{
                        window.__TAURI__.core.invoke('script_callback', {{ id: id, data: true }});
                        return true;
                    }}
                    return false;
                }};
                if (!check()) {{
                    const observer = new MutationObserver(() => {{
                        if (check()) observer.disconnect();
                    }});
                    observer.observe(document.body, {{ childList: true, subtree: true }});
                }}
            }})()
        "#, selector, id);

        let _ = self.window.eval(&js);

        // Block until result
        let _ = rx.recv();
    }

    pub fn extract_text(&mut self, selector: &str) -> String {
        let (tx, rx) = channel();
        let id = Uuid::new_v4().to_string();

        {
            let mut state = self.state.lock().unwrap();
            state.pending_callbacks.insert(id.clone(), tx);
        }

        let js = format!(r#"
            (function() {{
                const el = document.querySelector("{}");
                const text = el ? el.innerText : "";
                window.__TAURI__.core.invoke('script_callback', {{ id: "{}", data: text }});
            }})()
        "#, selector, id);

        let _ = self.window.eval(&js);

        match rx.recv() {
            Ok(val) => val.as_str().unwrap_or("").to_string(),
            Err(_) => "".to_string(),
        }
    }
}

pub fn run_script(script: String, app_handle: AppHandle, state: Arc<Mutex<AppState>>) {
    // Clone for the thread
    let app_handle = app_handle.clone();

    thread::spawn(move || {
        let window = match app_handle.get_webview_window("target") {
            Some(w) => w,
            None => {
                let _ = app_handle.emit("log_output", "Error: Target window not found");
                return;
            }
        };

        let mut engine = Engine::new();

        let browser_api = BrowserApi::new(window.clone(), state.clone());

        // Register the type and functions
        engine.register_type_with_name::<BrowserApi>("BrowserApi")
            .register_fn("navigate", |api: &mut BrowserApi, url: &str| api.navigate(url))
            .register_fn("click", |api: &mut BrowserApi, selector: &str| api.click(selector))
            .register_fn("wait_for_selector", |api: &mut BrowserApi, selector: &str| api.wait_for_selector(selector))
            .register_fn("extract_text", |api: &mut BrowserApi, selector: &str| api.extract_text(selector));

        let mut scope = Scope::new();
        scope.push("browser", browser_api);

        match engine.run_with_scope(&mut scope, &script) {
            Ok(_) => {
                 let _ = app_handle.emit("log_output", "Script finished successfully");
            },
            Err(e) => {
                 let _ = app_handle.emit("log_output", format!("Script error: {}", e));
            },
        }
    });
}
