mod commands;
mod proxy;
mod server;
mod intercept;
mod certs;
pub mod error;

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
    /// Pre-built TLS client config using native system root certs
    pub tls_client_config: Arc<rustls::ClientConfig>,
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

        // Load or generate the CA key passphrase from the store
        let passphrase = Self::load_or_generate_passphrase();

        let certs = CertManager::with_passphrase(passphrase);

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
            tls_client_config: Arc::new(Self::build_tls_client_config()),
        }
    }

    fn build_tls_client_config() -> rustls::ClientConfig {
        let mut root_store = rustls::RootCertStore::empty();
        let native_certs = rustls_native_certs::load_native_certs();
        for cert in native_certs.certs {
            let _ = root_store.add(cert);
        }
        rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth()
    }

    /// Load the CA key passphrase from the store file, or generate and persist a new one.
    fn load_or_generate_passphrase() -> String {
        let store_path = Self::get_store_path();

        // Try to read existing passphrase from a simple JSON file
        if store_path.exists() {
            if let Ok(contents) = std::fs::read_to_string(&store_path) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&contents) {
                    if let Some(p) = val.get("ca_key_passphrase").and_then(|v| v.as_str()) {
                        if !p.is_empty() {
                            return p.to_string();
                        }
                    }
                }
            }
        }

        // Generate a new random 32-byte passphrase
        let bytes: [u8; 32] = rand::random();
        let passphrase = hex::encode(bytes);

        // Persist it
        let val = serde_json::json!({ "ca_key_passphrase": passphrase });
        if let Some(parent) = store_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&store_path, serde_json::to_string(&val).unwrap_or_default());

        // Restrict permissions on the store file
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = std::fs::metadata(&store_path) {
                let mut perms = meta.permissions();
                perms.set_mode(0o600);
                let _ = std::fs::set_permissions(&store_path, perms);
            }
        }

        passphrase
    }

    fn get_store_path() -> std::path::PathBuf {
        if let Some(proj_dirs) = directories::ProjectDirs::from("com", "r-burp", "r-burp") {
            proj_dirs.data_local_dir().join("store.json")
        } else {
            std::env::temp_dir().join("r-burp-store.json")
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
            app.handle().plugin(
                tauri_plugin_log::Builder::default()
                    .level(if cfg!(debug_assertions) {
                        log::LevelFilter::Info
                    } else {
                        log::LevelFilter::Warn
                    })
                    .build(),
            )?;

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
