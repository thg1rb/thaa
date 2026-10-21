import { getCurrentWindow } from "@tauri-apps/api/window";
import { useCallback, useEffect, useMemo, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { Skeleton } from "../../shared/ui/skeleton/Skeleton";
import { runtimeClient, snapshotFromEvent } from "./api/runtimeClient";
import type { RuntimeSnapshot, RuntimeSnapshotDto } from "./model/types";
import { filterRuntimeEntries } from "./utils/filterRuntimeEntries";
import { bindingLabel, displayName, safeDisplay } from "./utils/processDisplay";
import { groupTrayEntries } from "./utils/trayGroups";
import appIcon from "../../../src-tauri/icons/128x128.png";
import "./tray-popover.css";

export function TrayPopover() {
  const [snapshot, setSnapshot] = useState<RuntimeSnapshot | null>(null);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [snapshotStale, setSnapshotStale] = useState(false);
  const [query, setQuery] = useState("");
  const [expandedPorts, setExpandedPorts] = useState<Set<string>>(
    () => new Set(),
  );

  const applySnapshot = useCallback(
    (incoming: RuntimeSnapshot, isFresh = true) => {
      setSnapshot((current) =>
        current && current.generation > incoming.generation
          ? current
          : incoming,
      );
      if (isFresh) setSnapshotStale(false);
      setError(null);
      setLoading(false);
    },
    [],
  );

  useEffect(() => {
    let cancelled = false;
    const unlisten: Array<() => void> = [];

    void (async () => {
      try {
        const stopUpdated = await listen<RuntimeSnapshotDto>(
          "runtime-snapshot-updated",
          (event) => applySnapshot(snapshotFromEvent(event.payload)),
        );
        if (cancelled) stopUpdated();
        else unlisten.push(stopUpdated);
        const stopStale = await listen<boolean>(
          "runtime-snapshot-stale",
          () => {
            setSnapshotStale(true);
            setError("The latest refresh failed.");
          },
        );
        if (cancelled) stopStale();
        else unlisten.push(stopStale);
      } catch {
        // The snapshot read can still render a useful view if event setup fails.
      }

      try {
        const initial = await runtimeClient.getTrayRuntimeSnapshot();
        if (cancelled) return;
        if (initial.snapshot)
          applySnapshot(snapshotFromEvent(initial.snapshot), false);
        if (initial.stale) {
          setSnapshotStale(true);
          setError("The latest refresh failed.");
        }
        setLoading(false);
      } catch {
        if (!cancelled) {
          setError("Runtime data is unavailable. Try Refresh.");
          setLoading(false);
        }
      }
    })();

    return () => {
      cancelled = true;
      unlisten.forEach((stop) => stop());
    };
  }, [applySnapshot]);

  const visibleEntries = useMemo(
    () => filterRuntimeEntries(snapshot?.entries ?? [], query),
    [snapshot?.entries, query],
  );
  const groups = useMemo(
    () => groupTrayEntries(visibleEntries),
    [visibleEntries],
  );
  const allGroups = useMemo(
    () => groupTrayEntries(snapshot?.entries ?? []),
    [snapshot?.entries],
  );
  const identifiedProcesses = useMemo(
    () => allGroups.filter((group) => group.identityKnown).length,
    [allGroups],
  );

  const refresh = async () => {
    if (refreshing) return;
    setRefreshing(true);
    setError(null);
    try {
      applySnapshot(snapshotFromEvent(await runtimeClient.refreshSnapshot()));
    } catch {
      if (snapshot) setSnapshotStale(true);
      setError(
        snapshot
          ? "Refresh failed. Showing the last successful snapshot."
          : "Refresh failed. Try again.",
      );
    } finally {
      setRefreshing(false);
    }
  };

  const navigate = async (entryRef: string) => {
    if (!snapshot) return;
    try {
      await runtimeClient.navigateToRuntimeEntry(snapshot.generation, entryRef);
    } catch {
      setError(
        "This listener changed. Refresh the runtime list and try again.",
      );
    }
  };

  const showMainWindow = async () => {
    try {
      await runtimeClient.showMainWindow();
    } catch {
      setError("Thaa could not open its main window.");
    }
  };

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        void getCurrentWindow().hide();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

  return (
    <main className="tray-popover" aria-label="Thaa runtime overview">
      <header className="tray-header">
        <div className="tray-brand">
          <img src={appIcon} alt="" aria-hidden="true" />
          <div className="tray-brand-copy">
            <strong>Thaa</strong>
            {snapshot ? (
              <span>
                {snapshot.entries.length} TCP{" "}
                {snapshot.entries.length === 1 ? "endpoint" : "endpoints"} ·{" "}
                {identifiedProcesses} identified{" "}
                {identifiedProcesses === 1 ? "process" : "processes"}
              </span>
            ) : loading ? (
              <Skeleton className="tray-summary-skeleton" />
            ) : (
              <span>Runtime overview</span>
            )}
          </div>
        </div>
        <div className="tray-header-actions">
          <button
            type="button"
            className="tray-icon-action"
            aria-label={refreshing ? "Refreshing runtimes" : "Refresh runtimes"}
            title="Refresh runtimes"
            onClick={() => void refresh()}
            disabled={refreshing}
          >
            <span
              className={refreshing ? "tray-refresh spinning" : "tray-refresh"}
              aria-hidden="true"
            >
              ↻
            </span>
          </button>
          <button
            type="button"
            className="tray-open-action"
            onClick={() => void showMainWindow()}
          >
            Open Thaa
          </button>
        </div>
      </header>

      <label className="tray-search">
        <span className="sr-only">
          Search listeners by process or exact port
        </span>
        <span aria-hidden="true">⌕</span>
        <input
          type="search"
          value={query}
          onChange={(event) => setQuery(event.currentTarget.value)}
          placeholder="Search process or port"
          autoComplete="off"
          spellCheck={false}
        />
        {query.length > 0 && (
          <button
            type="button"
            aria-label="Clear search"
            onClick={() => setQuery("")}
          >
            ×
          </button>
        )}
      </label>

      <section
        className="tray-results"
        aria-label="Listening processes"
        aria-busy={loading || refreshing}
      >
        {loading && !snapshot ? (
          <div
            className="tray-skeleton-list"
            role="status"
            aria-label="Loading runtime snapshot"
          >
            <span className="sr-only">
              Loading the current runtime snapshot
            </span>
            {[0, 1, 2].map((item) => (
              <div className="tray-skeleton-row" aria-hidden="true" key={item}>
                <div className="tray-skeleton-heading">
                  <Skeleton className="tray-skeleton-name" />
                  <Skeleton className="tray-skeleton-meta" />
                </div>
                <div className="tray-skeleton-ports">
                  <Skeleton className="tray-skeleton-port" />
                  {item !== 1 && <Skeleton className="tray-skeleton-port" />}
                </div>
              </div>
            ))}
          </div>
        ) : !snapshot ? (
          <div className="tray-state">
            <p>No accepted runtime snapshot is available.</p>
          </div>
        ) : snapshot.entries.length === 0 && query.trim().length === 0 ? (
          <p className="tray-state">
            No listeners were observed in the current snapshot.
          </p>
        ) : visibleEntries.length === 0 ? (
          <p className="tray-state">No matching listeners.</p>
        ) : (
          <div className="tray-process-list">
            {groups.map((group) => {
              const representative = group.entries[0];
              if (!representative) return null;
              const pidLabel =
                group.identityKnown && group.processId !== null
                  ? `PID ${group.processId}`
                  : "Process identity unavailable";
              const ports = new Map<string, typeof group.entries>();
              for (const entry of group.entries) {
                const portKey = `${entry.protocol}:${entry.port}`;
                const portEntries = ports.get(portKey) ?? [];
                portEntries.push(entry);
                ports.set(portKey, portEntries);
              }
              return (
                <article className="tray-process" key={group.key}>
                  <div className="tray-process-heading">
                    <button
                      type="button"
                      className="tray-process-name"
                      aria-label={`Open ${group.name} in Thaa, ${pidLabel}`}
                      onClick={() => void navigate(representative.entryRef)}
                    >
                      <strong title={group.name}>{group.name}</strong>
                      <span>{pidLabel}</span>
                    </button>
                    <span className="tray-process-open" aria-hidden="true">
                      ›
                    </span>
                  </div>
                  <div
                    className="tray-port-list"
                    role="group"
                    aria-label={`Ports for ${group.name}`}
                  >
                    {[...ports.entries()].map(([portKey, entries]) => {
                      const firstEntry = entries[0];
                      if (!firstEntry) return null;
                      const toggleKey = `${group.key}:${portKey}`;
                      const expanded = expandedPorts.has(toggleKey);
                      return (
                        <div className="tray-port-group" key={portKey}>
                          <div className="tray-port-controls">
                            <button
                              type="button"
                              className="tray-port-chip"
                              title={`Open ${firstEntry.protocol.toUpperCase()} port ${firstEntry.port} details in Thaa.`}
                              aria-label={`Open ${firstEntry.protocol.toUpperCase()} port ${firstEntry.port} for ${group.name} in Thaa`}
                              onClick={() => void navigate(firstEntry.entryRef)}
                            >
                              :{firstEntry.port}
                            </button>
                            {entries.length > 1 && (
                              <button
                                type="button"
                                className="tray-port-disclosure"
                                aria-expanded={expanded}
                                aria-label={`${expanded ? "Hide" : "Show"} ${entries.length} bindings for ${firstEntry.protocol.toUpperCase()} port ${firstEntry.port}`}
                                onClick={() =>
                                  setExpandedPorts((current) => {
                                    const next = new Set(current);
                                    if (next.has(toggleKey))
                                      next.delete(toggleKey);
                                    else next.add(toggleKey);
                                    return next;
                                  })
                                }
                              >
                                {entries.length} bindings
                              </button>
                            )}
                          </div>
                          {expanded && (
                            <div
                              className="tray-binding-list"
                              role="group"
                              aria-label={`${firstEntry.protocol.toUpperCase()} port ${firstEntry.port} bindings`}
                            >
                              {entries.map((entry) => {
                                const address = entry.localAddress
                                  ? safeDisplay(entry.localAddress)
                                  : "Address unavailable";
                                const label = `${entry.protocol.toUpperCase()} port ${entry.port}, ${address}, ${bindingLabel(entry.binding)}`;
                                return (
                                  <button
                                    type="button"
                                    className="tray-binding-link"
                                    key={entry.entryRef}
                                    aria-label={`Open ${label} for ${displayName(entry)} in Thaa`}
                                    onClick={() =>
                                      void navigate(entry.entryRef)
                                    }
                                  >
                                    <span>{address}</span>
                                    <small>{bindingLabel(entry.binding)}</small>
                                  </button>
                                );
                              })}
                            </div>
                          )}
                        </div>
                      );
                    })}
                  </div>
                </article>
              );
            })}
          </div>
        )}
      </section>

      <footer className="tray-footer">
        <div className="tray-snapshot-status" aria-live="polite">
          {error && <span className="tray-error">{error}</span>}
          {snapshotStale && snapshot && (
            <span className="tray-stale">
              Showing the last accepted snapshot.
            </span>
          )}
          {refreshing && <span role="status">Refreshing…</span>}
          {snapshot ? (
            <>
              {snapshot.completeness.state === "partial" && (
                <span className="tray-partial">Partial scan</span>
              )}
              <span>
                Updated{" "}
                {new Date(snapshot.observedAtUnixMs).toLocaleTimeString([], {
                  hour: "2-digit",
                  minute: "2-digit",
                })}
              </span>
            </>
          ) : !error ? (
            <span>Snapshot unavailable</span>
          ) : null}
        </div>
      </footer>
    </main>
  );
}
