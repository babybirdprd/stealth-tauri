use rhai::{Engine, Scope};
use tauri::{AppHandle, Manager, WebviewWindow, Emitter};
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc::channel;
use uuid::Uuid;
use crate::state::{AppState, ProxyConfig};
use url::Url;

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

    pub fn type_text(&mut self, selector: &str, text: &str) {
         let js = format!(r#"
            (function() {{
                let el = document.querySelector('{}');
                if (el) {{
                    el.value = '{}';
                    el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                }}
            }})()
         "#, selector, text);
         let _ = self.window.eval(&js);
    }

    pub fn sleep(&mut self, ms: i64) {
        std::thread::sleep(std::time::Duration::from_millis(ms as u64));
    }

    pub fn wait(&mut self, ms: i64) {
        self.sleep(ms);
    }

    pub fn set_proxy(&mut self, proxy_url: &str) {
        if let Ok(url) = Url::parse(proxy_url) {
            let protocol = url.scheme().to_string();
            let host = url.host_str().unwrap_or("").to_string();
            let port = url.port().unwrap_or(80);
            let username = if url.username().is_empty() { None } else { Some(url.username().to_string()) };
            let password = url.password().map(|s| s.to_string());

            let config = ProxyConfig { protocol, host, port, username, password };

            {
                let mut state = self.state.lock().unwrap();
                if let Some(profile) = &mut state.current_profile {
                    profile.proxy = Some(config);
                }
            }

            // Restart Proxy
            let app = self.window.app_handle().clone();
            let state = self.state.clone();
            tauri::async_runtime::block_on(async {
                let _ = crate::proxy::restart_proxy(app, state).await;
            });
        }
    }

    pub fn get_last_request(&mut self) -> String {
        let state = self.state.lock().unwrap();
        state.last_request.clone().unwrap_or_default()
    }
}

pub fn run_script(script: String, app_handle: AppHandle, state: Arc<Mutex<AppState>>, window_label: String) {
    let app_handle = app_handle.clone();

    thread::spawn(move || {
        // 1. Initialization Delay (Increased to ensure Webview2 is ready)
        std::thread::sleep(std::time::Duration::from_millis(2000));

        let window = match app_handle.get_webview_window(&window_label) {
            Some(w) => w,
            None => {
                let _ = app_handle.emit("log_output", format!("Error: Target window '{}' not found", window_label));
                return;
            }
        };

        // 2. Visual Warm-up
        let _ = window.eval("window.location.href = 'data:text/html,<html><style>body{background:#222;color:#888;display:flex;justify-content:center;align-items:center;height:100vh;font-family:monospace;}</style><body>Initializing Secure Tunnel...</body></html>';");

        // 3. Wait for warm-up to render and proxy to settle
        std::thread::sleep(std::time::Duration::from_millis(800));

        // 4. Run User Script
        match execute(script, window, state, Some(app_handle.clone())) {
            Ok(_) => {
                 let _ = app_handle.emit("log_output", "Script finished successfully");
            },
            Err(e) => {
                 let _ = app_handle.emit("log_output", format!("Script error: {}", e));
            },
        }
    });
}

fn register_api(engine: &mut Engine) {
    engine.register_type_with_name::<BrowserApi>("BrowserApi")
        .register_fn("navigate", |api: &mut BrowserApi, url: &str| api.navigate(url))
        .register_fn("click", |api: &mut BrowserApi, selector: &str| api.click(selector))
        .register_fn("wait_for_selector", |api: &mut BrowserApi, selector: &str| api.wait_for_selector(selector))
        .register_fn("extract_text", |api: &mut BrowserApi, selector: &str| api.extract_text(selector))
        .register_fn("type", |api: &mut BrowserApi, selector: &str, text: &str| api.type_text(selector, text))
        .register_fn("sleep", |api: &mut BrowserApi, ms: i64| api.sleep(ms))
        .register_fn("wait", |api: &mut BrowserApi, ms: i64| api.wait(ms))
        .register_fn("set_proxy", |api: &mut BrowserApi, url: &str| api.set_proxy(url))
        .register_fn("get_last_request", |api: &mut BrowserApi| api.get_last_request());
}

pub fn execute(
    script: String,
    window: WebviewWindow,
    state: Arc<Mutex<AppState>>,
    app_handle: Option<AppHandle>
) -> Result<rhai::Dynamic, Box<rhai::EvalAltResult>> {
    let mut engine = Engine::new();
    let state_clone = state.clone();
    let browser_api = BrowserApi::new(window, state);

    register_api(&mut engine);

    // Handle print/logging
    if let Some(app) = app_handle {
        engine.on_print(move |s| {
            let _ = app.emit("log_output", s);
            // Also save to AppState
            if let Ok(mut st) = state_clone.lock() {
                st.script_logs.push(s.to_string());
                // Keep log size manageable?
                if st.script_logs.len() > 1000 {
                    st.script_logs.remove(0);
                }
            }
        });
    } else {
        engine.on_print(|s| {
            println!("{}", s);
        });
    }

    let mut scope = Scope::new();
    scope.push("browser", browser_api);

    engine.eval_with_scope::<rhai::Dynamic>(&mut scope, &script)
}
