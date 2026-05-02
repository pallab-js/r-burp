"use client";

import { useState, useEffect, useCallback } from "react";
import { AppLayout } from "../../components/layout/app-layout";
import { Button } from "../../components/ui/button";
import { Badge } from "../../components/ui/badge";
import { Input, Textarea } from "../../components/ui/input";
import {
  MAX_HEADER_NAME_LENGTH,
  MAX_HEADER_VALUE_LENGTH,
  MAX_BODY_LENGTH,
  MAX_JSON_HEADERS_LENGTH,
  validateBody,
} from "../../components/ui/input";
import {
  enableIntercept,
  disableIntercept,
  isInterceptEnabled,
  resumeIntercept,
  dropIntercept,
  getPendingIntercepts,
  type PendingIntercept,
} from "../../lib/tauri-api";
import type { NavItem } from "../../types";

const navItems: NavItem[] = [
  { id: "dashboard", label: "Dashboard", icon: "◉" },
  { id: "proxy", label: "Proxy", icon: "⇄" },
  { id: "interceptor", label: "Interceptor", icon: "◎" },
  { id: "requests", label: "Rules", icon: "▤" },
  { id: "settings", label: "Settings", icon: "⚙" },
];

export default function InterceptorPage() {
  const [activeNav, setActiveNav] = useState("interceptor");
  const [interceptEnabled, setInterceptEnabled] = useState(false);
  const [pending, setPending] = useState<PendingIntercept[]>([]);
  const [selected, setSelected] = useState<PendingIntercept | null>(null);
  const [editHeaders, setEditHeaders] = useState("");
  const [editBody, setEditBody] = useState("");
  const [headerError, setHeaderError] = useState<string | null>(null);
  const [bodyError, setBodyError] = useState<string | null>(null);

  const refreshPending = useCallback(async () => {
    try {
      const items = await getPendingIntercepts();
      setPending(items);
      if (selected && !items.find((p) => p.request_id === selected.request_id)) {
        setSelected(null);
      }
    } catch {
      // Dev mode
    }
  }, [selected]);

  useEffect(() => {
    const checkStatus = async () => {
      try {
        const enabled = await isInterceptEnabled();
        setInterceptEnabled(enabled);
      } catch {
        // Dev mode
      }
    };
    checkStatus();
    refreshPending();

    let unsub: (() => void) | undefined;
    const setup = async () => {
      const { listen } = await import("@tauri-apps/api/event");
      unsub = await listen("intercept:pending-updated", () => {
        checkStatus();
        refreshPending();
      });
    };
    setup();

    return () => { unsub?.(); };
  }, [refreshPending]);

  const handleToggleIntercept = async () => {
    try {
      if (interceptEnabled) {
        await disableIntercept();
        setInterceptEnabled(false);
      } else {
        await enableIntercept();
        setInterceptEnabled(true);
      }
    } catch {
      // Dev mode
    }
  };

  const handleSelect = (req: PendingIntercept) => {
    setSelected(req);
    setEditHeaders(JSON.stringify(req.headers, null, 2));
    setEditBody(req.body_text || "");
    setHeaderError(null);
    setBodyError(null);
  };

  const handleForward = async () => {
    if (!selected) return;
    try {
      await resumeIntercept(selected.request_id);
      setSelected(null);
      refreshPending();
    } catch {
      // Dev mode
    }
  };

  const handleModify = async () => {
    if (!selected) return;
    try {
      // Validate headers JSON and structure
      let headers: Record<string, string> | undefined;
      try {
        if (editHeaders.length > MAX_JSON_HEADERS_LENGTH) {
          setHeaderError(`Headers JSON must be ${MAX_JSON_HEADERS_LENGTH.toLocaleString()} characters or less`);
          return;
        }
        const parsed = JSON.parse(editHeaders);
        if (typeof parsed !== 'object' || parsed === null || Array.isArray(parsed)) {
          setHeaderError("Headers must be a JSON object");
          return;
        }
        // Validate each header key and value
        for (const [key, value] of Object.entries(parsed)) {
          if (typeof key !== 'string') {
            setHeaderError("All header keys must be strings");
            return;
          }
          if (typeof value !== 'string') {
            setHeaderError(`Value for header "${key}" must be a string`);
            return;
          }
          if (key.length > MAX_HEADER_NAME_LENGTH) {
            setHeaderError(`Header name "${key}" exceeds ${MAX_HEADER_NAME_LENGTH} character limit`);
            return;
          }
          if (value.length > MAX_HEADER_VALUE_LENGTH) {
            setHeaderError(`Header value for "${key}" exceeds ${MAX_HEADER_VALUE_LENGTH.toLocaleString()} character limit`);
            return;
          }
          // Prevent prototype pollution keys
          if (key === "__proto__" || key === "constructor" || key === "prototype") {
            setHeaderError(`Invalid header name: ${key}`);
            return;
          }
        }
        headers = parsed as Record<string, string>;
        setHeaderError(null);
      } catch (e: any) {
        setHeaderError(e.message || "Invalid JSON");
        return;
      }

      // Validate body
      let bodyBytes: number[] | undefined;
      if (editBody) {
        const bodyValidationError = validateBody(editBody);
        if (bodyValidationError) {
          setBodyError(bodyValidationError);
          return;
        }
        bodyBytes = Array.from(new TextEncoder().encode(editBody));
        setBodyError(null);
      }

      await resumeIntercept(selected.request_id, undefined, undefined, headers, bodyBytes);
      setSelected(null);
      refreshPending();
    } catch {
      // Dev mode
    }
  };

  const handleDrop = async () => {
    if (!selected) return;
    try {
      await dropIntercept(selected.request_id);
      setSelected(null);
      refreshPending();
    } catch {
      // Dev mode
    }
  };

  return (
    <AppLayout
      navItems={navItems}
      activeItem={activeNav}
    >
      <div className="flex flex-col h-screen">
        {/* Toolbar */}
        <div className="flex items-center gap-3 px-4 py-2 border-b border-border-primary bg-surface-400">
          <Button
            variant={interceptEnabled ? "tertiary-pill" : "primary"}
            onClick={handleToggleIntercept}
          >
            {interceptEnabled ? "Intercept ON" : "Intercept OFF"}
          </Button>
          <div className="flex-1" />
          <Badge color={interceptEnabled ? "error" : "default"}>
            {interceptEnabled ? `Waiting... (${pending.length} pending)` : "Disabled"}
          </Badge>
        </div>

        <div className="flex flex-1 overflow-hidden">
          {/* Pending list */}
          <div className="w-80 border-r border-border-primary overflow-y-auto p-4">
            <h2
              className="text-xl font-gothic text-cursor-dark mb-4"
              style={{ letterSpacing: "-0.3px" }}
            >
              Pending Requests
            </h2>
            {pending.length === 0 ? (
              <p className="text-sm text-cursor-dark/40">
                {interceptEnabled ? "Waiting for requests..." : "Enable intercept to begin"}
              </p>
            ) : (
              <div className="space-y-2">
                {pending.map((req) => (
                  <button
                    key={req.request_id}
                    onClick={() => handleSelect(req)}
                    className={`w-full text-left p-3 rounded-comfortable border transition-colors duration-150 ${
                      selected?.request_id === req.request_id
                        ? "bg-surface-300 border-border-medium"
                        : "bg-surface-400 border-border-primary hover:bg-surface-300/50"
                    }`}
                  >
                    <div className="flex items-center gap-2 mb-1">
                      <Badge color="read">{req.method}</Badge>
                    </div>
                    <p className="text-xs text-cursor-dark/55 truncate font-mono">
                      {req.url}
                    </p>
                  </button>
                ))}
              </div>
            )}
          </div>

          {/* Modifier panel */}
          <div className="flex-1 overflow-y-auto p-6">
            {selected ? (
              <div className="max-w-3xl space-y-6">
                <div>
                  <h3
                    className="text-lg font-gothic text-cursor-dark mb-3"
                    style={{ letterSpacing: "-0.2px" }}
                  >
                    Modify Request
                  </h3>
                  <div className="mb-4 flex items-center gap-3">
                    <Badge color="read">{selected.method}</Badge>
                    <span className="text-xs text-cursor-dark/40 font-mono">{selected.url}</span>
                  </div>
                  <Textarea
                    label="Headers (JSON)"
                    value={editHeaders}
                    onChange={(e) => { setEditHeaders(e.target.value); setHeaderError(null); }}
                    error={headerError || undefined}
                    className="font-mono text-xs"
                    rows={8}
                    maxLength={MAX_JSON_HEADERS_LENGTH}
                  />
                  <div className="mt-4">
                    <Textarea
                      label="Body"
                      value={editBody}
                      onChange={(e) => { setEditBody(e.target.value); setBodyError(null); }}
                      error={bodyError || undefined}
                      className="font-mono text-xs"
                      rows={8}
                      maxLength={MAX_BODY_LENGTH}
                    />
                  </div>
                </div>

                {/* Actions */}
                <div className="flex gap-3 pt-4 border-t border-border-primary">
                  <Button variant="primary" onClick={handleModify}>
                    Forward (Modified)
                  </Button>
                  <Button variant="ghost" onClick={handleForward}>
                    Forward (Original)
                  </Button>
                  <Button variant="ghost" onClick={handleDrop}>
                    Drop
                  </Button>
                </div>
              </div>
            ) : (
              <div className="flex items-center justify-center h-full text-cursor-dark/30">
                <p className="text-sm">
                  {interceptEnabled
                    ? "Waiting for intercepted request..."
                    : "Enable intercept to begin"}
                </p>
              </div>
            )}
          </div>
        </div>
      </div>
    </AppLayout>
  );
}
