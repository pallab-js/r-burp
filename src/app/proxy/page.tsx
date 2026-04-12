"use client";

import { useState, useEffect, useCallback, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { AppLayout } from "../../components/layout/app-layout";
import { RequestList } from "../../components/proxy/request-list";
import { RequestInspector } from "../../components/proxy/request-inspector";
import { Button } from "../../components/ui/button";
import { Badge } from "../../components/ui/badge";
import {
  getTransactions,
  getRequestSummaries,
  getTrafficStats,
  startProxy,
  stopProxy,
  clearTransactions,
  getProxyStatus,
  exportHar,
} from "../../lib/tauri-api";
import type { NavItem, RequestSummary, HttpTransaction, TrafficStats } from "../../types";
import { formatBytes } from "../../lib/utils";

const navItems: NavItem[] = [
  { id: "dashboard", label: "Dashboard", icon: "◉" },
  { id: "proxy", label: "Proxy", icon: "⇄" },
  { id: "interceptor", label: "Interceptor", icon: "◎" },
  { id: "requests", label: "Rules", icon: "▤" },
  { id: "settings", label: "Settings", icon: "⚙" },
];

export default function ProxyPage() {
  const [activeNav, setActiveNav] = useState("proxy");
  const [summaries, setSummaries] = useState<RequestSummary[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [selectedTx, setSelectedTx] = useState<HttpTransaction | null>(null);
  const [stats, setStats] = useState<TrafficStats>({
    total_requests: 0,
    completed_requests: 0,
    intercepted_requests: 0,
    total_bytes: 0,
    avg_response_time_ms: null,
  });
  const [proxyRunning, setProxyRunning] = useState(false);
  const [statusText, setStatusText] = useState("Idle");
  const [filterMethod, setFilterMethod] = useState("");
  const [searchQuery, setSearchQuery] = useState("");

  const refreshData = useCallback(async () => {
    try {
      const [s, txs, st, status] = await Promise.all([
        getRequestSummaries(),
        getTransactions(),
        getTrafficStats(),
        getProxyStatus(),
      ]);
      setSummaries(s.reverse());
      setStats(st);
      setStatusText(status);

      if (selectedId) {
        const found = txs.find((t) => t.id === selectedId) ?? null;
        setSelectedTx(found);
      }
    } catch {
      // Running in browser dev mode — no Tauri backend
    }
  }, [selectedId]);

  // Listen for Tauri events instead of polling
  useEffect(() => {
    refreshData();

    const unsubs: Array<() => void> = [];

    const setupListeners = async () => {
      const unsubStarted = await listen("proxy-started", () => {
        setProxyRunning(true);
        refreshData();
      });
      unsubs.push(unsubStarted);

      const unsubStopped = await listen("proxy-stopped", () => {
        setProxyRunning(false);
        refreshData();
      });
      unsubs.push(unsubStopped);

      const unsubCleared = await listen("transactions-cleared", () => {
        setSelectedId(null);
        setSelectedTx(null);
        refreshData();
      });
      unsubs.push(unsubCleared);
    };

    setupListeners();

    // Still poll for new transactions (events from Rust backend during proxy)
    const interval = setInterval(refreshData, 1000);
    return () => {
      clearInterval(interval);
      unsubs.forEach((unsub) => unsub());
    };
  }, [refreshData]);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const tag = (e.target as HTMLElement)?.tagName;
      const isInput = tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT";

      // Cmd/Ctrl+K: Clear transactions
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        handleClear();
      }
      // Escape: Deselect
      if (e.key === "Escape" && !isInput && selectedId) {
        setSelectedId(null);
        setSelectedTx(null);
      }
      // Arrow down: Select next (only when not in input)
      if (e.key === "ArrowDown" && !isInput && summaries.length > 0) {
        e.preventDefault();
        const currentIndex = summaries.findIndex((s) => s.id === selectedId);
        const nextIndex = currentIndex < summaries.length - 1 ? currentIndex + 1 : 0;
        setSelectedId(summaries[nextIndex].id);
      }
      // Arrow up: Select previous (only when not in input)
      if (e.key === "ArrowUp" && !isInput && summaries.length > 0) {
        e.preventDefault();
        const currentIndex = summaries.findIndex((s) => s.id === selectedId);
        const prevIndex = currentIndex > 0 ? currentIndex - 1 : summaries.length - 1;
        setSelectedId(summaries[prevIndex].id);
      }
      // Space: Toggle proxy (only when not in input, and focus is on body)
      if (e.key === " " && !isInput && document.activeElement === document.body) {
        e.preventDefault();
        if (proxyRunning) {
          handleStopProxy();
        } else {
          handleStartProxy();
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [summaries, selectedId, proxyRunning]);

  const handleStartProxy = async () => {
    try {
      await startProxy("127.0.0.1", 8080);
      setProxyRunning(true);
      refreshData();
    } catch {
      // Dev mode
    }
  };

  const handleStopProxy = async () => {
    try {
      await stopProxy();
      setProxyRunning(false);
      refreshData();
    } catch {
      // Dev mode
    }
  };

  const handleClear = async () => {
    try {
      await clearTransactions();
      setSelectedId(null);
      setSelectedTx(null);
      refreshData();
    } catch {
      // Dev mode
    }
  };

  const handleExport = async () => {
    try {
      const har = await exportHar();
      if (har) {
        const blob = new Blob([har], { type: "application/json" });
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = `r-burp-${new Date().toISOString().slice(0, 10)}.har`;
        a.click();
        URL.revokeObjectURL(url);
      }
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
            variant={proxyRunning ? "tertiary-pill" : "primary"}
            onClick={proxyRunning ? handleStopProxy : handleStartProxy}
          >
            {proxyRunning ? "Stop" : "Start"} Proxy
          </Button>
          <Button variant="ghost" onClick={handleClear}>
            Clear <span className="ml-1 text-[10px] opacity-50">⌘K</span>
          </Button>
          <Button variant="ghost" onClick={handleExport}>
            Export HAR
          </Button>
          <div className="flex-1" />
          <Badge color={proxyRunning ? "success" : "default"}>
            {statusText}
          </Badge>
          <span className="text-xs text-cursor-dark/40 font-mono">
            {stats.total_requests} captured • {formatBytes(stats.total_bytes)}
          </span>
        </div>

        {/* Main content: list + inspector split */}
        <div className="flex flex-1 overflow-hidden">
          {/* Request list */}
          <div className="w-96 border-r border-border-primary overflow-hidden">
            <RequestList
              requests={summaries}
              selectedId={selectedId}
              onSelect={(id) => {
                setSelectedId(id);
              }}
              filterMethod={filterMethod}
              onFilterChange={setFilterMethod}
              searchQuery={searchQuery}
              onSearchChange={setSearchQuery}
            />
          </div>

          {/* Request inspector */}
          <div className="flex-1 overflow-hidden">
            <RequestInspector transaction={selectedTx} />
          </div>
        </div>
      </div>
    </AppLayout>
  );
}
