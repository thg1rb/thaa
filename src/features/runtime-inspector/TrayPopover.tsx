import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { Skeleton } from "../../shared/ui/skeleton/Skeleton";
import { BindingBadge } from "./components/BindingBadge";
import { ProcessIcon } from "./components/ProcessIcon";
import { runtimeClient, snapshotFromEvent } from "./api/runtimeClient";
import type {
  ProcessIconAsset,
  RuntimeSnapshot,
  RuntimeSnapshotDto,
} from "./model/types";
import { filterRuntimeEntries } from "./utils/filterRuntimeEntries";
import { bindingLabel, displayName, safeDisplay } from "./utils/processDisplay";
import { groupTrayEntries } from "./utils/trayGroups";
import appIcon from "../../../src-tauri/icons/128x128.png";
import "./tray-popover.css";

export function TrayPopover() {
  const [snapshot, setSnapshot] = useState<RuntimeSnapshot | null>(null);
  const acceptedSnapshot = useRef<RuntimeSnapshot | null>(null);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [snapshotStale, setSnapshotStale] = useState(false);
  const [query, setQuery] = useState("");
  const [expandedProcesses, setExpandedProcesses] = useState<Set<string>>(
    () => new Set(),
  );
  const [selectedBindingPorts, setSelectedBindingPorts] = useState<
    Record<string, string>
  >({});
  const [iconAssets, setIconAssets] = useState<{
    generation: number;
    assets: ProcessIconAsset[];
  } | null>(null);

  const applySnapshot = useCallback(
    (incoming: RuntimeSnapshot, isFresh = true) => {
      if (
        acceptedSnapshot.current &&
        acceptedSnapshot.current.generation > incoming.generation
      ) {
        return;
      }
      acceptedSnapshot.current = incoming;
      setSnapshot(incoming);
      const bindingPortsByProcess = new Map<string, Set<string>>();
      for (const group of groupTrayEntries(incoming.entries)) {
        const endpointCounts = new Map<string, number>();
        for (const entry of group.entries) {
          const key = `${entry.protocol}:${entry.port}`;
          endpointCounts.set(key, (endpointCounts.get(key) ?? 0) + 1);
        }
        bindingPortsByProcess.set(
          group.key,
          new Set(
            [...endpointCounts]
              .filter(([, count]) => count > 1)
              .map(([key]) => key),
          ),
        );
      }
      setExpandedProcesses((current) => {
        const next = new Set(
          [...current].filter((key) => bindingPortsByProcess.has(key)),
        );
        return next.size === current.size ? current : next;
      });
      setSelectedBindingPorts((current) => {
        const next = Object.fromEntries(
          Object.entries(current).filter(([processKey, portKey]) =>
            bindingPortsByProcess.get(processKey)?.has(portKey),
          ),
        );
        return Object.keys(next).length === Object.keys(current).length
          ? current
          : next;
      });
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
  const processIconRefs = useMemo(
    () => [
      ...new Set(
        (snapshot?.entries ?? []).flatMap((entry) =>
          entry.processIconRef ? [entry.processIconRef] : [],
        ),
      ),
    ],
    [snapshot?.entries],
  );
  useEffect(() => {
    let cancelled = false;
    const generation = snapshot?.generation;
    if (generation === undefined || processIconRefs.length === 0) return;
    void runtimeClient
      .getProcessIcons(generation)
      .then((assets) => {
        if (!cancelled) setIconAssets({ generation, assets });
      })
      .catch(() => {
        // A missing optional icon must not make the runtime list unavailable.
        if (!cancelled) setIconAssets({ generation, assets: [] });
      });
    return () => {
      cancelled = true;
    };
  }, [snapshot?.generation, processIconRefs]);

  const iconSourceByReference = useMemo(() => {
    const assets =
      iconAssets !== null && iconAssets.generation === snapshot?.generation
        ? iconAssets.assets
        : [];
    return new Map(
      assets.map((asset) => [
        asset.reference,
        `data:image/png;base64,${asset.pngBase64}`,
      ]),
    );
  }, [iconAssets, snapshot?.generation]);
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
        void runtimeClient.dismissTrayPopover().catch(() => {
          setError("Thaa could not close the Tray overview.");
        });
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [setError]);

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
              <div
                className="runtime-card tray-process tray-skeleton-row"
                aria-hidden="true"
                key={item}
              >
                <div className="tray-skeleton-heading">
                  <Skeleton className="tray-skeleton-icon" />
                  <div>
                    <Skeleton className="tray-skeleton-name" />
                    <Skeleton className="tray-skeleton-meta" />
                  </div>
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
              const allPorts = [...ports.entries()];
              const processExpanded = expandedProcesses.has(group.key);
              const visiblePorts = processExpanded
                ? allPorts
                : allPorts.slice(0, 3);
              const hiddenPortCount = allPorts.length - visiblePorts.length;
              const selectedPortKey = selectedBindingPorts[group.key];
              const selectedPort = visiblePorts.find(
                ([portKey]) => portKey === selectedPortKey,
              );
              const iconRef = representative.processIconRef ?? undefined;
              return (
                <article
                  className="runtime-card tray-process"
                  key={group.key}
                  onClick={(event) => {
                    const target = event.target;
                    if (
                      target instanceof Element &&
                      target.closest(
                        "button, a, input, select, textarea, summary, [role='button'], [role='link']",
                      )
                    ) {
                      return;
                    }
                    void navigate(representative.entryRef);
                  }}
                >
                  <div className="tray-process-heading">
                    <span className="tray-process-icon-slot">
                      <ProcessIcon
                        source={
                          iconRef
                            ? iconSourceByReference.get(iconRef)
                            : undefined
                        }
                        unknown={representative.process.state === "noOwner"}
                        compact
                      />
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
                        <span aria-hidden="true"> · </span>
                        {group.entries.length} TCP{" "}
                        {group.entries.length === 1 ? "endpoint" : "endpoints"}
                      </span>
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
                    {visiblePorts.map(([portKey, entries]) => {
                      const firstEntry = entries[0];
                      if (!firstEntry) return null;
                      const expanded = selectedPortKey === portKey;
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
                                aria-controls={
                                  expanded
                                    ? `tray-bindings-${group.key}-${portKey}`
                                    : undefined
                                }
                                aria-label={`${expanded ? "Hide" : "Show"} bindings for ${firstEntry.protocol.toUpperCase()} port ${firstEntry.port}`}
                                onClick={() =>
                                  setSelectedBindingPorts((current) => {
                                    const next = { ...current };
                                    if (next[group.key] === portKey)
                                      delete next[group.key];
                                    else next[group.key] = portKey;
                                    return next;
                                  })
                                }
                              >
                                {expanded ? "Hide bindings" : "Show bindings"}
                              </button>
                            )}
                          </div>
                        </div>
                      );
                    })}
                    {(allPorts.length > 3 || processExpanded) && (
                      <button
                        type="button"
                        className="tray-more-ports"
                        aria-expanded={processExpanded}
                        aria-label={
                          processExpanded
                            ? `Show fewer ports for ${group.name}`
                            : `Show ${hiddenPortCount} more ports for ${group.name}`
                        }
                        onClick={() => {
                          setExpandedProcesses((current) => {
                            const next = new Set(current);
                            if (processExpanded) next.delete(group.key);
                            else next.add(group.key);
                            return next;
                          });
                          if (processExpanded) {
                            const selected = selectedBindingPorts[group.key];
                            if (
                              selected &&
                              !allPorts
                                .slice(0, 3)
                                .some(([key]) => key === selected)
                            ) {
                              setSelectedBindingPorts((current) => {
                                const next = { ...current };
                                delete next[group.key];
                                return next;
                              });
                            }
                          }
                        }}
                      >
                        {processExpanded
                          ? "Show less"
                          : `+${hiddenPortCount} more`}
                      </button>
                    )}
                  </div>
                  {selectedPort && selectedPort[1].length > 1 && (
                    <section
                      id={`tray-bindings-${group.key}-${selectedPort[0]}`}
                      className="tray-bindings-panel"
                      aria-label={`${selectedPort[1][0]?.protocol.toUpperCase()} port ${selectedPort[1][0]?.port} bindings`}
                    >
                      <h4>
                        {selectedPort[1][0]?.protocol.toUpperCase()} port{" "}
                        <span>:{selectedPort[1][0]?.port}</span>
                        <small>{selectedPort[1].length} bindings</small>
                      </h4>
                      <div className="tray-binding-list">
                        {selectedPort[1].map((entry) => {
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
                              onClick={() => void navigate(entry.entryRef)}
                            >
                              <code>{address}</code>
                              <BindingBadge
                                binding={entry.binding}
                                localAddress={entry.localAddress}
                                showProtocol={false}
                              />
                            </button>
                          );
                        })}
                      </div>
                    </section>
                  )}
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
