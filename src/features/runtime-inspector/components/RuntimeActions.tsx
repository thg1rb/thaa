import type { Action, RuntimeSnapshot } from "../model/types";

export function RuntimeActions({
  targetRef,
  capabilities,
  busy,
  onAction,
  onConfirmForce,
}: {
  targetRef: string | null;
  capabilities: RuntimeSnapshot["capabilities"];
  busy: boolean;
  onAction: (targetRef: string, action: Action) => void;
  onConfirmForce: () => void;
}) {
  if (!targetRef) return null;
  return (
    <div className="process-actions">
      {capabilities.gracefulStop && (
        <button
          className="button-stop"
          disabled={busy}
          onClick={() => onAction(targetRef, "gracefulStop")}
        >
          {busy ? "Working…" : "Stop"}
        </button>
      )}
      {capabilities.forceStop && (
        <button
          className="button-force"
          disabled={busy}
          onClick={onConfirmForce}
        >
          {busy ? "Working…" : "Force stop"}
        </button>
      )}
    </div>
  );
}
