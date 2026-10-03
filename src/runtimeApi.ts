import { invoke } from "@tauri-apps/api/core";

export type Field<T> =
  { state: "available"; value: T } | { state: "unavailable"; value: string };
export type ProcessInfo = {
  processId: number;
  name: Field<string>;
  executablePath: Field<string>;
  startTimeUnixMs: Field<number>;
  commandArguments: Field<string[]>;
  workingDirectory: Field<string>;
};
export type ProcessDetails =
  | { state: "noOwner" }
  | { state: "available"; details: ProcessInfo }
  | { state: "unavailable"; details: { processId: number; reason: string } };
export type RuntimeEntry = {
  entryRef: string;
  protocol: "tcp";
  localAddress: string | null;
  port: number;
  binding: "loopbackOnly" | "potentiallyReachable" | "unknown";
  processId: number | null;
  process: ProcessDetails;
  localUrl: string | null;
  actionTargetRef: string | null;
};
export type RuntimeSnapshot = {
  generation: number;
  observedAtUnixMs: number;
  completeness: { state: "complete" } | { state: "partial"; reason: string };
  capabilities: { gracefulStop: boolean; forceStop: boolean };
  entries: RuntimeEntry[];
};
export type RuntimeError = { state: string; message?: string };
export type Action = "gracefulStop" | "forceStop";
export type ActionResult =
  | { state: "requested" }
  | { state: "alreadyExited" }
  | { state: "refused"; reason: string }
  | { state: "failed"; reason: string };

let initialSnapshot: Promise<RuntimeSnapshot> | undefined;

export function getInitialSnapshot() {
  initialSnapshot ??= invoke<RuntimeSnapshot>("get_runtime_snapshot").finally(
    () => {
      initialSnapshot = undefined;
    },
  );
  return initialSnapshot;
}

export function refreshSnapshot() {
  return invoke<RuntimeSnapshot>("refresh_runtime_snapshot");
}

export function requestAction(actionTargetRef: string, action: Action) {
  return invoke<ActionResult>("request_process_action", {
    actionTargetRef,
    action,
  });
}

export function openListenerUrl(entryRef: string) {
  return invoke<void>("open_listener_url", { entryRef });
}
