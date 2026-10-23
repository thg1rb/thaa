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
  revealProjectFolder,
  openProjectInTerminal,
  openProjectInEditor,
  getTrayRuntimeSnapshot,
  navigateToRuntimeEntry,
  showMainWindow,
  dismissTrayPopover,
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

function getTrayRuntimeSnapshot() {
  return invoke<{ snapshot: RuntimeSnapshotDto | null; stale: boolean }>(
    "get_tray_runtime_snapshot",
  ).then(({ snapshot, stale }) => ({
    snapshot: snapshot ? withEmptyIconAssets(snapshot) : null,
    stale,
  }));
}

function navigateToRuntimeEntry(generation: number, entryRef: string) {
  return invoke<void>("navigate_to_runtime_entry", { generation, entryRef });
}

function showMainWindow() {
  return invoke<void>("show_main_window");
}

function dismissTrayPopover() {
  return invoke<void>("dismiss_tray_popover");
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

function revealProjectFolder(entryRef: string) {
  return invoke<void>("reveal_project_folder", { entryRef });
}

function openProjectInTerminal(entryRef: string) {
  return invoke<void>("open_project_in_terminal", { entryRef });
}

function openProjectInEditor(entryRef: string) {
  return invoke<void>("open_project_in_editor", { entryRef });
}

export function snapshotFromEvent(
  snapshot: RuntimeSnapshotDto,
): RuntimeSnapshot {
  return withEmptyIconAssets(snapshot);
}
