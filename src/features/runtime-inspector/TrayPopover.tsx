import { getCurrentWindow } from "@tauri-apps/api/window";
import { useCallback, useEffect, useMemo, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { runtimeClient, snapshotFromEvent } from "./api/runtimeClient";
import type { RuntimeSnapshot, RuntimeSnapshotDto } from "./model/types";
import { filterRuntimeEntries } from "./utils/filterRuntimeEntries";
import { bindingLabel, displayName, safeDisplay } from "./utils/processDisplay";
import { runtimeLabel } from "./utils/runtimeKind";
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
          <div>
            <strong>Thaa</strong>
            <span>Runtime overview</span>
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
            className="tray-view-all"
            onClick={() => void showMainWindow()}
          >
            View all
          </button>
        </div>
      </header>

      {snapshot && (
        <section className="tray-overview" aria-label="Observed runtime counts">
          <div>
            <strong>{snapshot.entries.length}</strong>
            <span>observed endpoints</span>
          </div>
          <div>
            <strong>{identifiedProcesses}</strong>
            <span>identified processes</span>
          </div>
          <span className="tray-overview-protocol">TCP</span>
        </section>
      )}

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
          <p className="tray-state" role="status">
            Loading the current runtime snapshot…
          </p>
        ) : !snapshot ? (
          <div className="tray-state">
            <p>No accepted runtime snapshot is available.</p>
            <button onClick={() => void refresh()}>Refresh</button>
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
              return (
                <article className="tray-process" key={group.key}>
                  <div className="tray-process-heading">
                    <span className="tray-process-mark" aria-hidden="true">
                      ◉
                    </span>
                    <button
                      type="button"
                      className="tray-process-name"
                      aria-label={`Open ${group.name} in Thaa, ${pidLabel}`}
                      onClick={() => void navigate(representative.entryRef)}
                    >
                      <strong title={group.name}>{group.name}</strong>
                      <span>
                        {pidLabel}
                        {group.runtime
                          ? ` · ${runtimeLabel(group.runtime)}`
                          : ""}
                      </span>
                    </button>
                    <span className="tray-process-count">
                      {group.entries.length}
                    </span>
                  </div>
                  <div
                    className="tray-port-list"
                    aria-label={`Ports for ${group.name}`}
                  >
                    {group.entries.map((entry) => {
                      const address = entry.localAddress
                        ? safeDisplay(entry.localAddress)
                        : "Address unavailable";
                      const label = `${entry.protocol.toUpperCase()} port ${entry.port}, ${address}, ${bindingLabel(entry.binding)}`;
                      return (
                        <button
                          type="button"
                          className="tray-port-badge"
                          key={entry.entryRef}
                          title={`${label}. Open listener details in Thaa.`}
                          aria-label={`Open ${label} for ${displayName(entry)} in Thaa`}
                          onClick={() => void navigate(entry.entryRef)}
                        >
                          <span>
                            {entry.protocol.toUpperCase()} {entry.port}
                          </span>
                          <small>{address}</small>
                        </button>
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
        <button
          type="button"
          className="tray-footer-link"
          onClick={() => void showMainWindow()}
        >
          View all in Thaa
        </button>
      </footer>
    </main>
  );
}
