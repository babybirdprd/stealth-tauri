use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub user_agent: String,
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
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            is_running: false,
            current_profile: None,
            proxy_status: ProxyStatus::Disconnected,
            script_logs: Vec::new(),
            pending_callbacks: HashMap::new(),
        }
    }
}
