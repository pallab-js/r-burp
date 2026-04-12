use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;

/// Represents an HTTP request captured by the proxy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    pub id: String,
    pub method: String,
    pub url: String,
    pub path: String,
    pub query: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    pub body_text: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub host: String,
    pub content_type: Option<String>,
    pub content_length: usize,
}

/// Represents an HTTP response captured by the proxy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub id: String,
    pub status: u16,
    pub status_text: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    pub body_text: Option<String>,
    pub content_type: Option<String>,
    pub content_length: usize,
    pub duration_ms: u64,
}

/// A complete request/response pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpTransaction {
    pub id: String,
    pub request: HttpRequest,
    pub response: Option<HttpResponse>,
    pub is_complete: bool,
    pub is_intercepted: bool,
    pub is_modified: bool,
}

/// Summary view for the request list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestSummary {
    pub id: String,
    pub method: String,
    pub url: String,
    pub status: Option<u16>,
    pub content_type: Option<String>,
    pub content_length: usize,
    pub duration_ms: Option<u64>,
    pub timestamp: DateTime<Utc>,
    pub is_intercepted: bool,
}

/// Statistics about captured traffic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficStats {
    pub total_requests: usize,
    pub completed_requests: usize,
    pub intercepted_requests: usize,
    pub total_bytes: usize,
    pub avg_response_time_ms: Option<u64>,
}

/// The core proxy engine that manages HTTP traffic
pub struct ProxyEngine {
    pub transactions: RwLock<Vec<HttpTransaction>>,
    pub max_transactions: usize,
    pub stats: RwLock<TrafficStats>,
}

impl ProxyEngine {
    pub fn new(max_transactions: usize) -> Self {
        Self {
            transactions: RwLock::new(Vec::new()),
            max_transactions,
            stats: RwLock::new(TrafficStats {
                total_requests: 0,
                completed_requests: 0,
                intercepted_requests: 0,
                total_bytes: 0,
                avg_response_time_ms: None,
            }),
        }
    }

    /// Create a new transaction for an incoming request
    pub fn start_transaction(&self, request: HttpRequest) -> String {
        let id = request.id.clone();
        let transaction = HttpTransaction {
            id: id.clone(),
            request,
            response: None,
            is_complete: false,
            is_intercepted: false,
            is_modified: false,
        };

        let mut transactions = self.transactions.write();
        transactions.push(transaction);

        // Trim old transactions if we exceed the limit
        if transactions.len() > self.max_transactions {
            let drain_end = transactions.len() - self.max_transactions;
            transactions.drain(0..drain_end);
        }

        // Update stats
        let mut stats = self.stats.write();
        stats.total_requests = transactions.len();

        id
    }

    /// Complete a transaction with a response
    pub fn complete_transaction(&self, request_id: &str, response: HttpResponse) {
        let mut transactions = self.transactions.write();
        for txn in transactions.iter_mut() {
            if txn.request.id == request_id {
                txn.response = Some(response.clone());
                txn.is_complete = true;
                break;
            }
        }

        // Update stats
        let mut stats = self.stats.write();
        stats.completed_requests += 1;
        stats.total_bytes += response.content_length;
    }

    /// Get all transactions
    pub fn get_transactions(&self) -> Vec<HttpTransaction> {
        self.transactions.read().clone()
    }

    /// Get a single transaction by ID
    pub fn get_transaction(&self, id: &str) -> Option<HttpTransaction> {
        self.transactions
            .read()
            .iter()
            .find(|t| t.id == id)
            .cloned()
    }

    /// Get request summaries for the list view
    pub fn get_summaries(&self) -> Vec<RequestSummary> {
        self.transactions
            .read()
            .iter()
            .map(|t| RequestSummary {
                id: t.id.clone(),
                method: t.request.method.clone(),
                url: t.request.url.clone(),
                status: t.response.as_ref().map(|r| r.status),
                content_type: t.response.as_ref().and_then(|r| r.content_type.clone()),
                content_length: t.response.as_ref().map_or(0, |r| r.content_length),
                duration_ms: t.response.as_ref().map(|r| r.duration_ms),
                timestamp: t.request.timestamp,
                is_intercepted: t.is_intercepted,
            })
            .collect()
    }

    /// Get current stats
    pub fn get_stats(&self) -> TrafficStats {
        self.stats.read().clone()
    }

    /// Clear all transactions
    pub fn clear_transactions(&self) {
        self.transactions.write().clear();
        let mut stats = self.stats.write();
        stats.total_requests = 0;
        stats.completed_requests = 0;
        stats.intercepted_requests = 0;
        stats.total_bytes = 0;
        stats.avg_response_time_ms = None;
    }
}

impl Default for ProxyEngine {
    fn default() -> Self {
        Self::new(1000)
    }
}

/// Helper to convert bytes to text if it's valid UTF-8
pub fn bytes_to_text(bytes: &[u8]) -> Option<String> {
    String::from_utf8(bytes.to_vec()).ok()
}

/// Helper to detect content type from headers
pub fn detect_content_type(headers: &HashMap<String, String>) -> Option<String> {
    headers
        .get("content-type")
        .or_else(|| headers.get("Content-Type"))
        .map(|ct| ct.split(';').next().unwrap_or(ct).to_string())
}

/// Determine if a body should be displayed as text
pub fn is_text_body(content_type: Option<&str>) -> bool {
    match content_type {
        Some(ct) => ct.starts_with("text/")
            || ct.starts_with("application/json")
            || ct.starts_with("application/xml")
            || ct.starts_with("application/javascript")
            || ct.starts_with("image/svg"),
        None => false,
    }
}
