"use client";

import { useState } from "react";
import type { HttpTransaction } from "../../types";
import { Card } from "../ui/card";
import { Badge } from "../ui/badge";
import { getMethodColor } from "../../lib/utils";
import clsx from "clsx";

interface RequestInspectorProps {
  transaction: HttpTransaction | null;
}

export function RequestInspector({ transaction }: RequestInspectorProps) {
  const [activeTab, setActiveTab] = useState<"request" | "response">("request");
  const [subTab, setSubTab] = useState<"headers" | "body">("headers");

  if (!transaction) {
    return (
      <div className="flex items-center justify-center h-full text-cursor-dark/40 text-sm">
        Select a request to inspect
      </div>
    );
  }

  const { request, response } = transaction;

  return (
    <div className="flex flex-col h-full overflow-hidden">
      {/* Top bar */}
      <div className="flex items-center gap-3 px-4 py-3 border-b border-border-primary bg-surface-400">
        <Badge color={getMethodColor(request.method)}>{request.method}</Badge>
        <span className="text-sm font-mono text-cursor-dark/70 truncate flex-1">
          {request.url}
        </span>
        {response && (
          <Badge color={getStatusBadgeColor(response.status)}>
            {response.status} {response.status_text}
          </Badge>
        )}
        {response && (
          <span className="text-xs text-cursor-dark/40 font-mono">
            {response.duration_ms}ms
          </span>
        )}
      </div>

      {/* Tabs */}
      <div className="flex border-b border-border-primary">
        <TabButton
          label="Request"
          active={activeTab === "request"}
          onClick={() => { setActiveTab("request"); setSubTab("headers"); }}
        />
        <TabButton
          label="Response"
          active={activeTab === "response"}
          onClick={() => { setActiveTab("response"); setSubTab("headers"); }}
          disabled={!response}
        />
      </div>

      {/* Sub-tabs */}
      <div className="flex border-b border-border-primary/50 px-4">
        <SubTabButton
          label="Headers"
          active={subTab === "headers"}
          onClick={() => setSubTab("headers")}
        />
        <SubTabButton
          label="Body"
          active={subTab === "body"}
          onClick={() => setSubTab("body")}
        />
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-4">
        {activeTab === "request" && subTab === "headers" && (
          <HeadersView headers={request.headers} />
        )}
        {activeTab === "request" && subTab === "body" && (
          <BodyView body={request.body_text} contentType={request.content_type} />
        )}
        {activeTab === "response" && response && subTab === "headers" && (
          <HeadersView headers={response.headers} />
        )}
        {activeTab === "response" && response && subTab === "body" && (
          <BodyView body={response.body_text} contentType={response.content_type} />
        )}
      </div>
    </div>
  );
}

function TabButton({
  label,
  active,
  onClick,
  disabled,
}: {
  label: string;
  active: boolean;
  onClick: () => void;
  disabled?: boolean;
}) {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      className={clsx(
        "px-4 py-2 text-sm font-gothic border-b-2 transition-colors duration-150 ease",
        active
          ? "border-border-solid text-cursor-dark"
          : "border-transparent text-cursor-dark/40 hover:text-cursor-dark/60",
        disabled && "opacity-30 cursor-not-allowed"
      )}
    >
      {label}
    </button>
  );
}

function SubTabButton({
  label,
  active,
  onClick,
}: {
  label: string;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className={clsx(
        "px-3 py-1.5 text-xs text-cursor-dark/55 transition-colors duration-150 ease",
        active
          ? "text-cursor-dark font-medium"
          : "hover:text-cursor-dark/75"
      )}
    >
      {label}
    </button>
  );
}

function HeadersView({ headers }: { headers: Record<string, string> }) {
  const entries = Object.entries(headers);

  if (entries.length === 0) {
    return <p className="text-sm text-cursor-dark/40">No headers</p>;
  }

  return (
    <div className="space-y-1">
      {entries.map(([name, value]) => (
        <div key={name} className="flex gap-2 text-xs font-mono">
          <span className="text-cursor-dark/55 shrink-0 w-48 truncate">
            {name}:
          </span>
          <span className="text-cursor-dark/80 break-all">{value}</span>
        </div>
      ))}
    </div>
  );
}

function BodyView({
  body,
  contentType,
}: {
  body: string | null;
  contentType: string | null;
}) {
  if (!body) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-cursor-dark/30">
        <p className="text-sm">No body content</p>
        {contentType && (
          <p className="text-xs mt-1 font-mono">{contentType}</p>
        )}
      </div>
    );
  }

  const isJson = contentType?.includes("json");
  const isHtml = contentType?.includes("html");
  const displayBody = isJson ? formatJson(body) : body;

  return (
    <Card variant="compact" className="p-3 bg-surface-100">
      <pre
        className={clsx(
          "text-xs font-mono text-cursor-dark/80 whitespace-pre-wrap break-all",
          isHtml && "text-cursor-dark/60"
        )}
      >
        {displayBody}
      </pre>
    </Card>
  );
}

function getStatusBadgeColor(status: number): "success" | "grep" | "thinking" | "error" | "default" {
  if (status >= 200 && status < 300) return "success";
  if (status >= 300 && status < 400) return "grep";
  if (status >= 400 && status < 500) return "thinking";
  if (status >= 500) return "error";
  return "default";
}

function formatJson(body: string): string {
  try {
    return JSON.stringify(JSON.parse(body), null, 2);
  } catch {
    return body;
  }
}
