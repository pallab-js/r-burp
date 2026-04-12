mod commands;
mod proxy;
mod server;
mod intercept;
mod certs;

use commands::AppState;
use proxy::ProxyEngine;
use intercept::InterceptEngine;
use certs::CertManager;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tauri::Emitter;

/// Global runtime for running async proxy server
pub struct AppRuntime {
    pub runtime: Runtime,
    pub engine: Arc<ProxyEngine>,
    pub intercept: Arc<InterceptEngine>,
    pub certs: Arc<CertManager>,
    pub app_handle: parking_lot::Mutex<Option<tauri::AppHandle>>,
    pub proxy_shutdown: parking_lot::Mutex<Option<tokio::sync::oneshot::Sender<()>>>,
}

impl Default for AppRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl AppRuntime {
    fn new() -> Self {
        let runtime = Runtime::new().expect("Failed to create tokio runtime");
        let engine = ProxyEngine::new(1000);
        let certs = CertManager::default();

        // Initialize cert manager with app data directory
        let cert_dir = Self::get_cert_dir();
        if let Err(e) = certs.init(cert_dir) {
            log::warn!("Cert manager init warning: {}", e);
        }

        Self {
            runtime,
            engine: Arc::new(engine),
            intercept: Arc::new(InterceptEngine::default()),
            certs: Arc::new(certs),
            app_handle: parking_lot::Mutex::new(None),
            proxy_shutdown: parking_lot::Mutex::new(None),
        }
    }

    fn get_cert_dir() -> std::path::PathBuf {
        if let Some(proj_dirs) = directories::ProjectDirs::from("com", "r-burp", "r-burp") {
            proj_dirs.data_local_dir().join("certs")
        } else {
            std::env::temp_dir().join("r-burp-certs")
        }
    }

    pub fn emit_event(&self, event: &str, payload: impl serde::Serialize + Clone) {
        if let Some(handle) = self.app_handle.lock().as_ref() {
            let _ = handle.emit(event, payload);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_runtime = Arc::new(AppRuntime::new());

    tauri::Builder::default()
        .manage(AppState::default())
        .manage(app_runtime.clone())
        .setup(move |app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Store app handle for event emission
            *app_runtime.app_handle.lock() = Some(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_name,
            commands::get_current_timestamp,
            commands::get_listeners,
            commands::start_listener,
            commands::stop_listener,
            commands::get_request_count,
            commands::add_listener,
            commands::remove_listener,
            commands::get_transactions,
            commands::get_transaction,
            commands::get_request_summaries,
            commands::get_traffic_stats,
            commands::clear_transactions,
            commands::start_proxy,
            commands::stop_proxy,
            commands::get_proxy_status,
            commands::enable_intercept,
            commands::disable_intercept,
            commands::is_intercept_enabled,
            commands::resume_intercept,
            commands::drop_intercept,
            commands::get_pending_intercepts,
            commands::add_rule,
            commands::remove_rule,
            commands::toggle_rule,
            commands::get_rules,
            commands::generate_ca_cert,
            commands::get_cert_info,
            commands::get_cert_pem,
            commands::export_har,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
