import type { HTMLAttributes, ReactNode } from "react";
import clsx from "clsx";

interface CardProps extends HTMLAttributes<HTMLDivElement> {
  children: ReactNode;
  variant?: "default" | "elevated" | "compact";
}

export function Card({
  children,
  variant = "default",
  className,
  ...props
}: CardProps) {
  const variants = {
    default: clsx(
      "bg-surface-400 rounded-comfortable border border-border-primary"
    ),
    elevated: clsx(
      "bg-surface-400 rounded-comfortable border border-border-primary shadow-card"
    ),
    compact: clsx(
      "bg-surface-400 rounded-standard border border-border-primary"
    ),
  };

  return (
    <div className={clsx(variants[variant], className)} {...props}>
      {children}
    </div>
  );
}
