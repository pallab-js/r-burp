import { invoke } from "@tauri-apps/api/core";
import type { ListenerConfig, HttpTransaction, RequestSummary, TrafficStats } from "../types";

// === Listener Commands ===

export async function getAppName(): Promise<string> {
  return invoke<string>("get_app_name");
}

export async function getCurrentTimestamp(): Promise<string> {
  return invoke<string>("get_current_timestamp");
}

export async function getListeners(): Promise<ListenerConfig[]> {
  return invoke<ListenerConfig[]>("get_listeners");
}

export async function startListener(id: number): Promise<boolean> {
  return invoke<boolean>("start_listener", { listenerId: id });
}

export async function stopListener(id: number): Promise<boolean> {
  return invoke<boolean>("stop_listener", { listenerId: id });
}

export async function getRequestCount(): Promise<number> {
  return invoke<number>("get_request_count");
}

export async function addListener(
  host: string,
  port: number,
  interceptHttps: boolean
): Promise<number> {
  return invoke<number>("add_listener", {
    host,
    port,
    interceptHttps,
  });
}

export async function removeListener(id: number): Promise<boolean> {
  return invoke<boolean>("remove_listener", { listenerId: id });
}

// === Proxy Engine Commands ===

export async function getTransactions(): Promise<HttpTransaction[]> {
  return invoke<HttpTransaction[]>("get_transactions");
}

export async function getTransaction(id: string): Promise<HttpTransaction | null> {
  return invoke<HttpTransaction | null>("get_transaction", { id });
}

export async function getRequestSummaries(): Promise<RequestSummary[]> {
  return invoke<RequestSummary[]>("get_request_summaries");
}

export async function getTrafficStats(): Promise<TrafficStats> {
  return invoke<TrafficStats>("get_traffic_stats");
}

export async function clearTransactions(): Promise<void> {
  return invoke("clear_transactions");
}

export async function startProxy(host: string, port: number): Promise<string> {
  return invoke<string>("start_proxy", { host, port });
}

export async function stopProxy(): Promise<string> {
  return invoke<string>("stop_proxy");
}

export async function getProxyStatus(): Promise<string> {
  return invoke<string>("get_proxy_status");
}

// === Intercept Commands ===

export async function enableIntercept(): Promise<void> {
  return invoke("enable_intercept");
}

export async function disableIntercept(): Promise<void> {
  return invoke("disable_intercept");
}

export async function isInterceptEnabled(): Promise<boolean> {
  return invoke<boolean>("is_intercept_enabled");
}

export async function resumeIntercept(
  requestId: string,
  method?: string,
  url?: string,
  headers?: Record<string, string>,
  body?: number[]
): Promise<boolean> {
  return invoke<boolean>("resume_intercept", {
    requestId,
    method,
    url,
    headers,
    body,
  });
}

export async function dropIntercept(requestId: string): Promise<boolean> {
  return invoke<boolean>("drop_intercept", { requestId });
}

export async function getPendingIntercepts(): Promise<PendingIntercept[]> {
  return invoke<PendingIntercept[]>("get_pending_intercepts");
}

export interface PendingIntercept {
  request_id: string;
  method: string;
  url: string;
  headers: Record<string, string>;
  body_text: string | null;
  content_type: string | null;
}

// === Rule Commands ===

export interface InterceptRule {
  id: string;
  name: string;
  enabled: boolean;
  match_type: string;
  match_pattern: string;
  actions: RuleAction[];
}

export interface RuleAction {
  action_type: string;
  target: string;
  value: string;
}

export async function getRules(): Promise<InterceptRule[]> {
  return invoke<InterceptRule[]>("get_rules");
}

export async function addRule(
  name: string,
  matchType: string,
  matchPattern: string,
  actions: RuleAction[]
): Promise<string> {
  return invoke<string>("add_rule", { name, matchType, matchPattern, actions });
}

export async function removeRule(id: string): Promise<boolean> {
  return invoke<boolean>("remove_rule", { id });
}

export async function toggleRule(id: string, enabled: boolean): Promise<boolean> {
  return invoke<boolean>("toggle_rule", { id, enabled });
}

// === Certificate Commands ===

export interface CertInfo {
  generated: boolean;
  fingerprint: string;
  cert_path: string;
  installed: boolean;
}

export async function generateCaCert(): Promise<string> {
  return invoke<string>("generate_ca_cert");
}

export async function getCertInfo(): Promise<CertInfo> {
  return invoke<CertInfo>("get_cert_info");
}

export async function getCertPem(): Promise<string | null> {
  return invoke<string | null>("get_cert_pem");
}

export async function exportHar(): Promise<string> {
  return invoke<string>("export_har");
}
