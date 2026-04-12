"use client";

import { useState, useEffect, useCallback } from "react";
import { useRouter } from "next/navigation";
import { invoke } from "@tauri-apps/api/core";
import { AppLayout } from "../components/layout/app-layout";
import { Card, Badge } from "../components/ui";
import type { NavItem, TrafficStats } from "../types";
import { formatBytes } from "../lib/utils";

const navItems: NavItem[] = [
  { id: "dashboard", label: "Dashboard", icon: "◉" },
  { id: "proxy", label: "Proxy", icon: "⇄" },
  { id: "interceptor", label: "Interceptor", icon: "◎" },
  { id: "requests", label: "Rules", icon: "▤" },
  { id: "settings", label: "Settings", icon: "⚙" },
];

export default function Home() {
  const router = useRouter();
  const [activeNav, setActiveNav] = useState("dashboard");
  const [isTauri, setIsTauri] = useState(false);
  const [appName, setAppName] = useState("r-burp");
  const [stats, setStats] = useState<TrafficStats>({
    total_requests: 0,
    completed_requests: 0,
    intercepted_requests: 0,
    total_bytes: 0,
    avg_response_time_ms: null,
  });
  const [proxyStatus, setProxyStatus] = useState("Idle");

  const refreshDashboard = useCallback(async () => {
    try {
      const [name, status, s] = await Promise.all([
        invoke<string>("get_app_name"),
        invoke<string>("get_proxy_status"),
        invoke<TrafficStats>("get_traffic_stats"),
      ]);
      setAppName(name);
      setProxyStatus(status);
      setStats(s);
      setIsTauri(true);
    } catch {
      setIsTauri(false);
    }
  }, []);

  useEffect(() => {
    refreshDashboard();
    const interval = setInterval(refreshDashboard, 2000);
    return () => clearInterval(interval);
  }, [refreshDashboard]);

  return (
    <AppLayout
      navItems={navItems}
      activeItem={activeNav}
    >
      <div className="p-8">
        {/* Header */}
        <div className="mb-8">
          <h1
            className="text-4xl font-gothic text-cursor-dark mb-2"
            style={{ letterSpacing: "-1px" }}
          >
            Dashboard
          </h1>
          <p className="text-cursor-dark/55 text-base-serif-sm">
            Welcome to {appName} — your lightweight HTTP proxy toolkit
          </p>
        </div>

        {/* Status Overview */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 mb-8">
          <Card className="p-5">
            <div className="flex items-center justify-between mb-3">
              <h3
                className="text-sm font-gothic text-cursor-dark/55 uppercase tracking-wide"
                style={{ letterSpacing: "0.05px" }}
              >
                Proxy Status
              </h3>
              <Badge color={proxyStatus.includes("Active") ? "success" : "default"}>
                {proxyStatus}
              </Badge>
            </div>
            <p className="text-2xl font-gothic text-cursor-dark" style={{ letterSpacing: "-0.3px" }}>
              {proxyStatus.includes("Idle") ? "—" : "127.0.0.1:8080"}
            </p>
            <p className="text-xs text-cursor-dark/40 mt-2">
              {proxyStatus.includes("Idle") ? "Not listening" : "Listening for connections"}
            </p>
          </Card>

          <Card className="p-5">
            <div className="flex items-center justify-between mb-3">
              <h3
                className="text-sm font-gothic text-cursor-dark/55 uppercase tracking-wide"
                style={{ letterSpacing: "0.05px" }}
              >
                Requests Captured
              </h3>
              <Badge color={stats.total_requests > 0 ? "grep" : "default"}>
                {stats.total_requests > 0 ? "Live" : "Idle"}
              </Badge>
            </div>
            <p className="text-2xl font-gothic text-cursor-dark" style={{ letterSpacing: "-0.3px" }}>
              {stats.total_requests}
            </p>
            <p className="text-xs text-cursor-dark/40 mt-2">
              {stats.total_requests === 0
                ? "No requests intercepted yet"
                : `${stats.completed_requests} completed`}
            </p>
          </Card>

          <Card className="p-5">
            <div className="flex items-center justify-between mb-3">
              <h3
                className="text-sm font-gothic text-cursor-dark/55 uppercase tracking-wide"
                style={{ letterSpacing: "0.05px" }}
              >
                Backend
              </h3>
              <Badge color={isTauri ? "success" : "default"}>
                {isTauri ? "Connected" : "Dev Mode"}
              </Badge>
            </div>
            <p className="text-2xl font-gothic text-cursor-dark" style={{ letterSpacing: "-0.3px" }}>
              {formatBytes(stats.total_bytes)}
            </p>
            <p className="text-xs text-cursor-dark/40 mt-2">
              Total traffic captured
            </p>
          </Card>
        </div>

        {/* Quick Actions */}
        <Card className="p-6">
          <h2
            className="text-2xl font-gothic text-cursor-dark mb-4"
            style={{ letterSpacing: "-0.5px" }}
          >
            Quick Actions
          </h2>
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
            <QuickAction
              title="Start Listener"
              description="Begin intercepting HTTP traffic on port 8080"
              action="Start"
              onClick={() => router.push("/proxy")}
            />
            <QuickAction
              title="Configure Rules"
              description="Set up request/response modification rules"
              action="Configure"
              onClick={() => router.push("/requests")}
            />
            <QuickAction
              title="Import Certificates"
              description="Install CA certificate for HTTPS interception"
              action="Import"
              onClick={() => router.push("/settings")}
            />
            <QuickAction
              title="View Documentation"
              description="Learn how to use r-burp effectively"
              action="Read"
              onClick={() => {}}
            />
          </div>
        </Card>
      </div>
    </AppLayout>
  );
}

function QuickAction({
  title,
  description,
  action,
  onClick,
}: {
  title: string;
  description: string;
  action: string;
  onClick: () => void;
}) {
  return (
    <div className="bg-surface-300 rounded-comfortable border border-border-primary p-4 hover:shadow-ambient transition-shadow duration-200">
      <h3
        className="text-base font-gothic text-cursor-dark mb-1"
        style={{ letterSpacing: "-0.1px" }}
      >
        {title}
      </h3>
      <p className="text-xs text-cursor-dark/55 mb-3 leading-relaxed">
        {description}
      </p>
      <button
        onClick={onClick}
        className="text-xs font-gothic text-accent hover:text-error transition-colors duration-150 ease cursor-pointer"
      >
        {action} →
      </button>
    </div>
  );
}
