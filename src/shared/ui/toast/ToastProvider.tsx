import { useCallback, useMemo, useState, type ReactNode } from "react";
import { ToastViewport } from "./ToastViewport";
import { ToastContext } from "./toastContext";
import type { ToastInput, ToastItem } from "./toastTypes";

const VISIBLE_LIMIT = 3;
const QUEUE_LIMIT = 5;
const EXIT_FALLBACK_MS = 260;
let nextToastId = 1;

type Notifications = {
  visible: ToastItem[];
  queued: ToastItem[];
};

export function ToastProvider({ children }: { children: ReactNode }) {
  const [notifications, setNotifications] = useState<Notifications>({
    visible: [],
    queued: [],
  });

  const showToast = useCallback((input: ToastInput) => {
    const durationMs =
      input.tone === "success" || input.tone === "info" ? 6000 : 10000;
    const toast: ToastItem = {
      ...input,
      id: nextToastId++,
      durationMs,
      phase: "visible",
    };
    setNotifications((current) => {
      if (current.visible.length < VISIBLE_LIMIT) {
        return { ...current, visible: [...current.visible, toast] };
      }
      const boundedQueue =
        current.queued.length >= QUEUE_LIMIT
          ? current.queued.slice(1)
          : current.queued;
      return { ...current, queued: [...boundedQueue, toast] };
    });
  }, []);

  const dismissToast = useCallback((id: number) => {
    setNotifications((current) => {
      const item = current.visible.find((toast) => toast.id === id);
      if (item) {
        if (item.phase === "exiting") return current;
        return {
          ...current,
          visible: current.visible.map((toast) =>
            toast.id === id ? { ...toast, phase: "exiting" } : toast,
          ),
        };
      }
      return {
        ...current,
        queued: current.queued.filter((toast) => toast.id !== id),
      };
    });
  }, []);

  const completeToastExit = useCallback((id: number) => {
    setNotifications((current) => {
      const item = current.visible.find((toast) => toast.id === id);
      if (!item || item.phase !== "exiting") return current;
      const [promoted, ...remainingQueue] = current.queued;
      const visible = current.visible.filter((toast) => toast.id !== id);
      return {
        visible: promoted ? [...visible, promoted] : visible,
        queued: remainingQueue,
      };
    });
  }, []);

  const context = useMemo(() => ({ showToast }), [showToast]);

  return (
    <ToastContext.Provider value={context}>
      {children}
      <ToastViewport
        toasts={notifications.visible}
        onDismiss={dismissToast}
        onExited={completeToastExit}
        exitFallbackMs={EXIT_FALLBACK_MS}
      />
    </ToastContext.Provider>
  );
}
