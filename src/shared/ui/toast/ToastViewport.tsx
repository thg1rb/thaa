import { useEffect, useRef, useState } from "react";
import type { ToastItem } from "./toastTypes";
import "./toast.css";

export function ToastViewport({
  toasts,
  onDismiss,
}: {
  toasts: ToastItem[];
  onDismiss: (id: number) => void;
}) {
  return (
    <ol className="toast-viewport" aria-label="Notifications">
      {toasts.map((toast) => (
        <ToastCard key={toast.id} toast={toast} onDismiss={onDismiss} />
      ))}
    </ol>
  );
}

function ToastCard({
  toast,
  onDismiss,
}: {
  toast: ToastItem;
  onDismiss: (id: number) => void;
}) {
  const [hovered, setHovered] = useState(false);
  const [focused, setFocused] = useState(false);
  const remainingMs = useRef(toast.durationMs);
  const paused = hovered || focused;

  useEffect(() => {
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
  }, [onDismiss, paused, toast.id]);

  return (
    <li
      className={`toast toast-${toast.tone}`}
      role={toast.tone === "error" ? "alert" : "status"}
      aria-live={toast.tone === "error" ? "assertive" : "polite"}
      onPointerEnter={() => setHovered(true)}
      onPointerLeave={() => setHovered(false)}
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
