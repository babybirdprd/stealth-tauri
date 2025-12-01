use hudsucker::{
    certificate_authority::RcgenAuthority,
    Proxy,
    HttpContext, HttpHandler, RequestOrResponse,
};
use http::{Request, Response};
use http_body_util::{BodyExt, Full};
use std::net::SocketAddr;
use std::net::TcpListener;
use tauri::{AppHandle, Emitter};
use rcgen::{KeyPair, CertificateParams, Issuer};
use tokio::sync::oneshot;
use crate::state::{ProxyConfig, AppState};
use std::sync::{Arc, Mutex};
use rustls::crypto::ring;

#[derive(Clone)]
struct LogHandler {
    app: AppHandle,
    state: Arc<Mutex<AppState>>,
}

impl HttpHandler for LogHandler {
    async fn handle_request(&mut self, _ctx: &HttpContext, req: Request<hudsucker::Body>) -> RequestOrResponse {
        let url = req.uri().to_string();
        let method = req.method().to_string();

        let _ = self.app.emit("proxy://log", format!("REQ: {} {}", method, url));

        let (mut parts, body) = req.into_parts();

        parts.headers.remove("X-Forwarded-For");
        parts.headers.remove("X-Real-IP");
        parts.headers.remove("Sec-CH-UA");

        match body.collect().await {
            Ok(collected) => {
                let bytes = collected.to_bytes();
                let body_str = String::from_utf8_lossy(&bytes).to_string();

                {
                    if let Ok(mut s) = self.state.lock() {
                        s.last_request = Some(body_str);
                    }
                }

                let req = Request::from_parts(parts, hudsucker::Body::from(Full::new(bytes)));
                RequestOrResponse::Request(req)
            },
            Err(e) => {
                let _ = self.app.emit("proxy://log", format!("Error reading body: {}", e));
                let req = Request::from_parts(parts, hudsucker::Body::empty());
                RequestOrResponse::Request(req)
            }
        }
    }

    async fn handle_response(&mut self, _ctx: &HttpContext, res: Response<hudsucker::Body>) -> Response<hudsucker::Body> {
        let status = res.status();
        let _ = self.app.emit("proxy://log", format!("RES: {}", status));
        res
    }
}

pub async fn start_proxy(
    app: AppHandle,
    port: u16,
    _upstream: Option<ProxyConfig>, // Placeholder
    shutdown_rx: oneshot::Receiver<()>,
    state: Arc<Mutex<AppState>>
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let key_pair = KeyPair::generate()?;

    let mut params = CertificateParams::default();
    params.distinguished_name.push(rcgen::DnType::CommonName, "Phantom Browser CA");
    params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Constrained(0));

    let issuer = Issuer::new(params, key_pair);
    let provider = ring::default_provider();
    let ca = RcgenAuthority::new(issuer, 3650, provider);

    let proxy = Proxy::builder()
        .with_addr(SocketAddr::from(([127, 0, 0, 1], port)))
        .with_ca(ca)
        .with_rustls_connector(ring::default_provider())
        .with_http_handler(LogHandler { app, state })
        .with_graceful_shutdown(async move {
            let _ = shutdown_rx.await;
        })
        .build()?;

    if let Err(e) = proxy.start().await {
        eprintln!("Proxy failed: {}", e);
    }

    Ok(())
}

pub async fn restart_proxy(app: AppHandle, state: Arc<Mutex<AppState>>) {
    let mut old_tx = None;
    {
        if let Ok(mut s) = state.lock() {
            old_tx = s.proxy_shutdown_tx.take();
        }
    }
    if let Some(tx) = old_tx {
        let _ = tx.send(());
    }

    let port = {
        match TcpListener::bind("127.0.0.1:0") {
            Ok(l) => l.local_addr().unwrap().port(),
            Err(_) => 0
        }
    };

    if port == 0 {
        eprintln!("Failed to find free port for proxy");
        return;
    }

    let proxy_config = {
        let s = state.lock().unwrap();
        s.current_profile.as_ref().and_then(|p| p.proxy.clone())
    };

    let (tx, rx) = tokio::sync::oneshot::channel();
    let app_handle = app.clone();
    let state_clone = state.clone();

    tokio::spawn(async move {
        if let Err(e) = start_proxy(app_handle, port, proxy_config, rx, state_clone).await {
            eprintln!("Proxy error: {}", e);
        }
    });

    {
        if let Ok(mut s) = state.lock() {
            s.proxy_port = port;
            s.proxy_shutdown_tx = Some(tx);
        }
    }
}
