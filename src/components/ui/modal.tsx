import type { ReactNode, HTMLAttributes } from "react";
import clsx from "clsx";

interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  title: string;
  children: ReactNode;
}

export function Modal({ isOpen, onClose, title, children }: ModalProps) {
  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-cursor-dark/20"
      onClick={onClose}
      role="dialog"
      aria-modal="true"
      aria-labelledby="modal-title"
    >
      <div
        className="bg-surface-300 rounded-comfortable border border-border-primary shadow-card max-w-lg w-full mx-4"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between px-6 py-4 border-b border-border-primary">
          <h2
            id="modal-title"
            className="text-xl font-gothic text-cursor-dark"
            style={{ letterSpacing: "-0.11px" }}
          >
            {title}
          </h2>
          <button
            onClick={onClose}
            className="text-cursor-dark/55 hover:text-error transition-colors duration-150 ease"
            aria-label="Close modal"
          >
            ✕
          </button>
        </div>
        <div className="px-6 py-4">{children}</div>
      </div>
    </div>
  );
}
