import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  getInitialSnapshot,
  openListenerUrl,
  refreshSnapshot,
  requestAction,
  type Action,
  type ActionResult,
  type RuntimeEntry,
  type RuntimeSnapshot,
} from "./runtimeApi";
import "./App.css";

type ScreenState = {
  snapshot: RuntimeSnapshot | null;
  loading: boolean;
  refreshing: boolean;
  error: string | null;
  notice: string | null;
  actionTarget: string | null;
  confirming: RuntimeEntry | null;
};

function friendlyError(error: unknown) {
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

function field(value: { state: string; value?: string }) {
  return value.state === "available" && value.value
    ? safeDisplay(value.value)
    : null;
}

function safeDisplay(value: string) {
  return Array.from(value, (character) => {
    const point = character.codePointAt(0) ?? 0;
    return point < 0x20 ||
      (point >= 0x7f && point <= 0x9f) ||
      (point >= 0x202a && point <= 0x202e) ||
      (point >= 0x2066 && point <= 0x2069)
      ? "�"
      : character;
  }).join("");
}

function newestSnapshot(
  current: RuntimeSnapshot | null,
  incoming: RuntimeSnapshot,
) {
  return current && current.generation > incoming.generation
    ? current
    : incoming;
}

function bindingLabel(binding: RuntimeEntry["binding"]) {
  if (binding === "loopbackOnly") return "Loopback only";
  if (binding === "potentiallyReachable") return "Beyond loopback";
  return "Address unknown";
}

export default function App() {
  const [state, setState] = useState<ScreenState>({
    snapshot: null,
    loading: true,
    refreshing: false,
    error: null,
    notice: null,
    actionTarget: null,
    confirming: null,
  });
  const mounted = useRef(true);
  const refreshBusy = useRef(false);
  const queuedActionRefresh = useRef<Promise<void> | null>(null);
  const resolveQueuedActionRefresh = useRef<(() => void) | null>(null);
  const cancelButton = useRef<HTMLButtonElement>(null);

  const refresh = useCallback(async (initial = false, afterCurrent = false) => {
    if (refreshBusy.current) {
      if (!afterCurrent) return;
      if (!queuedActionRefresh.current) {
        queuedActionRefresh.current = new Promise((resolve) => {
          resolveQueuedActionRefresh.current = resolve;
        });
      }
      return queuedActionRefresh.current;
    }
    refreshBusy.current = true;
    setState((current) => ({
      ...current,
      loading: initial && !current.snapshot,
      refreshing: !initial,
      error: initial && !current.snapshot ? null : current.error,
    }));
    try {
      const snapshot = initial
        ? await getInitialSnapshot()
        : await refreshSnapshot();
      if (mounted.current) {
        setState((current) => ({
          ...current,
          snapshot: newestSnapshot(current.snapshot, snapshot),
          loading: false,
          refreshing: false,
          error: null,
        }));
      }
    } catch (error) {
      if (mounted.current) {
        setState((current) => ({
          ...current,
          loading: false,
          refreshing: false,
          error: friendlyError(error),
        }));
      }
    } finally {
      refreshBusy.current = false;
      if (queuedActionRefresh.current) {
        const queued = queuedActionRefresh.current;
        const resolve = resolveQueuedActionRefresh.current;
        queuedActionRefresh.current = null;
        resolveQueuedActionRefresh.current = null;
        void refresh().then(
          () => resolve?.(),
          () => resolve?.(),
        );
        await queued;
      }
    }
  }, []);

  useEffect(() => {
    mounted.current = true;
    void refresh(true);
    let unlisten: (() => void) | undefined;
    let cancelled = false;
    void listen<RuntimeSnapshot>("runtime-snapshot-updated", (event) => {
      setState((current) => ({
        ...current,
        snapshot: newestSnapshot(current.snapshot, event.payload),
        error: null,
        refreshing: false,
      }));
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
  }, [refresh]);

  useEffect(() => {
    if (!state.confirming) return;
    const previousFocus = document.activeElement as HTMLElement | null;
    cancelButton.current?.focus();
    return () => previousFocus?.focus();
  }, [state.confirming]);

  useEffect(() => {
    const timer = window.setInterval(() => {
      if (document.visibilityState === "visible") void refresh();
    }, 10_000);
    return () => window.clearInterval(timer);
  }, [refresh]);

  const submitAction = async (entry: RuntimeEntry, action: Action) => {
    if (!entry.actionTargetRef) return;
    const targetRef = entry.actionTargetRef;
    setState((current) => ({
      ...current,
      actionTarget: targetRef,
      notice: null,
    }));
    try {
      const result: ActionResult = await requestAction(targetRef, action);
      const message = actionMessage(result);
      setState((current) => ({
        ...current,
        actionTarget: null,
        confirming: null,
        notice: message,
      }));
      if (result.state !== "failed") {
        await refresh(false, true);
      }
    } catch {
      setState((current) => ({
        ...current,
        actionTarget: null,
        confirming: null,
        notice: "The process action could not be completed.",
      }));
    }
  };

  const snapshot = state.snapshot;
  const iconSources = useMemo(
    () =>
      new Map(
        (snapshot?.processIcons ?? []).map((icon) => [
          icon.reference,
          `data:image/png;base64,${icon.pngBase64}`,
        ]),
      ),
    [snapshot],
  );
  return (
    <main className="app-shell">
      <header className="topbar">
        <div className="brand-lockup">
          <span className="brand-mark" aria-hidden="true">
            T
          </span>
          <div>
            <p className="eyebrow">LOCAL RUNTIME INSPECTOR</p>
            <h1>Thaa</h1>
          </div>
        </div>
        <div className="topbar-actions">
          {snapshot && (
            <span className="runtime-count">
              {snapshot.entries.length} listeners
            </span>
          )}
          <button
            className="refresh-button"
            onClick={() => void refresh()}
            disabled={state.refreshing || state.loading}
          >
            <span
              aria-hidden="true"
              className={
                state.refreshing ? "refresh-icon spinning" : "refresh-icon"
              }
            >
              ↻
            </span>
            {state.refreshing ? "Refreshing" : "Refresh"}
          </button>
        </div>
      </header>

      <section className="intro-row">
        <div>
          <p className="eyebrow">YOUR MACHINE</p>
          <h2>Listening now</h2>
          <p className="intro-copy">
            Local TCP services, their ports, and the processes behind them.
          </p>
        </div>
        {snapshot && (
          <p className="scan-time">
            Updated{" "}
            {new Date(snapshot.observedAtUnixMs).toLocaleTimeString([], {
              hour: "2-digit",
              minute: "2-digit",
            })}
          </p>
        )}
      </section>

      {snapshot?.completeness.state === "partial" && (
        <p className="banner banner-warn" role="status">
          Some listeners may be missing from this scan.
        </p>
      )}
      {state.error && (
        <p className="banner banner-error" role="alert">
          {state.error}
        </p>
      )}
      {state.notice && (
        <p className="banner banner-info" role="status">
          {state.notice}
        </p>
      )}

      {state.loading && !snapshot ? (
        <section className="empty-panel" aria-busy="true" aria-live="polite">
          <span className="loader" aria-hidden="true" />
          <h3>Finding local listeners</h3>
          <p>Checking active TCP ports and process details…</p>
        </section>
      ) : snapshot && snapshot.entries.length === 0 ? (
        <section className="empty-panel">
          <div className="empty-symbol" aria-hidden="true">
            ⌁
          </div>
          <h3>No listening TCP ports found</h3>
          <p>Start a local server and refresh to see it here.</p>
          <button
            className="button-secondary"
            onClick={() => void refresh()}
            disabled={state.refreshing}
          >
            Refresh list
          </button>
        </section>
      ) : snapshot ? (
        <section
          className="runtime-list"
          aria-label="Listening TCP ports"
          aria-busy={state.refreshing}
        >
          {snapshot.entries.map((entry) => (
            <RuntimeCard
              key={entry.entryRef}
              entry={entry}
              capabilities={snapshot.capabilities}
              iconSource={
                entry.processIconRef
                  ? iconSources.get(entry.processIconRef)
                  : undefined
              }
              busy={state.actionTarget === entry.actionTargetRef}
              onAction={(action) => void submitAction(entry, action)}
              onConfirmForce={() =>
                setState((current) => ({ ...current, confirming: entry }))
              }
              onOpen={() =>
                void openListenerUrl(entry.entryRef).catch(() =>
                  setState((current) => ({
                    ...current,
                    notice: "Thaa could not open this local address.",
                  })),
                )
              }
              onCopy={async (value, label) => {
                try {
                  await navigator.clipboard.writeText(value);
                  setState((current) => ({
                    ...current,
                    notice: `${label} copied to clipboard.`,
                  }));
                } catch {
                  setState((current) => ({
                    ...current,
                    notice: "Clipboard access is unavailable.",
                  }));
                }
              }}
            />
          ))}
        </section>
      ) : null}

      <footer className="app-footer">
        <span className="privacy-dot" />
        Process details stay on this device
      </footer>

      {state.confirming && (
        <div
          className="dialog-backdrop"
          role="presentation"
          onMouseDown={(event) => {
            if (event.target === event.currentTarget)
              setState((current) => ({ ...current, confirming: null }));
          }}
        >
          <section
            className="confirm-dialog"
            role="dialog"
            aria-modal="true"
            aria-labelledby="force-title"
            aria-describedby="force-description"
            onKeyDown={(event) => {
              if (event.key === "Escape") {
                event.preventDefault();
                setState((current) => ({ ...current, confirming: null }));
                return;
              }
              if (event.key !== "Tab") return;
              const controls =
                event.currentTarget.querySelectorAll<HTMLButtonElement>(
                  "button:not(:disabled)",
                );
              const first = controls[0];
              const last = controls[controls.length - 1];
              if (event.shiftKey && document.activeElement === first) {
                event.preventDefault();
                last?.focus();
              } else if (!event.shiftKey && document.activeElement === last) {
                event.preventDefault();
                first?.focus();
              }
            }}
          >
            <span className="dialog-icon" aria-hidden="true">
              !
            </span>
            <p className="eyebrow">FORCE STOP</p>
            <h2 id="force-title">Stop this process immediately?</h2>
            <p id="force-description">
              Normal cleanup may not run. Unsaved work handled by this process
              could be lost.
            </p>
            <div className="confirm-target">
              <span className="confirm-process-name">
                {displayName(state.confirming)}
              </span>
              <span>
                PID {state.confirming.processId ?? "unknown"} · Port{" "}
                {state.confirming.port}
              </span>
            </div>
            <div className="dialog-actions">
              <button
                ref={cancelButton}
                className="button-secondary"
                disabled={
                  state.actionTarget === state.confirming.actionTargetRef
                }
                onClick={() =>
                  setState((current) => ({ ...current, confirming: null }))
                }
              >
                Cancel
              </button>
              <button
                className="button-danger"
                disabled={
                  state.actionTarget === state.confirming.actionTargetRef
                }
                onClick={() =>
                  void submitAction(state.confirming!, "forceStop")
                }
              >
                {state.actionTarget === state.confirming.actionTargetRef
                  ? "Working…"
                  : "Force stop"}
              </button>
            </div>
          </section>
        </div>
      )}
    </main>
  );
}

function displayName(entry: RuntimeEntry) {
  if (entry.process.state === "available")
    return field(entry.process.details.name) ?? "Unknown process";
  return entry.process.state === "unavailable"
    ? "Process details unavailable"
    : "Unknown process";
}

function actionMessage(result: ActionResult) {
  if (result.state === "requested")
    return "Stop request sent. Thaa will confirm the change on the next scan.";
  if (result.state === "alreadyExited")
    return "This process has already exited. The list has been refreshed.";
  if (result.state === "refused") return result.reason;
  return result.reason;
}

function RuntimeCard({
  entry,
  capabilities,
  iconSource,
  busy,
  onAction,
  onConfirmForce,
  onOpen,
  onCopy,
}: {
  entry: RuntimeEntry;
  capabilities: RuntimeSnapshot["capabilities"];
  iconSource: string | undefined;
  busy: boolean;
  onAction: (action: Action) => void;
  onConfirmForce: () => void;
  onOpen: () => void;
  onCopy: (value: string, label: string) => void;
}) {
  const info =
    entry.process.state === "available" ? entry.process.details : null;
  const processName = displayName(entry);
  const executable = info ? field(info.executablePath) : null;
  const workingDirectory = info ? field(info.workingDirectory) : null;
  const pid = entry.processId;
  return (
    <article className="runtime-card">
      <div className="runtime-details">
        <div className="process-heading">
          <RuntimeIcon
            source={iconSource}
            unknown={entry.process.state === "noOwner"}
          />
          <div className="process-title">
            <p className="eyebrow">PROCESS / OWNER</p>
            <h3 title={processName}>{processName}</h3>
            <p>
              {entry.process.state === "noOwner"
                ? "Owner not identified"
                : `PID ${pid ?? "unknown"}`}
              <span className="separator">·</span>
              {entry.localAddress ?? "Address unavailable"}
            </p>
          </div>
          <div className="port-identity">
            <span className="eyebrow">TCP PORT</span>
            <strong className="port-number">:{entry.port}</strong>
          </div>
          <div className="entry-actions">
            {entry.localUrl && (
              <button
                className="icon-button"
                aria-label={`Open ${entry.localUrl}`}
                title="Open local service"
                onClick={onOpen}
              >
                ↗
              </button>
            )}
            {entry.localUrl && (
              <button
                className="icon-button"
                aria-label={`Copy URL for port ${entry.port}`}
                title="Copy URL"
                onClick={() => onCopy(entry.localUrl!, "URL")}
              >
                ⧉
              </button>
            )}
            <button
              className="icon-button copy-value"
              aria-label={`Copy port ${entry.port}`}
              title="Copy port"
              onClick={() => onCopy(String(entry.port), "Port")}
            >
              #
            </button>
            {pid !== null && (
              <button
                className="icon-button copy-value"
                aria-label={`Copy PID ${pid}`}
                title="Copy PID"
                onClick={() => onCopy(String(pid), "PID")}
              >
                ID
              </button>
            )}
          </div>
        </div>
        <div className="metadata-row">
          {executable && <span title={executable}>{executable}</span>}
          {workingDirectory && (
            <span title={workingDirectory}>in {workingDirectory}</span>
          )}
          {info && info.name.state !== "available" && (
            <span className="muted-metadata">Process name unavailable</span>
          )}
          {!info && (
            <span className="muted-metadata">
              {entry.process.state === "unavailable"
                ? safeDisplay(entry.process.details.reason)
                : "Owner not identified"}
            </span>
          )}
          {info && !executable && !workingDirectory && (
            <span className="muted-metadata">
              Additional process details unavailable
            </span>
          )}
        </div>
        <div className="card-bottom">
          <div className="binding-group">
            <span
              className={`binding-mark binding-${entry.binding}`}
              aria-hidden="true"
            />
            <span className={`binding-badge binding-${entry.binding}`}>
              {bindingLabel(entry.binding)}
            </span>
            <span className="protocol-label">LOCAL TCP</span>
          </div>
          <div className="process-actions">
            {entry.actionTargetRef && capabilities.gracefulStop && (
              <button
                className="button-stop"
                disabled={busy}
                onClick={() => onAction("gracefulStop")}
              >
                {busy ? "Working…" : "Stop"}
              </button>
            )}
            {entry.actionTargetRef && capabilities.forceStop && (
              <button
                className="button-force"
                disabled={busy}
                onClick={onConfirmForce}
              >
                {busy ? "Working…" : "Force stop"}
              </button>
            )}
          </div>
        </div>
      </div>
    </article>
  );
}

function RuntimeIcon({
  source,
  unknown,
}: {
  source: string | undefined;
  unknown: boolean;
}) {
  const [failedSource, setFailedSource] = useState<string | null>(null);
  const resolvedSource = source && source !== failedSource ? source : undefined;
  if (resolvedSource) {
    return (
      <span className="runtime-app-icon" aria-hidden="true">
        <img
          src={resolvedSource}
          alt=""
          onError={() => setFailedSource(resolvedSource)}
        />
      </span>
    );
  }
  return (
    <span
      className={`runtime-app-icon fallback-icon ${unknown ? "unknown-owner-icon" : ""}`}
      aria-hidden="true"
      data-icon-kind={unknown ? "unknown-owner" : "process-fallback"}
    >
      <svg viewBox="0 0 40 40" focusable="false">
        <path d="M12 13.5h16v13H12z" />
        <path d="M16 18.5h8M16 22.5h5M20 8v5M20 27v5M8 20h4M28 20h4" />
        {unknown && <circle cx="20" cy="20" r="14" />}
      </svg>
    </span>
  );
}
