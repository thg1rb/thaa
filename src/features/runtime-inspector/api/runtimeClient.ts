import { invoke } from "@tauri-apps/api/core";
import type {
  Action,
  ActionResult,
  ProcessIconAsset,
  RuntimeSnapshot,
  RuntimeSnapshotDto,
} from "../model/types";

export const runtimeClient = {
  getInitialSnapshot,
  refreshSnapshot,
  getProcessIcons,
  requestAction,
  openListener,
};

let initialSnapshot: Promise<RuntimeSnapshot> | undefined;

function withEmptyIconAssets(snapshot: RuntimeSnapshotDto): RuntimeSnapshot {
  return { ...snapshot, processIcons: [] };
}

function getInitialSnapshot() {
  initialSnapshot ??= invoke<RuntimeSnapshotDto>("get_runtime_snapshot")
    .then(withEmptyIconAssets)
    .finally(() => {
      initialSnapshot = undefined;
    });
  return initialSnapshot;
}

function refreshSnapshot() {
  return invoke<RuntimeSnapshotDto>("refresh_runtime_snapshot").then(
    withEmptyIconAssets,
  );
}

function getProcessIcons(generation: number) {
  return invoke<ProcessIconAsset[]>("get_runtime_process_icons", {
    generation,
  });
}

function requestAction(actionTargetRef: string, action: Action) {
  return invoke<ActionResult>("request_process_action", {
    actionTargetRef,
    action,
  });
}

function openListener(entryRef: string) {
  return invoke<void>("open_listener_url", { entryRef });
}

export function snapshotFromEvent(
  snapshot: RuntimeSnapshotDto,
): RuntimeSnapshot {
  return withEmptyIconAssets(snapshot);
}
