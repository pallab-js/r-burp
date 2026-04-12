import type { InputHTMLAttributes, TextareaHTMLAttributes, ReactNode } from "react";
import clsx from "clsx";

// Validation constants
export const MAX_RULE_NAME_LENGTH = 100;
export const MAX_MATCH_PATTERN_LENGTH = 2000;
export const MAX_HEADER_VALUE_LENGTH = 10000;
export const MAX_HEADER_NAME_LENGTH = 100;
export const MAX_ACTION_TARGET_LENGTH = 200;
export const MAX_ACTION_VALUE_LENGTH = 10000;
export const MAX_BODY_LENGTH = 100_000;
export const MAX_HOST_LENGTH = 253;
export const MAX_JSON_HEADERS_LENGTH = 50_000;

export function validateRuleName(name: string): string | null {
  if (!name.trim()) return "Rule name is required";
  if (name.length > MAX_RULE_NAME_LENGTH) return `Rule name must be ${MAX_RULE_NAME_LENGTH} characters or less`;
  return null;
}

export function validateMatchPattern(pattern: string): string | null {
  if (!pattern.trim()) return "Match pattern is required";
  if (pattern.length > MAX_MATCH_PATTERN_LENGTH) return `Pattern must be ${MAX_MATCH_PATTERN_LENGTH} characters or less`;
  return null;
}

export function validateActionTarget(target: string): string | null {
  if (target.length > MAX_ACTION_TARGET_LENGTH) return `Target must be ${MAX_ACTION_TARGET_LENGTH} characters or less`;
  return null;
}

export function validateActionValue(value: string): string | null {
  if (value.length > MAX_ACTION_VALUE_LENGTH) return `Value must be ${MAX_ACTION_VALUE_LENGTH} characters or less`;
  return null;
}

export function validateBody(body: string): string | null {
  if (body.length > MAX_BODY_LENGTH) return `Body must be ${MAX_BODY_LENGTH.toLocaleString()} characters or less`;
  return null;
}

export function validateHost(host: string): string | null {
  if (!host.trim()) return "Host is required";
  if (host.length > MAX_HOST_LENGTH) return `Host must be ${MAX_HOST_LENGTH} characters or less`;
  // Only allow valid hostname characters
  if (!/^[a-zA-Z0-9._-]+$/.test(host.trim())) return "Host contains invalid characters";
  return null;
}

interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
}

export function Input({ label, error, className, ...props }: InputProps) {
  return (
    <div className="flex flex-col gap-1.5 w-full">
      {label && (
        <label className="text-sm font-gothic text-cursor-dark">
          {label}
        </label>
      )}
      <input
        id={label ? `input-${label.replace(/\s+/g, '-').toLowerCase()}` : undefined}
        aria-label={label}
        className={clsx(
          "bg-transparent text-cursor-dark",
          "border border-border-primary rounded-comfortable",
          "px-2 py-1.5",
          "focus:border-border-medium focus:outline-none",
          "transition-colors duration-150 ease",
          "placeholder:text-cursor-dark/40",
          error && "border-error",
          className
        )}
        {...props}
      />
      {error && (
        <span className="text-xs text-error">{error}</span>
      )}
    </div>
  );
}

interface TextareaProps extends TextareaHTMLAttributes<HTMLTextAreaElement> {
  label?: string;
  error?: string;
}

export function Textarea({ label, error, className, ...props }: TextareaProps) {
  return (
    <div className="flex flex-col gap-1.5 w-full">
      {label && (
        <label className="text-sm font-gothic text-cursor-dark">
          {label}
        </label>
      )}
      <textarea
        id={label ? `textarea-${label.replace(/\s+/g, '-').toLowerCase()}` : undefined}
        aria-label={label}
        className={clsx(
          "bg-transparent text-cursor-dark",
          "border border-border-primary rounded-comfortable",
          "px-2 py-[8px] pb-[6px]",
          "focus:border-border-medium focus:outline-none",
          "transition-colors duration-150 ease",
          "placeholder:text-cursor-dark/40",
          "resize-y min-h-[80px]",
          error && "border-error",
          className
        )}
        {...props}
      />
      {error && (
        <span className="text-xs text-error">{error}</span>
      )}
    </div>
  );
}
