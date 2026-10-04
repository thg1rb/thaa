import { useEffect, useRef, useState } from "react";
import type { ToastItem } from "./toastTypes";
import "./toast.css";

export function ToastViewport({
  toasts,
  onDismiss,
  onExited,
  exitFallbackMs,
}: {
  toasts: ToastItem[];
  onDismiss: (id: number) => void;
  onExited: (id: number) => void;
  exitFallbackMs: number;
}) {
  return (
    <ol
      className="toast-viewport"
      aria-label="Notifications"
      aria-relevant="additions text"
    >
      {toasts.map((toast) => (
        <ToastCard
          key={toast.id}
          toast={toast}
          onDismiss={onDismiss}
          onExited={onExited}
          exitFallbackMs={exitFallbackMs}
        />
      ))}
    </ol>
  );
}

function ToastCard({
  toast,
  onDismiss,
  onExited,
  exitFallbackMs,
}: {
  toast: ToastItem;
  onDismiss: (id: number) => void;
  onExited: (id: number) => void;
  exitFallbackMs: number;
}) {
  const [hovered, setHovered] = useState(false);
  const [focused, setFocused] = useState(false);
  const remainingMs = useRef(toast.durationMs);
  const paused = hovered || focused;

  useEffect(() => {
    if (toast.phase === "exiting") {
      const fallback = window.setTimeout(
        () => onExited(toast.id),
        exitFallbackMs,
      );
      return () => window.clearTimeout(fallback);
    }
    if (paused) return;
    const startedAt = Date.now();
    const timer = window.setTimeout(
      () => onDismiss(toast.id),
      remainingMs.current,
    );
    return () => {
      window.clearTimeout(timer);
      remainingMs.current = Math.max(
        0,
        remainingMs.current - (Date.now() - startedAt),
      );
    };
  }, [exitFallbackMs, onDismiss, onExited, paused, toast.id, toast.phase]);

  return (
    <li
      className={`toast toast-${toast.tone}${toast.phase === "exiting" ? " toast-exiting" : ""}`}
      role={toast.tone === "error" ? "alert" : "status"}
      aria-live={toast.tone === "error" ? "assertive" : "polite"}
      onPointerEnter={() => setHovered(true)}
      onPointerLeave={() => setHovered(false)}
      onAnimationEnd={(event) => {
        if (toast.phase === "exiting" && event.target === event.currentTarget) {
          onExited(toast.id);
        }
      }}
      onFocusCapture={() => setFocused(true)}
      onBlurCapture={(event) => {
        if (!event.currentTarget.contains(event.relatedTarget as Node | null))
          setFocused(false);
      }}
    >
      <span className="toast-symbol" aria-hidden="true">
        {toast.tone === "success"
          ? "✓"
          : toast.tone === "error"
            ? "!"
            : toast.tone === "warning"
              ? "△"
              : "i"}
      </span>
      <div className="toast-copy">
        <strong>{toast.title}</strong>
        <p>{toast.message}</p>
      </div>
      <button
        className="toast-close"
        type="button"
        aria-label={`Dismiss notification: ${toast.title}`}
        onClick={() => onDismiss(toast.id)}
      >
        <span aria-hidden="true">×</span>
      </button>
    </li>
  );
}
