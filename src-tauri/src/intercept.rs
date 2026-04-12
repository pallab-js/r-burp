use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use parking_lot::RwLock;
use tokio::sync::oneshot;

/// A pending intercepted request waiting for frontend response
pub struct PendingIntercept {
    pub request_id: String,
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    pub body_text: Option<String>,
    pub content_type: Option<String>,
    pub sender: oneshot::Sender<InterceptAction>,
    pub is_response: bool,
    pub status: Option<u16>,
    pub status_text: Option<String>,
}

/// Action to take on an intercepted request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterceptAction {
    /// Forward the request as-is
    Forward,
    /// Forward with modified data
    Modify {
        method: Option<String>,
        url: Option<String>,
        headers: Option<HashMap<String, String>>,
        body: Option<Vec<u8>>,
    },
    /// Drop the request (return error to client)
    Drop,
}

/// Intercept rule for automatic request modification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterceptRule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub match_type: RuleMatchType,
    pub match_pattern: String,
    pub actions: Vec<RuleAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleMatchType {
    UrlContains,
    UrlRegex,
    MethodEquals,
    HeaderContains,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleAction {
    pub action_type: ActionType,
    pub target: String, // header name, body path, etc.
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    AddHeader,
    RemoveHeader,
    ReplaceHeader,
    ReplaceBody,
    AddQueryParam,
    RemoveQueryParam,
}

/// The intercept engine manages pending intercepts and rules
pub struct InterceptEngine {
    pub enabled: RwLock<bool>,
    pub pending: RwLock<HashMap<String, PendingIntercept>>,
    pub rules: RwLock<Vec<InterceptRule>>,
    pub rule_counter: RwLock<u32>,
}

impl Default for InterceptEngine {
    fn default() -> Self {
        Self {
            enabled: RwLock::new(false),
            pending: RwLock::new(HashMap::new()),
            rules: RwLock::new(Vec::new()),
            rule_counter: RwLock::new(0),
        }
    }
}

/// Parameters for registering a new intercept
pub struct InterceptRegistration {
    pub request_id: String,
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    pub body_text: Option<String>,
    pub content_type: Option<String>,
    pub is_response: bool,
    pub status: Option<u16>,
    pub status_text: Option<String>,
}

impl InterceptEngine {
    /// Register a new pending intercept, returns the receiver for the action
    pub fn register_intercept(
        &self,
        registration: InterceptRegistration,
    ) -> Option<oneshot::Receiver<InterceptAction>> {
        if !*self.enabled.read() {
            return None;
        }

        let (tx, rx) = oneshot::channel();
        let pending = PendingIntercept {
            request_id: registration.request_id.clone(),
            method: registration.method,
            url: registration.url,
            headers: registration.headers,
            body: registration.body,
            body_text: registration.body_text,
            content_type: registration.content_type,
            sender: tx,
            is_response: registration.is_response,
            status: registration.status,
            status_text: registration.status_text,
        };

        self.pending.write().insert(registration.request_id, pending);
        Some(rx)
    }

    /// Resume an intercepted request with an action
    pub fn resume_intercept(&self, request_id: &str, action: InterceptAction) -> bool {
        let mut pending = self.pending.write();
        if let Some(p) = pending.remove(request_id) {
            p.sender.send(action).is_ok()
        } else {
            false
        }
    }

    /// Check if a request matches any rule and get the actions to apply
    pub fn get_rule_actions(&self, method: &str, url: &str, headers: &HashMap<String, String>) -> Vec<RuleAction> {
        let rules = self.rules.read();
        rules
            .iter()
            .filter(|r| r.enabled && self.matches_rule(r, method, url, headers))
            .flat_map(|r| r.actions.clone())
            .collect()
    }

    fn matches_rule(&self, rule: &InterceptRule, method: &str, url: &str, headers: &HashMap<String, String>) -> bool {
        match rule.match_type {
            RuleMatchType::UrlContains => url.contains(&rule.match_pattern),
            RuleMatchType::MethodEquals => method.eq_ignore_ascii_case(&rule.match_pattern),
            RuleMatchType::UrlRegex => {
                if let Ok(re) = regex::Regex::new(&rule.match_pattern) {
                    re.is_match(url)
                } else {
                    false
                }
            }
            RuleMatchType::HeaderContains => {
                let parts: Vec<&str> = rule.match_pattern.splitn(2, ':').collect();
                if parts.len() == 2 {
                    headers
                        .get(parts[0].trim())
                        .map(|v| v.contains(parts[1].trim()))
                        .unwrap_or(false)
                } else {
                    false
                }
            }
        }
    }

    /// Add a new rule
    pub fn add_rule(&self, name: String, match_type: RuleMatchType, match_pattern: String, actions: Vec<RuleAction>) -> String {
        let mut counter = self.rule_counter.write();
        *counter += 1;
        let id = format!("rule_{}", *counter);

        let rule = InterceptRule {
            id: id.clone(),
            name,
            enabled: true,
            match_type,
            match_pattern,
            actions,
        };

        self.rules.write().push(rule);
        id
    }

    /// Remove a rule
    pub fn remove_rule(&self, id: &str) -> bool {
        let mut rules = self.rules.write();
        let initial_len = rules.len();
        rules.retain(|r| r.id != id);
        rules.len() < initial_len
    }

    /// Toggle a rule
    pub fn toggle_rule(&self, id: &str, enabled: bool) -> bool {
        let mut rules = self.rules.write();
        for rule in rules.iter_mut() {
            if rule.id == id {
                rule.enabled = enabled;
                return true;
            }
        }
        false
    }

    /// Get all rules
    pub fn get_rules(&self) -> Vec<InterceptRule> {
        self.rules.read().clone()
    }

    /// Get pending intercepts for display
    pub fn get_pending(&self) -> Vec<PendingInterceptSummary> {
        self.pending
            .read()
            .values()
            .map(|p| PendingInterceptSummary {
                request_id: p.request_id.clone(),
                method: p.method.clone(),
                url: p.url.clone(),
                headers: p.headers.clone(),
                body_text: p.body_text.clone(),
                content_type: p.content_type.clone(),
            })
            .collect()
    }

    /// Enable or disable intercept
    pub fn set_enabled(&self, enabled: bool) {
        *self.enabled.write() = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        *self.enabled.read()
    }
}

/// Summary of a pending intercept for the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingInterceptSummary {
    pub request_id: String,
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body_text: Option<String>,
    pub content_type: Option<String>,
}
