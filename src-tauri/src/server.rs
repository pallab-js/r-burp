use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::server::conn::http1 as http1_server;
use hyper::service::service_fn;
use hyper::Request;
use hyper::Response;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::TokioIo;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rustls::ServerConfig;
use rustls_pki_types::PrivateKeyDer;
use tauri::Emitter;
use crate::proxy::*;
use crate::intercept::{InterceptEngine, InterceptAction};
use crate::certs::CertManager;

const MAX_BODY_BYTES: usize = 10 * 1024 * 1024; // 10 MB
const MAX_HEADER_COUNT: usize = 100;

/// Convert SystemTime to chrono DateTime<Utc>
fn system_time_to_datetime(t: SystemTime) -> DateTime<Utc> {
    let duration = t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    DateTime::from_timestamp(duration.as_secs() as i64, duration.subsec_nanos())
        .unwrap_or_else(Utc::now)
}

/// The HTTP proxy server that intercepts and forwards traffic
pub struct ProxyServer {
    pub engine: Arc<ProxyEngine>,
    pub intercept: Arc<InterceptEngine>,
    pub certs: Arc<CertManager>,
    pub host: String,
    pub port: u16,
    pub app_handle: Option<tauri::AppHandle>,
}

impl ProxyServer {
    pub fn new(engine: Arc<ProxyEngine>, intercept: Arc<InterceptEngine>, certs: Arc<CertManager>, host: String, port: u16, app_handle: Option<tauri::AppHandle>) -> Self {
        Self { engine, intercept, certs, host, port, app_handle }
    }

    pub async fn start_with_shutdown(
        &mut self,
        mut shutdown_rx: tokio::sync::oneshot::Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr: SocketAddr = format!("{}:{}", self.host, self.port).parse()?;
        let listener = TcpListener::bind(addr).await?;
        let engine = self.engine.clone();
        let intercept = self.intercept.clone();
        let app_handle = self.app_handle.clone();

        log::info!("Proxy server listening on {}:{}", self.host, self.port);

        loop {
            tokio::select! {
                result = listener.accept() => {
                    let (stream, _) = result?;
                    let engine = engine.clone();
                    let intercept = intercept.clone();
                    let certs = self.certs.clone();
                    let handle = app_handle.clone();
                    tokio::task::spawn(async move {
                        if let Err(e) = Self::handle_connection(stream, engine, intercept, certs, handle).await {
                            log::warn!("Connection error: {}", e);
                        }
                    });
                }
                _ = &mut shutdown_rx => {
                    log::info!("Proxy server shutting down");
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_connection(
        stream: TcpStream,
        engine: Arc<ProxyEngine>,
        intercept: Arc<InterceptEngine>,
        certs: Arc<CertManager>,
        app_handle: Option<tauri::AppHandle>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut buf = [0u8; 16];
        let n = stream.peek(&mut buf).await?;
        let peeked = String::from_utf8_lossy(&buf[..n]);

        if peeked.starts_with("CONNECT ") {
            Self::handle_connect(stream, engine, intercept, certs, app_handle).await
        } else {
            Self::handle_http(stream, engine, intercept, app_handle).await
        }
    }

    async fn handle_connect(
        mut stream: TcpStream,
        engine: Arc<ProxyEngine>,
        intercept: Arc<InterceptEngine>,
        certs: Arc<CertManager>,
        app_handle: Option<tauri::AppHandle>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut buf = Vec::new();
        let mut header_buf = [0u8; 1];
        loop {
            let n = stream.read(&mut header_buf).await?;
            if n == 0 { return Ok(()); }
            buf.extend_from_slice(&header_buf[..n]);
            if buf.ends_with(b"\r\n\r\n") { break; }
            if buf.len() > 8192 {
                return Err("CONNECT request too large".into());
            }
        }

        let request_str = String::from_utf8_lossy(&buf);
        let first_line = request_str.lines().next().unwrap_or("");
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() < 2 {
            log::warn!("Malformed CONNECT request: {}", first_line);
            return Ok(());
        }

        let target = parts[1];
        let target_parts: Vec<&str> = target.split(':').collect();
        let host = target_parts.first().unwrap_or(&"unknown");
        let port = target_parts.get(1).unwrap_or(&"443");

        stream.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n").await?;

        let tunnel_url = format!("https://{}:{}", host, port);
        log::info!("HTTPS tunnel to {}", tunnel_url);

        let tunnel_id = Uuid::new_v4().to_string();
        let captured_request = HttpRequest {
            id: tunnel_id.clone(),
            method: "CONNECT".to_string(),
            url: tunnel_url.clone(),
            path: tunnel_url.clone(),
            query: String::new(),
            version: "HTTP/1.1".to_string(),
            headers: HashMap::new(),
            body: None,
            timestamp: system_time_to_datetime(SystemTime::now()),
            host: host.to_string(),
            content_type: None,
            content_length: 0,
        };
        engine.start_transaction(captured_request);
        emit_new_request(&app_handle, engine.get_stats());

        let (cert_pem, key_pem) = if let Some(pair) = certs.generate_domain_cert(host) {
            pair
        } else {
            log::warn!("Cannot generate cert for {}, tunneling without MITM", host);
            Self::blind_tunnel(stream, host.to_string(), port.to_string(), engine, tunnel_id, app_handle).await?;
            return Ok(());
        };

        use rustls_pki_types::pem::PemObject;
        use rustls_pki_types::{CertificateDer, PrivatePkcs8KeyDer};

        let certs_der: Vec<CertificateDer<'_>> = CertificateDer::pem_slice_iter(cert_pem.as_bytes())
            .filter_map(|c| c.ok())
            .collect();
        let key_der = PrivatePkcs8KeyDer::from_pem_slice(key_pem.as_bytes()).ok();
        let key: PrivateKeyDer<'_> = key_der.map(PrivateKeyDer::Pkcs8).ok_or("No private key found")?;

        let server_config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs_der, key)?;

        let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(server_config));
        let tls_stream = acceptor.accept(stream).await?;

        Self::handle_tls_stream(tls_stream, engine, intercept, app_handle).await
    }

    async fn blind_tunnel(
        stream: tokio::net::TcpStream,
        host: String,
        port: String,
        engine: Arc<ProxyEngine>,
        tunnel_id: String,
        app_handle: Option<tauri::AppHandle>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let target = TcpStream::connect(format!("{}:{}", host, port)).await?;
        let (mut target_r, mut target_w) = tokio::io::split(target);
        let (mut client_r, mut client_w) = tokio::io::split(stream);

        let client_to_target = tokio::spawn(async move {
            let _ = tokio::io::copy(&mut target_r, &mut client_w).await;
        });
        let target_to_client = tokio::io::copy(&mut client_r, &mut target_w).await?;
        let _ = client_to_target.await;

        log::info!("Blind tunnel to {}:{} closed - {} bytes forwarded", host, port, target_to_client);

        engine.complete_transaction(&tunnel_id, HttpResponse {
            id: Uuid::new_v4().to_string(),
            status: 200,
            status_text: "Tunneled (Blind)".to_string(),
            version: "HTTP/1.1".to_string(),
            headers: HashMap::new(),
            body: Some(format!("Blind tunnel to {}:{} - {} bytes forwarded.", host, port, target_to_client).into_bytes()),
            content_type: Some("text/plain".to_string()),
            content_length: 0,
            duration_ms: 0,
        });
        emit_stats_updated(&app_handle, engine.get_stats());

        Ok(())
    }

    async fn handle_tls_stream<S>(
        stream: S,
        engine: Arc<ProxyEngine>,
        intercept: Arc<InterceptEngine>,
        app_handle: Option<tauri::AppHandle>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
    {
        let io = TokioIo::new(stream);
        let svc = service_fn(move |req| {
            let engine = engine.clone();
            let intercept = intercept.clone();
            let handle = app_handle.clone();
            handle_request(engine, intercept, req, true, handle)
        });

        if let Err(e) = http1_server::Builder::new().keep_alive(true).serve_connection(io, svc).await {
            log::warn!("TLS stream error: {}", e);
        }

        Ok(())
    }

    async fn handle_http(
        stream: TcpStream,
        engine: Arc<ProxyEngine>,
        intercept: Arc<InterceptEngine>,
        app_handle: Option<tauri::AppHandle>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let io = TokioIo::new(stream);
        let svc = service_fn(move |req| {
            let engine = engine.clone();
            let intercept = intercept.clone();
            let handle = app_handle.clone();
            handle_request(engine, intercept, req, false, handle)
        });

        if let Err(e) = http1_server::Builder::new().serve_connection(io, svc).await {
            log::warn!("HTTP connection error: {}", e);
        }

        Ok(())
    }
}

fn emit_new_request(handle: &Option<tauri::AppHandle>, stats: TrafficStats) {
    if let Some(h) = handle {
        let _ = h.emit("proxy:stats-updated", &stats);
    }
}

fn emit_stats_updated(handle: &Option<tauri::AppHandle>, stats: TrafficStats) {
    if let Some(h) = handle {
        let _ = h.emit("proxy:stats-updated", &stats);
    }
}

async fn handle_request(
    engine: Arc<ProxyEngine>,
    intercept: Arc<InterceptEngine>,
    req: Request<Incoming>,
    is_tls: bool,
    app_handle: Option<tauri::AppHandle>,
) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
    let start_time = std::time::Instant::now();
    let request_timestamp = SystemTime::now();

    let method = req.method().to_string();
    let uri = req.uri().to_string();

    let mut headers: HashMap<String, String> = HashMap::new();
    for (name, value) in req.headers() {
        if let Ok(v) = value.to_str() {
            headers.insert(name.to_string(), v.to_string());
        }
    }

    let host = headers.get("host").cloned().unwrap_or_else(|| "unknown".to_string());

    if headers.len() > MAX_HEADER_COUNT {
        return Ok(Response::builder().status(400).body(Full::new(Bytes::from("Too many headers")))?);
    }

    if let Some(cl) = headers.get("content-length").and_then(|v| v.parse::<usize>().ok()) {
        if cl > MAX_BODY_BYTES {
            return Ok(Response::builder().status(413).body(Full::new(Bytes::from("Request body too large")))?);
        }
    }

    let content_type = detect_content_type(&headers);
    let body_bytes = req.collect().await?.to_bytes();

    let (final_method, final_url, final_headers, final_body) = if intercept.is_enabled() {
        let request_id = Uuid::new_v4().to_string();
        let body_text = body_as_text(&body_bytes, content_type.as_deref()).map(|s| s.to_string());

        if let Some(rx) = intercept.register_intercept(crate::intercept::InterceptRegistration {
            request_id: request_id.clone(),
            method: method.clone(),
            url: uri.clone(),
            headers: headers.clone(),
            body: Some(body_bytes.to_vec()),
            body_text,
            content_type: content_type.clone(),
            is_response: false,
            status: None,
            status_text: None,
        }) {
            match tokio::time::timeout(std::time::Duration::from_secs(30), rx).await {
                Ok(Ok(action)) => match action {
                    InterceptAction::Drop => {
                        return Ok(Response::builder().status(503).body(Full::new(Bytes::from("Request dropped by intercept")))?);
                    }
                    InterceptAction::Modify { method: m, url: u, headers: h, body: b } => {
                        (m.unwrap_or(method.clone()), u.unwrap_or(uri.clone()), h.unwrap_or(headers.clone()), b.unwrap_or(body_bytes.to_vec()))
                    }
                    InterceptAction::Forward => (method.clone(), uri.clone(), headers.clone(), body_bytes.to_vec()),
                },
                Ok(Err(_)) => (method.clone(), uri.clone(), headers.clone(), body_bytes.to_vec()),
                Err(_) => {
                    log::warn!("Intercept timeout for request {}", request_id);
                    (method.clone(), uri.clone(), headers.clone(), body_bytes.to_vec())
                }
            }
        } else {
            (method.clone(), uri.clone(), headers.clone(), body_bytes.to_vec())
        }
    } else {
        (method.clone(), uri.clone(), headers.clone(), body_bytes.to_vec())
    };

    // Apply rule actions
    let mut rule_modified_headers = final_headers.clone();
    let mut rule_modified_body = final_body.clone();
    let mut rule_modified_url = final_url.clone();
    let actions = intercept.get_rule_actions(&final_method, &final_url, &final_headers);
    for action in &actions {
        match action.action_type {
            crate::intercept::ActionType::AddHeader | crate::intercept::ActionType::ReplaceHeader => {
                rule_modified_headers.insert(action.target.clone(), action.value.clone());
            }
            crate::intercept::ActionType::RemoveHeader => {
                rule_modified_headers.remove(&action.target);
            }
            crate::intercept::ActionType::ReplaceBody => {
                rule_modified_body = action.value.as_bytes().to_vec();
            }
            crate::intercept::ActionType::AddQueryParam => {
                let sep = if rule_modified_url.contains('?') { "&" } else { "?" };
                rule_modified_url = format!("{}{}{}={}", rule_modified_url, sep, action.target, action.value);
            }
            crate::intercept::ActionType::RemoveQueryParam => {
                if let Some((base, query)) = rule_modified_url.split_once('?') {
                    let new_query: String = query
                        .split('&')
                        .filter(|p| !p.starts_with(&format!("{}=", action.target)) && *p != action.target)
                        .collect::<Vec<_>>()
                        .join("&");
                    rule_modified_url = if new_query.is_empty() { base.to_string() } else { format!("{}?{}", base, new_query) };
                }
            }
        }
    }

    let captured_request = HttpRequest {
        id: Uuid::new_v4().to_string(),
        method: final_method.clone(),
        url: final_url.clone(),
        path: final_url.clone(),
        query: String::new(),
        version: "HTTP/1.1".to_string(),
        headers: rule_modified_headers.clone(),
        body: Some(rule_modified_body.clone()),
        timestamp: system_time_to_datetime(request_timestamp),
        host: host.clone(),
        content_type: content_type.clone(),
        content_length: rule_modified_body.len(),
    };

    let request_id = captured_request.id.clone();
    engine.start_transaction(captured_request);
    emit_new_request(&app_handle, engine.get_stats());

    let response_result = forward_request(&host, &final_method, &rule_modified_url, &rule_modified_headers, &rule_modified_body, is_tls).await;
    let duration_ms = start_time.elapsed().as_millis() as u64;

    let (response_status, response_headers, response_body) = match response_result {
        Ok((status, resp_headers, body)) => (status, resp_headers, body),
        Err(e) => {
            log::warn!("Failed to forward request: {}", e);
            let mut resp_headers = HashMap::new();
            resp_headers.insert("content-type".to_string(), "text/plain".to_string());
            (502, resp_headers, Bytes::from(format!("Proxy error: {}", e)))
        }
    };

    let (final_resp_status, final_resp_headers, final_resp_body) = if intercept.is_enabled() {
        let intercept_id = Uuid::new_v4().to_string();
        let resp_ct = response_headers.get("content-type").map(|s| s.split(';').next().unwrap_or(s).to_string());
        let body_text = body_as_text(&response_body, resp_ct.as_deref()).map(|s| s.to_string());

        if let Some(rx) = intercept.register_intercept(crate::intercept::InterceptRegistration {
            request_id: intercept_id.clone(),
            method: method.clone(),
            url: uri.clone(),
            headers: response_headers.clone(),
            body: Some(response_body.to_vec()),
            body_text,
            content_type: resp_ct,
            is_response: true,
            status: Some(response_status),
            status_text: Some(status_text(response_status)),
        }) {
            match tokio::time::timeout(std::time::Duration::from_secs(30), rx).await {
                Ok(Ok(action)) => match action {
                    InterceptAction::Drop => {
                        let mut h = HashMap::new();
                        h.insert("content-type".to_string(), "text/plain".to_string());
                        (503, h, Bytes::from("Response dropped by intercept"))
                    }
                    InterceptAction::Modify { method: _, url: _, headers: h, body: b } => {
                        (response_status, h.unwrap_or(response_headers.clone()), Bytes::from(b.unwrap_or(response_body.to_vec())))
                    }
                    InterceptAction::Forward => (response_status, response_headers.clone(), response_body.clone()),
                },
                Ok(Err(_)) => (response_status, response_headers.clone(), response_body.clone()),
                Err(_) => {
                    log::warn!("Response intercept timeout for {}", intercept_id);
                    (response_status, response_headers.clone(), response_body.clone())
                }
            }
        } else {
            (response_status, response_headers.clone(), response_body.clone())
        }
    } else {
        (response_status, response_headers.clone(), response_body.clone())
    };

    let resp_ct = final_resp_headers.get("content-type").map(|s| s.split(';').next().unwrap_or(s).to_string());

    let captured_response = HttpResponse {
        id: Uuid::new_v4().to_string(),
        status: final_resp_status,
        status_text: status_text(final_resp_status),
        version: "HTTP/1.1".to_string(),
        headers: final_resp_headers.clone(),
        body: Some(final_resp_body.to_vec()),
        content_type: resp_ct,
        content_length: final_resp_body.len(),
        duration_ms,
    };

    engine.complete_transaction(&request_id, captured_response);
    emit_stats_updated(&app_handle, engine.get_stats());

    let mut builder = Response::builder().status(final_resp_status);
    for (name, value) in &final_resp_headers {
        builder = builder.header(name, value);
    }

    Ok(builder.body(Full::new(final_resp_body)).unwrap())
}

async fn forward_request(
    host: &str,
    method: &str,
    uri: &str,
    headers: &HashMap<String, String>,
    body: &[u8],
    is_tls: bool,
) -> Result<(u16, HashMap<String, String>, Bytes), Box<dyn std::error::Error + Send + Sync>> {
    let target_url = if uri.starts_with("http://") || uri.starts_with("https://") {
        uri.to_string()
    } else {
        let scheme = if is_tls { "https" } else { "http" };
        format!("{}://{}{}", scheme, host, uri)
    };

    let parsed = url::Url::parse(&target_url)?;
    let target_host = parsed.host_str().ok_or("No host in URL")?.to_string();
    let target_path = parsed.path();
    let query = parsed.query().unwrap_or("");
    let scheme = parsed.scheme();

    let full_uri = if query.is_empty() {
        format!("{}://{}{}", scheme, target_host, target_path)
    } else {
        format!("{}://{}{}?{}", scheme, target_host, target_path, query)
    };

    let mut req_builder = hyper::Request::builder()
        .method(method)
        .uri(&full_uri)
        .header("host", &target_host);

    for (name, value) in headers {
        let name_lower = name.to_lowercase();
        if name_lower != "host" && name_lower != "connection" {
            req_builder = req_builder.header(name.as_str(), value.as_str());
        }
    }

    let outbound_req = req_builder.body(Full::new(Bytes::from(body.to_vec())))?;

    if scheme == "https" {
        let mut root_store = rustls::RootCertStore::empty();
        let native_certs = rustls_native_certs::load_native_certs();
        for cert in native_certs.certs {
            let _ = root_store.add(cert);
        }
        let tls_config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();
        let https = hyper_rustls::HttpsConnectorBuilder::new()
            .with_tls_config(tls_config)
            .https_or_http()
            .enable_http1()
            .build();
        let client: hyper_util::client::legacy::Client<_, Full<Bytes>> =
            hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new()).build(https);
        let response = client.request(outbound_req).await?;
        let status = response.status().as_u16();
        let mut resp_headers: HashMap<String, String> = HashMap::new();
        for (name, value) in response.headers() {
            if let Ok(v) = value.to_str() { resp_headers.insert(name.to_string(), v.to_string()); }
        }
        let resp_body = response.collect().await?.to_bytes();
        if resp_body.len() > MAX_BODY_BYTES {
            return Err(format!("Response body too large: {} bytes", resp_body.len()).into());
        }
        Ok((status, resp_headers, resp_body))
    } else {
        let client: hyper_util::client::legacy::Client<HttpConnector, Full<Bytes>> =
            hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new()).build_http();
        let response = client.request(outbound_req).await?;
        let status = response.status().as_u16();
        let mut resp_headers: HashMap<String, String> = HashMap::new();
        for (name, value) in response.headers() {
            if let Ok(v) = value.to_str() { resp_headers.insert(name.to_string(), v.to_string()); }
        }
        let resp_body = response.collect().await?.to_bytes();
        if resp_body.len() > MAX_BODY_BYTES {
            return Err(format!("Response body too large: {} bytes", resp_body.len()).into());
        }
        Ok((status, resp_headers, resp_body))
    }
}

fn status_text(status: u16) -> String {
    match status {
        200 => "OK".to_string(),
        201 => "Created".to_string(),
        204 => "No Content".to_string(),
        301 => "Moved Permanently".to_string(),
        302 => "Found".to_string(),
        304 => "Not Modified".to_string(),
        400 => "Bad Request".to_string(),
        401 => "Unauthorized".to_string(),
        403 => "Forbidden".to_string(),
        404 => "Not Found".to_string(),
        500 => "Internal Server Error".to_string(),
        502 => "Bad Gateway".to_string(),
        503 => "Service Unavailable".to_string(),
        _ => format!("Status {}", status),
    }
}
