use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use serde_json::Value;
use tokio::sync::oneshot;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub protocol: String, // "http", "socks5"
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub user_agent: String,
    pub seed: u64,
    pub proxy: Option<ProxyConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProxyStatus {
    Disconnected,
    Connected(String),
}

pub struct AppState {
    pub is_running: bool,
    pub current_profile: Option<Profile>,
    pub proxy_status: ProxyStatus,
    pub script_logs: Vec<String>,
    pub pending_callbacks: HashMap<String, Sender<Value>>,
    pub proxy_port: u16,
    pub ca_cert: Option<String>,
    pub proxy_shutdown_tx: Option<oneshot::Sender<()>>,
    pub last_request: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            is_running: false,
            current_profile: None,
            proxy_status: ProxyStatus::Disconnected,
            script_logs: Vec::new(),
            pending_callbacks: HashMap::new(),
            proxy_port: 0,
            ca_cert: None,
            proxy_shutdown_tx: None,
            last_request: None,
        }
    }
}
