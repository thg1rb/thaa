import { useEffect, useRef } from "react";
import type { RuntimeEntry } from "../model/types";
import { displayName } from "../utils/processDisplay";

export function ForceStopDialog({
  entry,
  busy,
  onCancel,
  onConfirm,
}: {
  entry: RuntimeEntry;
  busy: boolean;
  onCancel: () => void;
  onConfirm: () => void;
}) {
  const cancelButton = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    const previousFocus = document.activeElement as HTMLElement | null;
    cancelButton.current?.focus();
    return () => previousFocus?.focus();
  }, []);

  return (
    <div
      className="dialog-backdrop"
      role="presentation"
      onMouseDown={(event) => {
        if (!busy && event.target === event.currentTarget) onCancel();
      }}
    >
      <section
        className="confirm-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="force-title"
        aria-describedby="force-description"
        onKeyDown={(event) => {
          if (event.key === "Escape" && !busy) {
            event.preventDefault();
            onCancel();
            return;
          }
          if (event.key !== "Tab") return;
          const controls =
            event.currentTarget.querySelectorAll<HTMLButtonElement>(
              "button:not(:disabled)",
            );
          const first = controls[0];
          const last = controls[controls.length - 1];
          if (event.shiftKey && document.activeElement === first) {
            event.preventDefault();
            last?.focus();
          } else if (!event.shiftKey && document.activeElement === last) {
            event.preventDefault();
            first?.focus();
          }
        }}
      >
        <span className="dialog-icon" aria-hidden="true">
          !
        </span>
        <p className="eyebrow">FORCE STOP</p>
        <h2 id="force-title">Stop this process immediately?</h2>
        <p id="force-description">
          Normal cleanup may not run. Unsaved work handled by this process could
          be lost.
        </p>
        <div className="confirm-target">
          <span className="confirm-process-name">{displayName(entry)}</span>
          <span>
            PID {entry.processId ?? "unknown"} · Port {entry.port}
          </span>
        </div>
        <div className="dialog-actions">
          <button
            ref={cancelButton}
            className="button-secondary"
            disabled={busy}
            onClick={onCancel}
          >
            Cancel
          </button>
          <button className="button-danger" disabled={busy} onClick={onConfirm}>
            {busy ? "Working…" : "Force stop"}
          </button>
        </div>
      </section>
    </div>
  );
}
