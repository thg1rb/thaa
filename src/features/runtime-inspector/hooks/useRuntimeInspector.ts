import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { runtimeClient, snapshotFromEvent } from "../api/runtimeClient";
import type {
  RefreshOutcome,
  RefreshSource,
  RuntimeInspectorState,
  RuntimeSnapshot,
  RuntimeSnapshotDto,
} from "../model/types";

const initialState: RuntimeInspectorState = {
  snapshot: null,
  loading: true,
  refreshing: false,
  error: null,
  backgroundStale: false,
};

export function useRuntimeInspector() {
  const [state, setState] = useState(initialState);
  const mounted = useRef(true);
  const refreshBusy = useRef(false);
  const refreshRef = useRef<(source: RefreshSource) => Promise<RefreshOutcome>>(
    async () => "queued",
  );
  const currentSnapshot = useRef<RuntimeSnapshot | null>(null);
  const iconGeneration = useRef<number | null>(null);
  const iconRequestInFlight = useRef(false);
  const actionRefreshWaiters = useRef<Array<(result: RefreshOutcome) => void>>(
    [],
  );

  const applySnapshot = useCallback((incoming: RuntimeSnapshot) => {
    const current = currentSnapshot.current;
    const accepted =
      current && current.generation > incoming.generation ? current : incoming;
    currentSnapshot.current = accepted;
    setState((previous) => ({
      ...previous,
      snapshot: accepted,
      error: null,
      backgroundStale: false,
      refreshing: false,
    }));
    if (
      !accepted.entries.some((entry) => entry.processIconRef) ||
      iconGeneration.current === accepted.generation ||
      iconRequestInFlight.current
    ) {
      return;
    }
    iconRequestInFlight.current = true;
    iconGeneration.current = accepted.generation;
    void runtimeClient
      .getProcessIcons(accepted.generation)
      .then((processIcons) => {
        if (
          !mounted.current ||
          currentSnapshot.current?.generation !== accepted.generation
        ) {
          return;
        }
        const enriched = { ...currentSnapshot.current, processIcons };
        currentSnapshot.current = enriched;
        setState((previous) =>
          previous.snapshot?.generation === accepted.generation
            ? { ...previous, snapshot: enriched }
            : previous,
        );
      })
      .catch(() => undefined)
      .finally(() => {
        iconRequestInFlight.current = false;
        const latest = currentSnapshot.current;
        if (latest && latest.generation !== iconGeneration.current) {
          applySnapshot(latest);
        }
      });
  }, []);

  const refresh = useCallback(
    async (source: RefreshSource): Promise<RefreshOutcome> => {
      if (refreshBusy.current) {
        if (source !== "action") return "queued";
        return new Promise((resolve) =>
          actionRefreshWaiters.current.push(resolve),
        );
      }

      refreshBusy.current = true;
      const hasSnapshot = currentSnapshot.current !== null;
      setState((previous) => ({
        ...previous,
        loading: source === "initial" && !hasSnapshot,
        refreshing: source !== "initial" || hasSnapshot,
        error: source === "manual" && !hasSnapshot ? previous.error : null,
        backgroundStale: source === "manual" ? false : previous.backgroundStale,
      }));

      let outcome: RefreshOutcome = "failed";
      try {
        const snapshot =
          source === "initial"
            ? await runtimeClient.getInitialSnapshot()
            : await runtimeClient.refreshSnapshot();
        if (mounted.current) {
          applySnapshot(snapshot);
          setState((previous) => ({ ...previous, loading: false }));
        }
        outcome = "updated";
      } catch (error) {
        if (mounted.current) {
          setState((previous) => ({
            ...previous,
            loading: false,
            refreshing: false,
            error:
              source === "initial" || !hasSnapshot
                ? friendlyScanError(error)
                : previous.error,
            backgroundStale:
              source === "background" && hasSnapshot
                ? true
                : previous.backgroundStale,
          }));
        }
      } finally {
        refreshBusy.current = false;
        const waiters = actionRefreshWaiters.current;
        actionRefreshWaiters.current = [];
        if (waiters.length > 0) {
          void refreshRef.current("action").then((actionOutcome) => {
            waiters.forEach((resolve) => resolve(actionOutcome));
          });
        }
      }
      return outcome;
    },
    [applySnapshot],
  );
  refreshRef.current = refresh;

  useEffect(() => {
    mounted.current = true;
    void refreshRef.current("initial");
    let unlisten: (() => void) | undefined;
    let cancelled = false;
    void listen<RuntimeSnapshotDto>("runtime-snapshot-updated", (event) => {
      applySnapshot(snapshotFromEvent(event.payload));
    })
      .then((stop) => {
        if (cancelled) stop();
        else unlisten = stop;
      })
      .catch(() => undefined);
    return () => {
      cancelled = true;
      mounted.current = false;
      unlisten?.();
    };
  }, [applySnapshot]);

  useEffect(() => {
    const timer = window.setInterval(() => {
      if (document.visibilityState === "visible")
        void refreshRef.current("background");
    }, 10_000);
    return () => window.clearInterval(timer);
  }, []);

  return { state, refresh };
}

function friendlyScanError(error: unknown) {
  if (
    typeof error === "object" &&
    error !== null &&
    "state" in error &&
    typeof error.state === "string"
  ) {
    if (error.state === "permissionDenied")
      return "Port inspection was denied by the operating system.";
    if (error.state === "unsupported")
      return "Listening-port inspection is unavailable on this system.";
  }
  return "Thaa could not inspect listening ports. Try refreshing.";
}
