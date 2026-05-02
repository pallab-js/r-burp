use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
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
    pub transactions: RwLock<VecDeque<HttpTransaction>>,
    /// Maps transaction id → index in the deque (absolute, subtract base_offset to get deque index)
    tx_index: RwLock<HashMap<String, usize>>,
    /// Number of transactions ever popped from the front
    base_offset: RwLock<usize>,
    pub max_transactions: usize,
    pub stats: RwLock<TrafficStats>,
}

impl ProxyEngine {
    pub fn new(max_transactions: usize) -> Self {
        Self {
            transactions: RwLock::new(VecDeque::new()),
            tx_index: RwLock::new(HashMap::new()),
            base_offset: RwLock::new(0),
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
        let mut index = self.tx_index.write();
        let mut base = self.base_offset.write();

        // Evict oldest if at capacity
        if transactions.len() >= self.max_transactions {
            if let Some(evicted) = transactions.pop_front() {
                index.remove(&evicted.id);
                *base += 1;
            }
        }

        let abs_idx = *base + transactions.len();
        transactions.push_back(transaction);
        index.insert(id.clone(), abs_idx);

        let mut stats = self.stats.write();
        stats.total_requests = transactions.len();

        id
    }

    /// Complete a transaction with a response
    pub fn complete_transaction(&self, request_id: &str, response: HttpResponse) {
        let mut transactions = self.transactions.write();
        let index = self.tx_index.read();
        let base = self.base_offset.read();

        if let Some(&abs_idx) = index.get(request_id) {
            let deque_idx = abs_idx.saturating_sub(*base);
            if let Some(txn) = transactions.get_mut(deque_idx) {
                txn.response = Some(response.clone());
                txn.is_complete = true;
            }
        }

        let mut stats = self.stats.write();
        stats.completed_requests += 1;
        stats.total_bytes += response.content_length;
    }

    /// Get all transactions
    pub fn get_transactions(&self) -> Vec<HttpTransaction> {
        self.transactions.read().iter().cloned().collect()
    }

    /// Get a single transaction by ID — O(1)
    pub fn get_transaction(&self, id: &str) -> Option<HttpTransaction> {
        let transactions = self.transactions.read();
        let index = self.tx_index.read();
        let base = self.base_offset.read();
        let &abs_idx = index.get(id)?;
        let deque_idx = abs_idx.saturating_sub(*base);
        transactions.get(deque_idx).cloned()
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
        self.tx_index.write().clear();
        *self.base_offset.write() = 0;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_request(id: &str) -> HttpRequest {
        HttpRequest {
            id: id.to_string(),
            method: "GET".to_string(),
            url: format!("http://example.com/{}", id),
            path: format!("/{}", id),
            query: String::new(),
            version: "HTTP/1.1".to_string(),
            headers: HashMap::new(),
            body: None,
            timestamp: chrono::Utc::now(),
            host: "example.com".to_string(),
            content_type: None,
            content_length: 0,
        }
    }

    fn make_response(id: &str) -> HttpResponse {
        HttpResponse {
            id: id.to_string(),
            status: 200,
            status_text: "OK".to_string(),
            version: "HTTP/1.1".to_string(),
            headers: HashMap::new(),
            body: Some(b"hello".to_vec()),
            content_type: Some("text/plain".to_string()),
            content_length: 5,
            duration_ms: 10,
        }
    }

    #[test]
    fn test_start_and_complete_transaction() {
        let engine = ProxyEngine::new(10);
        let req = make_request("req1");
        let id = engine.start_transaction(req);
        assert_eq!(id, "req1");

        let txn = engine.get_transaction("req1").unwrap();
        assert!(!txn.is_complete);

        engine.complete_transaction("req1", make_response("resp1"));
        let txn = engine.get_transaction("req1").unwrap();
        assert!(txn.is_complete);
        assert_eq!(txn.response.unwrap().status, 200);
    }

    #[test]
    fn test_eviction_at_max_capacity() {
        let engine = ProxyEngine::new(3);
        for i in 0..4 {
            engine.start_transaction(make_request(&format!("req{}", i)));
        }
        // req0 should be evicted
        assert!(engine.get_transaction("req0").is_none());
        assert!(engine.get_transaction("req1").is_some());
        assert!(engine.get_transaction("req3").is_some());
        assert_eq!(engine.get_transactions().len(), 3);
    }

    #[test]
    fn test_get_summaries_shape() {
        let engine = ProxyEngine::new(10);
        engine.start_transaction(make_request("r1"));
        engine.complete_transaction("r1", make_response("resp1"));
        let summaries = engine.get_summaries();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].id, "r1");
        assert_eq!(summaries[0].status, Some(200));
    }

    #[test]
    fn test_clear_transactions() {
        let engine = ProxyEngine::new(10);
        engine.start_transaction(make_request("r1"));
        engine.clear_transactions();
        assert_eq!(engine.get_transactions().len(), 0);
        assert!(engine.get_transaction("r1").is_none());
    }

    #[test]
    fn test_body_as_text_text_type() {
        let body = b"hello world";
        assert_eq!(body_as_text(body, Some("text/plain")), Some("hello world"));
        assert_eq!(body_as_text(body, Some("application/json")), Some("hello world"));
    }

    #[test]
    fn test_body_as_text_binary_type() {
        let body = b"\x89PNG\r\n";
        assert_eq!(body_as_text(body, Some("image/png")), None);
        assert_eq!(body_as_text(body, None), None);
    }
}

/// Return body as text only if content-type indicates textual content.
/// Returns None for binary types, avoiding unnecessary allocation.
pub fn body_as_text<'a>(body: &'a [u8], content_type: Option<&str>) -> Option<&'a str> {
    if is_text_body(content_type) {
        std::str::from_utf8(body).ok()
    } else {
        None
    }
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
