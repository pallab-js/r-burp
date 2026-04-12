"use client";

import { useRouter } from "next/navigation";
import clsx from "clsx";
import type { NavItem } from "../../types";

const routeMap: Record<string, string> = {
  dashboard: "/",
  proxy: "/proxy",
  interceptor: "/interceptor",
  requests: "/requests",
  settings: "/settings",
};

interface SidebarProps {
  items: NavItem[];
  activeItem: string;
}

export function Sidebar({ items, activeItem }: SidebarProps) {
  const router = useRouter();

  return (
    <aside className="w-56 bg-surface-400 border-r border-border-primary flex flex-col flex-shrink-0">
      {/* Logo */}
      <div className="px-4 py-4 border-b border-border-primary">
        <h1
          className="text-lg font-gothic text-cursor-dark"
          style={{ letterSpacing: "-0.5px" }}
        >
          r-burp
        </h1>
        <p className="text-xs text-cursor-dark/55 mt-0.5">v0.1.0</p>
      </div>

      {/* Navigation */}
      <nav className="flex-1 px-2 py-3 flex flex-col gap-1 overflow-y-auto">
        {items.map((item) => {
          const route = routeMap[item.id] || "/";
          const isActive = activeItem === item.id;
          return (
            <button
              key={item.id}
              onClick={() => router.push(route)}
              className={clsx(
                "flex items-center gap-2.5 px-3 py-2 rounded-comfortable text-sm",
                "font-gothic text-left cursor-pointer",
                "transition-colors duration-150 ease",
                isActive
                  ? "bg-surface-300 text-cursor-dark"
                  : "text-cursor-dark/55 hover:bg-surface-300/50 hover:text-cursor-dark/75"
              )}
            >
              {item.icon && <span className="text-base">{item.icon}</span>}
              <span>{item.label}</span>
            </button>
          );
        })}
      </nav>

      {/* Footer */}
      <div className="px-4 py-3 border-t border-border-primary">
        <p className="text-xs text-cursor-dark/40">
          Secure by Design
        </p>
      </div>
    </aside>
  );
}
