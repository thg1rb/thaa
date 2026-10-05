import { useCallback, useRef, useState } from "react";
import { runtimeClient } from "../api/runtimeClient";
import type { Action, RefreshOutcome } from "../model/types";
import type { ToastInput } from "../../../shared/ui/toast/toastTypes";

export function useProcessAction(
  refresh: (source: "action") => Promise<RefreshOutcome>,
  showToast: (toast: ToastInput) => void,
) {
  const pendingRef = useRef(new Set<string>());
  const [pendingTargets, setPendingTargets] = useState<Set<string>>(new Set());

  const requestProcessAction = useCallback(
    async (targetRef: string, action: Action) => {
      if (!targetRef || pendingRef.current.has(targetRef)) return;
      pendingRef.current.add(targetRef);
      setPendingTargets(new Set(pendingRef.current));
      try {
        const result = await runtimeClient.requestAction(targetRef, action);
        const requestedName = action === "gracefulStop" ? "Stop" : "Force stop";
        if (result.state === "requested") {
          showToast({
            tone: "success",
            title: `${requestedName} request sent`,
            message: "Thaa will confirm the result on a later scan.",
          });
        } else if (result.state === "alreadyExited") {
          showToast({
            tone: "info",
            title: "Process already exited",
            message: "Refreshing the runtime list.",
          });
        } else if (result.state === "refused") {
          showToast({
            tone: "warning",
            title: "Action not performed",
            message: safeActionReason(result.reason),
          });
        } else {
          showToast({
            tone: "error",
            title: "Action failed",
            message: safeActionReason(result.reason),
          });
        }

        if (result.state !== "failed") {
          const refreshOutcome = await refresh("action");
          if (refreshOutcome === "failed") {
            showToast({
              tone: "warning",
              title: "Result not confirmed",
              message:
                "Thaa could not complete the follow-up scan. Refresh to check the current state.",
            });
          }
        }
      } catch {
        showToast({
          tone: "error",
          title: "Action failed",
          message: "Thaa could not complete this process action.",
        });
      } finally {
        pendingRef.current.delete(targetRef);
        setPendingTargets(new Set(pendingRef.current));
      }
    },
    [refresh, showToast],
  );

  return { pendingTargets, requestProcessAction };
}

function safeActionReason(reason: string) {
  if (reason.includes("permission"))
    return "Thaa does not have permission to perform this action.";
  if (reason.includes("changed"))
    return "The process changed since this entry was loaded. Refresh and try again.";
  if (reason.includes("identity"))
    return "Process identity could not be verified. Refresh and try again.";
  if (reason.includes("unavailable on this platform"))
    return "This action is unavailable on this platform.";
  if (reason.includes("unsupported"))
    return "This action is unavailable on this platform.";
  return "Thaa could not complete this process action.";
}
