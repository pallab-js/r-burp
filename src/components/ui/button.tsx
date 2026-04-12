import type { ButtonHTMLAttributes, ReactNode } from "react";
import clsx from "clsx";

type ButtonVariant = "primary" | "secondary-pill" | "tertiary-pill" | "ghost" | "light";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  children: ReactNode;
}

export function Button({
  variant = "primary",
  children,
  className,
  ...props
}: ButtonProps) {
  const variants: Record<ButtonVariant, string> = {
    primary: clsx(
      "bg-surface-300 text-cursor-dark rounded-comfortable",
      "py-[10px] px-[14px] pl-[14px] pr-[12px]",
      "transition-colors duration-150 ease",
      "hover:text-error",
      "focus:shadow-focus",
      "outline-none"
    ),
    "secondary-pill": clsx(
      "bg-surface-400 rounded-pill",
      "text-[oklab(0.263/0.6)]",
      "py-[3px] px-[8px]",
      "transition-colors duration-150 ease",
      "hover:text-error"
    ),
    "tertiary-pill": clsx(
      "bg-surface-500 rounded-pill",
      "text-[oklab(0.263/0.6)]",
      "py-[3px] px-[8px]"
    ),
    ghost: clsx(
      "bg-[rgba(38,37,30,0.06)] text-[rgba(38,37,30,0.55)]",
      "py-[6px] px-[12px]",
      "rounded-comfortable",
      "transition-colors duration-150 ease",
      "hover:text-error"
    ),
    light: clsx(
      "bg-surface-100 text-cursor-dark/90",
      "py-0 px-[12px] pl-[12px] pr-[8px] pb-[1px]",
      "rounded-comfortable"
    ),
  };

  return (
    <button
      className={clsx(variants[variant], className)}
      {...props}
    >
      {children}
    </button>
  );
}
