import { createContext } from "react";
import type { ToastInput } from "./toastTypes";

export type ToastContextValue = {
  showToast: (toast: ToastInput) => void;
};

export const ToastContext = createContext<ToastContextValue | null>(null);
