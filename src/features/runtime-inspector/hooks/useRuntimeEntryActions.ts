import { useCallback } from "react";
import { runtimeClient } from "../api/runtimeClient";
import type { ToastInput } from "../../../shared/ui/toast/toastTypes";

export function useRuntimeEntryActions(showToast: (toast: ToastInput) => void) {
  const openListener = useCallback(
    async (entryRef: string) => {
      try {
        await runtimeClient.openListener(entryRef);
      } catch {
        showToast({
          tone: "error",
          title: "Could not open address",
          message: "Thaa could not open this local address.",
        });
      }
    },
    [showToast],
  );

  const copyValue = useCallback(
    async (value: string, label: string) => {
      try {
        await navigator.clipboard.writeText(value);
        showToast({
          tone: "success",
          title: `${label} copied`,
          message: "Copied to clipboard.",
        });
      } catch {
        showToast({
          tone: "error",
          title: "Could not copy",
          message: "Clipboard access is unavailable.",
        });
      }
    },
    [showToast],
  );

  return { openListener, copyValue };
}
