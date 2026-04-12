"use client";

import { useState, useEffect, useCallback } from "react";
import { AppLayout } from "../../components/layout/app-layout";
import { Button } from "../../components/ui/button";
import { Card } from "../../components/ui/card";
import { Badge } from "../../components/ui/badge";
import {
  getCertInfo,
  generateCaCert,
  getCertPem,
} from "../../lib/tauri-api";
import type { NavItem } from "../../types";

const navItems: NavItem[] = [
  { id: "dashboard", label: "Dashboard", icon: "◉" },
  { id: "proxy", label: "Proxy", icon: "⇄" },
  { id: "interceptor", label: "Interceptor", icon: "◎" },
  { id: "requests", label: "Rules", icon: "▤" },
  { id: "settings", label: "Settings", icon: "⚙" },
];

interface CertInfoData {
  generated: boolean;
  fingerprint: string;
  cert_path: string;
  installed: boolean;
}

export default function SettingsPage() {
  const [activeNav, setActiveNav] = useState("settings");
  const [certInfo, setCertInfo] = useState<CertInfoData | null>(null);
  const [certPem, setCertPem] = useState<string | null>(null);
  const [showCert, setShowCert] = useState(false);
  const [generating, setGenerating] = useState(false);
  const [copied, setCopied] = useState(false);

  const refreshCertInfo = useCallback(async () => {
    try {
      const info = await getCertInfo();
      setCertInfo(info);

      if (info.generated) {
        const pem = await getCertPem();
        setCertPem(pem);
      }
    } catch {
      // Dev mode
    }
  }, []);

  useEffect(() => {
    refreshCertInfo();
  }, [refreshCertInfo]);

  const handleGenerateCert = async () => {
    setGenerating(true);
    try {
      await generateCaCert();
      await refreshCertInfo();
    } catch {
      // Dev mode
    }
    setGenerating(false);
  };

  const handleCopyCert = async () => {
    if (certPem) {
      await navigator.clipboard.writeText(certPem);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const handleDownloadCert = () => {
    if (certPem) {
      const blob = new Blob([certPem], { type: "application/x-x509-ca-cert" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "r-burp-ca.crt";
      a.click();
      URL.revokeObjectURL(url);
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
          <h1
            className="text-xl font-gothic text-cursor-dark"
            style={{ letterSpacing: "-0.3px" }}
          >
            Settings
          </h1>
        </div>

        <div className="flex-1 overflow-y-auto p-6">
          <div className="max-w-3xl space-y-6">
            {/* Certificate Section */}
            <Card className="p-6">
              <h2
                className="text-lg font-gothic text-cursor-dark mb-4"
                style={{ letterSpacing: "-0.2px" }}
              >
                CA Certificate
              </h2>
              <p className="text-sm text-cursor-dark/55 mb-4 leading-relaxed">
                The CA certificate is used to intercept HTTPS traffic. Generate
                it once, then install it in your system trust store to enable
                HTTPS interception.
              </p>

              <div className="flex items-center gap-3 mb-4">
                <Button
                  variant="primary"
                  onClick={handleGenerateCert}
                  disabled={generating}
                >
                  {generating ? "Generating..." : certInfo?.generated ? "Regenerate" : "Generate CA Certificate"}
                </Button>
                {certInfo?.generated && (
                  <Badge color="success">Generated</Badge>
                )}
              </div>

              {certInfo?.generated && (
                <div className="space-y-3 mt-4">
                  <div className="flex items-center gap-4 text-xs text-cursor-dark/55">
                    <span>
                      Fingerprint:{" "}
                      <code className="font-mono text-cursor-dark/70">
                        {certInfo.fingerprint}
                      </code>
                    </span>
                  </div>

                  <div className="flex gap-2">
                    <Button variant="secondary-pill" onClick={handleCopyCert}>
                      {copied ? "Copied!" : "Copy PEM"}
                    </Button>
                    <Button variant="secondary-pill" onClick={handleDownloadCert}>
                      Download .crt
                    </Button>
                    <Button
                      variant="ghost"
                      onClick={() => setShowCert(!showCert)}
                    >
                      {showCert ? "Hide" : "Show"} Certificate
                    </Button>
                  </div>

                  {showCert && certPem && (
                    <Card variant="compact" className="p-3 bg-surface-100">
                      <pre className="text-xs font-mono text-cursor-dark/60 whitespace-pre-wrap break-all max-h-48 overflow-y-auto">
                        {certPem}
                      </pre>
                    </Card>
                  )}

                  {/* Installation instructions */}
                  <div className="mt-4 p-4 bg-surface-300 rounded-comfortable border border-border-primary">
                    <h3 className="text-sm font-gothic text-cursor-dark mb-2">
                      Installation Instructions
                    </h3>
                    <div className="space-y-2 text-xs text-cursor-dark/55">
                      <p>
                        <strong className="text-cursor-dark/70">macOS:</strong>{" "}
                        Double-click the .crt file → Add to "System" keychain →
                        Trust → "Always Trust"
                      </p>
                      <p>
                        <strong className="text-cursor-dark/70">Windows:</strong>{" "}
                        Double-click the .crt file → Install Certificate → Place
                        in "Trusted Root Certification Authorities"
                      </p>
                      <p>
                        <strong className="text-cursor-dark/70">Linux:</strong>{" "}
                        Copy to <code className="font-mono">/usr/local/share/ca-certificates/</code>{" "}
                        and run <code className="font-mono">sudo update-ca-certificates</code>
                      </p>
                    </div>
                  </div>
                </div>
              )}
            </Card>

            {/* Proxy Settings */}
            <Card className="p-6">
              <h2
                className="text-lg font-gothic text-cursor-dark mb-4"
                style={{ letterSpacing: "-0.2px" }}
              >
                Proxy Configuration
              </h2>
              <div className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="text-xs text-cursor-dark/55 block mb-1">
                      Default Listen Address
                    </label>
                    <div className="bg-surface-300 rounded-comfortable border border-border-primary px-3 py-2 text-sm font-mono text-cursor-dark/70">
                      127.0.0.1
                    </div>
                  </div>
                  <div>
                    <label className="text-xs text-cursor-dark/55 block mb-1">
                      Default Port
                    </label>
                    <div className="bg-surface-300 rounded-comfortable border border-border-primary px-3 py-2 text-sm font-mono text-cursor-dark/70">
                      8080
                    </div>
                  </div>
                </div>
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="text-xs text-cursor-dark/55 block mb-1">
                      Max Captured Requests
                    </label>
                    <div className="bg-surface-300 rounded-comfortable border border-border-primary px-3 py-2 text-sm font-mono text-cursor-dark/70">
                      1000
                    </div>
                  </div>
                  <div>
                    <label className="text-xs text-cursor-dark/55 block mb-1">
                      Intercept Timeout
                    </label>
                    <div className="bg-surface-300 rounded-comfortable border border-border-primary px-3 py-2 text-sm font-mono text-cursor-dark/70">
                      30s
                    </div>
                  </div>
                </div>
              </div>
            </Card>

            {/* About */}
            <Card className="p-6">
              <h2
                className="text-lg font-gothic text-cursor-dark mb-4"
                style={{ letterSpacing: "-0.2px" }}
              >
                About
              </h2>
              <div className="space-y-2 text-sm text-cursor-dark/55">
                <p>
                  <strong className="text-cursor-dark/70">r-burp</strong> v0.1.0
                </p>
                <p>
                  A lightweight, secure, privacy-oriented desktop proxy toolkit.
                </p>
                <p>
                  Built with Rust, Tauri, Next.js, and Tailwind CSS.
                </p>
                <p className="text-xs text-cursor-dark/40">
                  MIT License • Open Source
                </p>
              </div>
            </Card>
          </div>
        </div>
      </div>
    </AppLayout>
  );
}
