import clsx from "clsx";
import { Badge } from "../ui/badge";

interface StatusBarProps {
  status: "running" | "stopped" | "error";
  listeners?: number;
  intercepted?: number;
}

export function StatusBar({ status, listeners = 0, intercepted = 0 }: StatusBarProps) {
  const statusConfig = {
    running: { color: "success" as const, label: "Running" },
    stopped: { color: "default" as const, label: "Stopped" },
    error: { color: "error" as const, label: "Error" },
  };

  const config = statusConfig[status];

  return (
    <div className="h-8 bg-surface-500 border-t border-border-primary flex items-center px-4 gap-4 text-xs">
      <Badge color={config.color}>{config.label}</Badge>
      <span className="text-cursor-dark/55">
        Listeners: <span className="text-cursor-dark">{listeners}</span>
      </span>
      <span className="text-cursor-dark/55">
        Intercepted: <span className="text-cursor-dark">{intercepted}</span>
      </span>
      <div className="flex-1" />
      <span className="text-cursor-dark/40 font-mono">
        {new Date().toLocaleTimeString()}
      </span>
    </div>
  );
}
