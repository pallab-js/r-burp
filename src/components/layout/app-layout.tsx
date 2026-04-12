"use client";

import type { ReactNode } from "react";
import { Sidebar } from "./sidebar";
import type { NavItem } from "../../types";

interface AppLayoutProps {
  children: ReactNode;
  navItems: NavItem[];
  activeItem: string;
}

export function AppLayout({
  children,
  navItems,
  activeItem,
}: AppLayoutProps) {
  return (
    <div className="flex h-screen w-full bg-cream text-cursor-dark overflow-hidden">
      <Sidebar
        items={navItems}
        activeItem={activeItem}
      />
      <main className="flex-1 overflow-y-auto">
        {children}
      </main>
    </div>
  );
}
