use tauri::{AppHandle, Manager};
use tokio::sync::oneshot;
use crate::state::{ProxyConfig, AppState};
use std::sync::{Arc, Mutex};
use std::fs;
use std::path::PathBuf;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use hudsucker::{
    certificate_authority::RcgenAuthority,
    Proxy,
    HttpContext, HttpHandler, RequestOrResponse,
};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use http::{Request, Response};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use http_body_util::{BodyExt, Full};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use std::net::SocketAddr;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use std::net::TcpListener;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tauri::Emitter;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use rcgen::{KeyPair, CertificateParams, DnType, IsCa, BasicConstraints, SanType, Issuer};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use rustls::crypto::ring;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[derive(Clone)]
struct LogHandler {
    app: AppHandle,
    state: Arc<Mutex<AppState>>,
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
impl HttpHandler for LogHandler {
    async fn handle_request(&mut self, _ctx: &HttpContext, req: Request<hudsucker::Body>) -> RequestOrResponse {
        // Intercept Health Check
        // Check URI host
        if let Some(host) = req.uri().host() {
            if host == "phantom.internal" && req.uri().path() == "/health" {
                 let json = r#"{"status": "ok", "provider": "phantom-proxy"}"#;
                 let response = Response::builder()
                    .status(200)
                    .header("Content-Type", "application/json")
                    .body(hudsucker::Body::from(json))
                    .unwrap();
                 return RequestOrResponse::Response(response);
            }
        }

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
    shutdown_rx: oneshot::Receiver<()>,
    state: Arc<Mutex<AppState>>,
    ready_tx: oneshot::Sender<bool>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = app;
        let _ = port;
        let _ = shutdown_rx;
        let _ = state;
        let _ = ready_tx;
        eprintln!("Proxy is not supported on mobile.");
        Ok(())
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        // 1. Persistence Setup
        let app_data_dir = app.path().app_data_dir().unwrap_or(PathBuf::from(".")).join("certs");
        if !app_data_dir.exists() {
            let _ = fs::create_dir_all(&app_data_dir);
        }
        let cert_path = app_data_dir.join("ca.pem");
        let key_path = app_data_dir.join("key.pem");

        // 2. Load or Generate Certs
        let (cert_pem, key_pem) = if cert_path.exists() && key_path.exists() {
            (
                fs::read_to_string(&cert_path)?,
                fs::read_to_string(&key_path)?
            )
        } else {
            let key_pair = KeyPair::generate()?;
            let mut params = CertificateParams::default();
            params.distinguished_name.push(DnType::CommonName, "Phantom Browser Internal");
            params.is_ca = IsCa::Ca(BasicConstraints::Constrained(0));
            // Critical for Webview2 localhost access
            params.subject_alt_names = vec![
                SanType::DnsName("localhost".try_into().unwrap()),
                SanType::IpAddress(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))),
            ];

            let cert = params.self_signed(&key_pair)?;
            let c = cert.pem();
            let k = key_pair.serialize_pem();

            // Save to disk
            let _ = fs::write(&cert_path, &c);
            let _ = fs::write(&key_path, &k);
            (c, k)
        };

        // Store for potential UI use
        if let Ok(mut s) = state.lock() {
            s.ca_cert = Some(cert_pem.clone());
        }

        // 3. Build Authority
        let key_pair = KeyPair::from_pem(&key_pem)?;
        let issuer = Issuer::from_ca_cert_pem(&cert_pem, key_pair)?;
        let ca = RcgenAuthority::new(issuer, 3650, ring::default_provider());

        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let proxy = Proxy::builder()
            .with_addr(addr)
            .with_ca(ca)
            .with_rustls_connector(ring::default_provider())
            .with_http_handler(LogHandler { app, state })
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.await;
            })
            .build()?;

        // Signal ready
        let _ = ready_tx.send(true);

        if let Err(e) = proxy.start().await {
            eprintln!("Proxy fatal error: {}", e);
        }

        Ok(())
    }
}

pub async fn restart_proxy(app: AppHandle, state: Arc<Mutex<AppState>>) -> bool {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = app;
        let _ = state;
        eprintln!("Proxy restart skipped on mobile");
        return true;
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        // 1. Cleanup old proxy
        let mut old_tx = None;
        {
            if let Ok(mut s) = state.lock() {
                old_tx = s.proxy_shutdown_tx.take();
                s.proxy_port = 0; // Reset port so we know it's down
            }
        }
        if let Some(tx) = old_tx {
            let _ = tx.send(());
            // Give it a moment to release the port
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }

        // 2. Find new port
        let port = match TcpListener::bind("127.0.0.1:0") {
            Ok(l) => l.local_addr().unwrap().port(),
            Err(_) => return false
        };

        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let (ready_tx, ready_rx) = oneshot::channel();

        let app_clone = app.clone();
        let state_clone = state.clone();

        tokio::spawn(async move {
            let _ = start_proxy(app_clone, port, shutdown_rx, state_clone, ready_tx).await;
        });

        // 3. Wait for Ready Signal
        if let Ok(Ok(true)) = tokio::time::timeout(std::time::Duration::from_secs(5), ready_rx).await {
             if let Ok(mut s) = state.lock() {
                s.proxy_port = port;
                s.proxy_shutdown_tx = Some(shutdown_tx);
            }
            return true;
        }

        return false;
    }
}
