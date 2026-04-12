export interface HttpRequest {
  id: string;
  method: string;
  url: string;
  path: string;
  query: string;
  version: string;
  headers: Record<string, string>;
  body: number[] | null;
  body_text: string | null;
  timestamp: string;
  host: string;
  content_type: string | null;
  content_length: number;
}

export interface HttpResponse {
  id: string;
  status: number;
  status_text: string;
  version: string;
  headers: Record<string, string>;
  body: number[] | null;
  body_text: string | null;
  content_type: string | null;
  content_length: number;
  duration_ms: number;
}

export interface HttpTransaction {
  id: string;
  request: HttpRequest;
  response: HttpResponse | null;
  is_complete: boolean;
  is_intercepted: boolean;
  is_modified: boolean;
}

export interface RequestSummary {
  id: string;
  method: string;
  url: string;
  status: number | null;
  content_type: string | null;
  content_length: number;
  duration_ms: number | null;
  timestamp: string;
  is_intercepted: boolean;
}

export interface TrafficStats {
  total_requests: number;
  completed_requests: number;
  intercepted_requests: number;
  total_bytes: number;
  avg_response_time_ms: number | null;
}

export interface ListenerConfig {
  id: number;
  host: string;
  port: number;
  is_running: boolean;
  intercept_https: boolean;
}

export type AppStatus = "running" | "stopped" | "error";

export interface NavItem {
  id: string;
  label: string;
  icon?: string;
}
