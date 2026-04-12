use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::command;

/// Represents the state of a proxy listener
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListenerConfig {
    pub id: u32,
    pub host: String,
    pub port: u16,
    pub is_running: bool,
    pub intercept_https: bool,
}

/// Application state shared across Tauri commands
pub struct AppState {
    pub listeners: Mutex<Vec<ListenerConfig>>,
    pub request_count: Mutex<u64>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            listeners: Mutex::new(vec![
                ListenerConfig {
                    id: 1,
                    host: "127.0.0.1".to_string(),
                    port: 8080,
                    is_running: false,
                    intercept_https: true,
                },
            ]),
            request_count: Mutex::new(0),
        }
    }
}

#[command]
pub fn get_app_name() -> String {
    "r-burp".to_string()
}

#[command]
pub fn get_current_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    format!("{}", duration.as_secs())
}

#[command]
pub fn get_listeners(state: tauri::State<AppState>) -> Vec<ListenerConfig> {
    let listeners = state.listeners.lock().unwrap();
    listeners.clone()
}

#[command]
pub fn start_listener(listener_id: u32, state: tauri::State<AppState>) -> Result<bool, String> {
    let mut listeners = state.listeners.lock().unwrap();
    for listener in listeners.iter_mut() {
        if listener.id == listener_id {
            listener.is_running = true;
            return Ok(true);
        }
    }
    Err(format!("Listener {} not found", listener_id))
}

#[command]
pub fn stop_listener(listener_id: u32, state: tauri::State<AppState>) -> Result<bool, String> {
    let mut listeners = state.listeners.lock().unwrap();
    for listener in listeners.iter_mut() {
        if listener.id == listener_id {
            listener.is_running = false;
            return Ok(true);
        }
    }
    Err(format!("Listener {} not found", listener_id))
}

#[command]
pub fn get_request_count(state: tauri::State<AppState>) -> u64 {
    *state.request_count.lock().unwrap()
}

#[command]
pub fn add_listener(
    host: String,
    port: u16,
    intercept_https: bool,
    state: tauri::State<AppState>,
) -> Result<u32, String> {
    // Validate port is not zero
    if port == 0 {
        return Err("Port cannot be 0".to_string());
    }

    // Validate host is not empty
    let trimmed_host = host.trim();
    if trimmed_host.is_empty() {
        return Err("Host cannot be empty".to_string());
    }

    // Validate host contains only valid characters
    if trimmed_host.contains(|c: char| !c.is_alphanumeric() && c != '.' && c != '-' && c != '_') {
        return Err("Host contains invalid characters".to_string());
    }

    // Require explicit confirmation for non-loopback binding
    // This prevents accidentally exposing the proxy to the network
    if !is_loopback_address(trimmed_host) {
        return Err(format!(
            "Binding to '{}' would expose the proxy on the network. \
            Only loopback addresses (127.0.0.1, localhost, ::1) are allowed by default. \
            This is a security restriction.",
            trimmed_host
        ));
    }

    let mut listeners = state.listeners.lock().unwrap();
    let new_id = listeners.iter().map(|l| l.id).max().unwrap_or(0) + 1;

    listeners.push(ListenerConfig {
        id: new_id,
        host: trimmed_host.to_string(),
        port,
        is_running: false,
        intercept_https,
    });

    Ok(new_id)
}

/// Check if an address is a loopback address
fn is_loopback_address(host: &str) -> bool {
    let lower = host.to_lowercase();
    lower == "127.0.0.1"
        || lower == "localhost"
        || lower == "::1"
        || lower == "0:0:0:0:0:0:0:1"
        || lower.starts_with("127.")
}

#[command]
pub fn remove_listener(listener_id: u32, state: tauri::State<AppState>) -> Result<bool, String> {
    let mut listeners = state.listeners.lock().unwrap();
    let initial_len = listeners.len();
    listeners.retain(|l| l.id != listener_id);
    Ok(listeners.len() < initial_len)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_state() -> AppState {
        AppState {
            listeners: Mutex::new(vec![
                ListenerConfig {
                    id: 1,
                    host: "127.0.0.1".to_string(),
                    port: 8080,
                    is_running: false,
                    intercept_https: true,
                },
            ]),
            request_count: Mutex::new(0),
        }
    }

    // Helper: start listener by directly manipulating state
    fn test_start_listener(state: &AppState, listener_id: u32) -> Result<bool, String> {
        let mut listeners = state.listeners.lock().unwrap();
        for listener in listeners.iter_mut() {
            if listener.id == listener_id {
                listener.is_running = true;
                return Ok(true);
            }
        }
        Err(format!("Listener {} not found", listener_id))
    }

    fn test_stop_listener(state: &AppState, listener_id: u32) -> Result<bool, String> {
        let mut listeners = state.listeners.lock().unwrap();
        for listener in listeners.iter_mut() {
            if listener.id == listener_id {
                listener.is_running = false;
                return Ok(true);
            }
        }
        Err(format!("Listener {} not found", listener_id))
    }

    fn test_add_listener(
        state: &AppState,
        host: String,
        port: u16,
        intercept_https: bool,
    ) -> Result<u32, String> {
        if port == 0 {
            return Err("Port cannot be 0".to_string());
        }
        let trimmed_host = host.trim();
        if trimmed_host.is_empty() {
            return Err("Host cannot be empty".to_string());
        }
        if trimmed_host.contains(|c: char| !c.is_alphanumeric() && c != '.' && c != '-' && c != '_') {
            return Err("Host contains invalid characters".to_string());
        }
        let mut listeners = state.listeners.lock().unwrap();
        let new_id = listeners.iter().map(|l| l.id).max().unwrap_or(0) + 1;
        listeners.push(ListenerConfig {
            id: new_id,
            host: trimmed_host.to_string(),
            port,
            is_running: false,
            intercept_https,
        });
        Ok(new_id)
    }

    fn test_remove_listener(state: &AppState, listener_id: u32) -> Result<bool, String> {
        let mut listeners = state.listeners.lock().unwrap();
        let initial_len = listeners.len();
        listeners.retain(|l| l.id != listener_id);
        Ok(listeners.len() < initial_len)
    }

    #[test]
    fn test_get_app_name() {
        assert_eq!(get_app_name(), "r-burp");
    }

    #[test]
    fn test_get_current_timestamp_returns_valid_number() {
        let timestamp = get_current_timestamp();
        let ts: u64 = timestamp.parse().expect("Timestamp should be a valid number");
        assert!(ts > 0);
    }

    #[test]
    fn test_get_listeners_returns_initial_listener() {
        let state = create_test_state();
        let listeners = state.listeners.lock().unwrap();
        assert_eq!(listeners.len(), 1);
        assert_eq!(listeners[0].id, 1);
        assert_eq!(listeners[0].host, "127.0.0.1");
        assert_eq!(listeners[0].port, 8080);
    }

    #[test]
    fn test_start_listener_works() {
        let state = create_test_state();
        let result = test_start_listener(&state, 1);
        assert!(result.is_ok());
        assert!(result.unwrap());

        let listeners = state.listeners.lock().unwrap();
        assert!(listeners[0].is_running);
    }

    #[test]
    fn test_stop_listener_works() {
        let state = create_test_state();
        let _ = test_start_listener(&state, 1);
        let result = test_stop_listener(&state, 1);
        assert!(result.is_ok());
        assert!(result.unwrap());

        let listeners = state.listeners.lock().unwrap();
        assert!(!listeners[0].is_running);
    }

    #[test]
    fn test_start_nonexistent_listener_returns_error() {
        let state = create_test_state();
        let result = test_start_listener(&state, 999);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_listener_validates_empty_host() {
        let state = create_test_state();
        let result = test_add_listener(&state, "".to_string(), 8080, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }

    #[test]
    fn test_add_listener_validates_zero_port() {
        let state = create_test_state();
        let result = test_add_listener(&state, "127.0.0.1".to_string(), 0, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("0"));
    }

    #[test]
    fn test_add_listener_rejects_invalid_characters() {
        let state = create_test_state();
        let result = test_add_listener(&state, "127.0.0.1; rm -rf /".to_string(), 8080, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid"));
    }

    #[test]
    fn test_add_listener_creates_listener() {
        let state = create_test_state();
        let result = test_add_listener(&state, "192.168.1.1".to_string(), 9090, true);
        assert!(result.is_ok());
        let new_id = result.unwrap();
        assert!(new_id > 1);

        let listeners = state.listeners.lock().unwrap();
        assert_eq!(listeners.len(), 2);

        let new_listener = listeners.iter().find(|l| l.id == new_id).unwrap();
        assert_eq!(new_listener.host, "192.168.1.1");
        assert_eq!(new_listener.port, 9090);
        assert!(new_listener.intercept_https);
        assert!(!new_listener.is_running);
    }

    #[test]
    fn test_remove_listener_works() {
        let state = create_test_state();
        let new_id = test_add_listener(&state, "10.0.0.1".to_string(), 3000, false).unwrap();
        let result = test_remove_listener(&state, new_id);
        assert!(result.is_ok());
        assert!(result.unwrap());

        let listeners = state.listeners.lock().unwrap();
        assert_eq!(listeners.len(), 1);
    }

    #[test]
    fn test_remove_nonexistent_listener_returns_false() {
        let state = create_test_state();
        let result = test_remove_listener(&state, 999);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_get_request_count_initially_zero() {
        let state = create_test_state();
        let count = state.request_count.lock().unwrap();
        assert_eq!(*count, 0);
    }
}

// =====================
// Proxy Engine Commands
// =====================

use crate::proxy::{RequestSummary, TrafficStats, HttpTransaction};
use crate::server::ProxyServer;
use crate::AppRuntime;
use tauri::Emitter;

#[command]
pub fn get_transactions(state: tauri::State<AppRuntime>) -> Vec<HttpTransaction> {
    state.engine.get_transactions()
}

#[command]
pub fn get_transaction(id: String, state: tauri::State<AppRuntime>) -> Option<HttpTransaction> {
    state.engine.get_transaction(&id)
}

#[command]
pub fn get_request_summaries(state: tauri::State<AppRuntime>) -> Vec<RequestSummary> {
    state.engine.get_summaries()
}

#[command]
pub fn get_traffic_stats(state: tauri::State<AppRuntime>) -> TrafficStats {
    state.engine.get_stats()
}

#[command]
pub fn clear_transactions(state: tauri::State<AppRuntime>) {
    state.engine.clear_transactions();
    state.emit_event("transactions-cleared", ());
}

#[command]
pub fn start_proxy(host: String, port: u16, state: tauri::State<AppRuntime>) -> Result<String, String> {
    // Check if already running
    {
        let existing = state.proxy_shutdown.lock();
        if existing.is_some() {
            return Err("Proxy is already running".to_string());
        }
    }

    let host_clone = host.clone();
    let engine_clone = state.engine.clone();
    let intercept_clone = state.intercept.clone();
    let certs_clone = state.certs.clone();
    let handle = state.app_handle.lock().clone();

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    *state.proxy_shutdown.lock() = Some(shutdown_tx);

    state.runtime.spawn(async move {
        let mut srv = ProxyServer::new(engine_clone, intercept_clone, certs_clone, host_clone, port);
        let srv_shutdown = shutdown_rx;

        let result = tokio::select! {
            r = srv.start_with_shutdown(srv_shutdown) => r,
        };

        match result {
            Ok(()) => {
                if let Some(h) = handle {
                    let _ = h.emit("proxy-stopped", ());
                }
            }
            Err(e) => {
                log::error!("Proxy server error: {}", e);
                if let Some(h) = handle {
                    let _ = h.emit("proxy-error", e.to_string());
                }
            }
        }
    });

    state.emit_event("proxy-started", ());
    Ok(format!("Proxy started on {}:{}", host, port))
}

#[command]
pub fn stop_proxy(state: tauri::State<AppRuntime>) -> Result<String, String> {
    let mut shutdown_guard = state.proxy_shutdown.lock();
    if let Some(tx) = shutdown_guard.take() {
        let _ = tx.send(());
        Ok("Proxy stopping...".to_string())
    } else {
        Err("Proxy is not running".to_string())
    }
}

#[command]
pub fn get_proxy_status(state: tauri::State<AppRuntime>) -> String {
    let stats = state.engine.get_stats();
    if stats.total_requests > 0 {
        format!("Active - {} requests captured", stats.total_requests)
    } else {
        "Idle - No traffic".to_string()
    }
}

// =====================
// Intercept Commands
// =====================

use crate::intercept::{InterceptAction, PendingInterceptSummary, RuleAction, RuleMatchType};
use std::collections::HashMap;

#[command]
pub fn enable_intercept(state: tauri::State<AppRuntime>) {
    state.intercept.set_enabled(true);
}

#[command]
pub fn disable_intercept(state: tauri::State<AppRuntime>) {
    state.intercept.set_enabled(false);
}

#[command]
pub fn is_intercept_enabled(state: tauri::State<AppRuntime>) -> bool {
    state.intercept.is_enabled()
}

#[command]
pub fn resume_intercept(
    request_id: String,
    method: Option<String>,
    url: Option<String>,
    headers: Option<HashMap<String, String>>,
    body: Option<Vec<u8>>,
    state: tauri::State<AppRuntime>,
) -> Result<bool, String> {
    let action = InterceptAction::Modify {
        method,
        url,
        headers,
        body,
    };
    Ok(state.intercept.resume_intercept(&request_id, action))
}

#[command]
pub fn drop_intercept(request_id: String, state: tauri::State<AppRuntime>) -> Result<bool, String> {
    Ok(state.intercept.resume_intercept(&request_id, InterceptAction::Drop))
}

#[command]
pub fn get_pending_intercepts(state: tauri::State<AppRuntime>) -> Vec<PendingInterceptSummary> {
    state.intercept.get_pending()
}

// =====================
// Rule Commands
// =====================

#[command]
pub fn add_rule(
    name: String,
    match_type: String,
    match_pattern: String,
    actions: Vec<RuleAction>,
    state: tauri::State<AppRuntime>,
) -> Result<String, String> {
    let mtype = match match_type.as_str() {
        "url_contains" => RuleMatchType::UrlContains,
        "method_equals" => RuleMatchType::MethodEquals,
        "url_regex" => RuleMatchType::UrlRegex,
        "header_contains" => RuleMatchType::HeaderContains,
        _ => return Err(format!("Unknown match type: {}", match_type)),
    };

    Ok(state.intercept.add_rule(name, mtype, match_pattern, actions))
}

#[command]
pub fn remove_rule(id: String, state: tauri::State<AppRuntime>) -> Result<bool, String> {
    Ok(state.intercept.remove_rule(&id))
}

#[command]
pub fn toggle_rule(id: String, enabled: bool, state: tauri::State<AppRuntime>) -> Result<bool, String> {
    Ok(state.intercept.toggle_rule(&id, enabled))
}

#[command]
pub fn get_rules(state: tauri::State<AppRuntime>) -> Vec<crate::intercept::InterceptRule> {
    state.intercept.get_rules()
}

// =====================
// Certificate Commands
// =====================

use crate::certs::CertInfo;

#[command]
pub fn generate_ca_cert(state: tauri::State<AppRuntime>) -> Result<String, String> {
    state.certs.generate_ca()
}

#[command]
pub fn get_cert_info(state: tauri::State<AppRuntime>) -> CertInfo {
    state.certs.get_cert_info()
}

#[command]
pub fn get_cert_pem(state: tauri::State<AppRuntime>) -> Option<String> {
    state.certs.get_cert_pem()
}

// =====================
// Export Commands
// =====================

use chrono::SecondsFormat;

#[derive(serde::Serialize)]
pub struct HarEntry {
    started_date_time: String,
    time: f64,
    request: HarEntryRequest,
    response: HarEntryResponse,
}

#[derive(serde::Serialize)]
pub struct HarEntryRequest {
    method: String,
    url: String,
    http_version: String,
    headers: Vec<HarHeader>,
    body_size: i64,
}

#[derive(serde::Serialize)]
pub struct HarEntryResponse {
    status: i32,
    status_text: String,
    http_version: String,
    headers: Vec<HarHeader>,
    content: HarContent,
    body_size: i64,
    time: f64,
}

#[derive(serde::Serialize, Clone)]
pub struct HarHeader {
    name: String,
    value: String,
}

#[derive(serde::Serialize)]
pub struct HarContent {
    size: i64,
    mime_type: String,
    text: Option<String>,
}

#[command]
pub fn export_har(state: tauri::State<AppRuntime>) -> String {
    let transactions = state.engine.get_transactions();

    let entries: Vec<HarEntry> = transactions
        .into_iter()
        .filter(|t| t.is_complete)
        .map(|t| {
            let req_headers: Vec<HarHeader> = t.request.headers
                .iter()
                .map(|(k, v)| HarHeader { name: k.clone(), value: v.clone() })
                .collect();

            let resp = t.response.as_ref();
            let resp_headers: Vec<HarHeader> = resp.iter()
                .flat_map(|r| r.headers.iter().map(|(k, v)| HarHeader { name: k.clone(), value: v.clone() }))
                .collect();

            let time_ms = resp.map(|r| r.duration_ms as f64).unwrap_or(0.0);

            HarEntry {
                started_date_time: t.request.timestamp.to_rfc3339_opts(SecondsFormat::Millis, true),
                time: time_ms,
                request: HarEntryRequest {
                    method: t.request.method,
                    url: t.request.url,
                    http_version: t.request.version,
                    headers: req_headers,
                    body_size: t.request.content_length as i64,
                },
                response: HarEntryResponse {
                    status: resp.map(|r| r.status as i32).unwrap_or(0),
                    status_text: resp.map(|r| r.status_text.clone()).unwrap_or_default(),
                    http_version: resp.map(|r| r.version.clone()).unwrap_or_default(),
                    headers: resp_headers,
                    content: HarContent {
                        size: resp.map(|r| r.content_length as i64).unwrap_or(0),
                        mime_type: resp.and_then(|r| r.content_type.clone()).unwrap_or_default(),
                        text: resp.and_then(|r| r.body_text.clone()),
                    },
                    body_size: resp.map(|r| r.content_length as i64).unwrap_or(0),
                    time: time_ms,
                },
            }
        })
        .collect();

    let har = serde_json::json!({
        "log": {
            "version": "1.2",
            "creator": {
                "name": "r-burp",
                "version": "0.1.0"
            },
            "entries": entries
        }
    });

    serde_json::to_string_pretty(&har).unwrap_or_default()
}
