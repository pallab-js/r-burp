import type { HTMLAttributes, ReactNode } from "react";
import clsx from "clsx";

interface BadgeProps extends HTMLAttributes<HTMLSpanElement> {
  children: ReactNode;
  color?: "default" | "thinking" | "grep" | "read" | "edit" | "success" | "error";
}

export function Badge({
  children,
  color = "default",
  className,
  ...props
}: BadgeProps) {
  const colors = {
    default: "bg-surface-400 text-cursor-dark/60",
    thinking: "bg-thinking/20 text-thinking",
    grep: "bg-grep/20 text-grep",
    read: "bg-read/20 text-read",
    edit: "bg-edit/20 text-edit",
    success: "bg-success/20 text-success",
    error: "bg-error/20 text-error",
  };

  return (
    <span
      className={clsx(
        "inline-flex items-center rounded-pill",
        "py-[3px] px-[8px]",
        "text-xs font-gothic",
        colors[color],
        className
      )}
      {...props}
    >
      {children}
    </span>
  );
}
