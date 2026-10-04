export type ToastTone = "success" | "error" | "info" | "warning";

export type ToastInput = {
  tone: ToastTone;
  title: string;
  message: string;
};

export type ToastItem = ToastInput & {
  id: number;
  durationMs: number;
};
