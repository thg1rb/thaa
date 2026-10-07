import { useMemo, useState } from "react";
import { useToast } from "../../shared/ui/toast/useToast";
import { ForceStopDialog } from "./components/ForceStopDialog";
import { RuntimeHeader } from "./components/RuntimeHeader";
import { RuntimeList } from "./components/RuntimeList";
import { RuntimeSearch } from "./components/RuntimeSearch";
import { RuntimeCardSkeleton } from "./components/RuntimeCardSkeleton";
import { useProcessAction } from "./hooks/useProcessAction";
import { useRuntimeEntryActions } from "./hooks/useRuntimeEntryActions";
import { useRuntimeInspector } from "./hooks/useRuntimeInspector";
import {
  filterRuntimeEntries,
  normalizeRuntimeQuery,
} from "./utils/filterRuntimeEntries";
import type { Action, RuntimeEntry } from "./model/types";
import "./runtime-inspector.css";

export function RuntimeInspectorPage() {
  const { state, refresh } = useRuntimeInspector();
  const { showToast } = useToast();
  const { pendingTargets, requestProcessAction } = useProcessAction(
    refresh,
    showToast,
  );
  const { openListener, copyValue } = useRuntimeEntryActions(showToast);
  const [confirming, setConfirming] = useState<RuntimeEntry | null>(null);
  const [query, setQuery] = useState("");
  const snapshot = state.snapshot;
  const visibleEntries = useMemo(
    () => filterRuntimeEntries(snapshot?.entries ?? [], query),
    [snapshot?.entries, query],
  );
  const isSearching = normalizeRuntimeQuery(query).length > 0;
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

  const manualRefresh = async () => {
    const hadSnapshot = state.snapshot !== null;
    const outcome = await refresh("manual");
    if (outcome === "failed" && hadSnapshot) {
      showToast({
        tone: "error",
        title: "Refresh failed",
        message: "The last successful runtime list is still shown.",
      });
    }
  };

  const runAction = (targetRef: string, action: Action) => {
    void requestProcessAction(targetRef, action);
  };

  return (
    <main className="app-shell">
      <div
        className="window-drag-region"
        data-tauri-drag-region
        aria-hidden="true"
      />
      <RuntimeHeader
        snapshot={snapshot}
        visibleCount={visibleEntries.length}
        isSearching={isSearching}
        loading={state.loading}
        refreshing={state.refreshing}
        onRefresh={() => void manualRefresh()}
      />

      {state.loading && !snapshot && (
        <p className="sr-only" role="status">
          Finding local listeners.
        </p>
      )}

      {snapshot?.completeness.state === "partial" && (
        <p className="banner banner-warn" role="status">
          Some listeners may be missing from this scan.
        </p>
      )}
      {state.backgroundStale && (
        <p className="banner banner-warn" role="status">
          The latest background scan failed. The list may be out of date.
        </p>
      )}
      {state.error && (
        <div className="banner banner-error" role="alert">
          <span>{state.error}</span>
          <button className="inline-retry" onClick={() => void manualRefresh()}>
            Try again
          </button>
        </div>
      )}

      {state.loading && !snapshot ? (
        <section
          className="runtime-list runtime-list-skeleton"
          aria-label="Loading listening TCP ports"
          aria-busy="true"
        >
          {Array.from({ length: 3 }, (_, index) => (
            <RuntimeCardSkeleton key={index} />
          ))}
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
            onClick={() => void manualRefresh()}
            disabled={state.refreshing}
          >
            Refresh list
          </button>
        </section>
      ) : snapshot ? (
        <>
          <RuntimeSearch query={query} onQueryChange={setQuery} />
          {visibleEntries.length === 0 ? (
            <section
              className="empty-panel search-empty"
              aria-labelledby="search-empty-title"
            >
              <div className="empty-symbol" aria-hidden="true">
                ⌕
              </div>
              <h3 id="search-empty-title">No matching listeners</h3>
              <p>
                Try another process name or exact port, or clear the search.
              </p>
            </section>
          ) : (
            <RuntimeList
              snapshot={snapshot}
              query={query}
              pendingTargets={pendingTargets}
              iconSources={iconSources}
              iconsLoading={state.iconsLoading}
              onAction={runAction}
              onConfirmForce={setConfirming}
              onOpen={(entryRef) => void openListener(entryRef)}
              onCopy={(value, label) => void copyValue(value, label)}
            />
          )}
        </>
      ) : null}

      <footer className="app-footer">
        <span className="privacy-dot" />
        Process details stay on this device
        {import.meta.env.DEV && (
          <button
            className="dev-preview-link"
            onClick={() => {
              window.location.search = "?w0131=preview";
            }}
          >
            Visual preview
          </button>
        )}
      </footer>

      {confirming && (
        <ForceStopDialog
          entry={confirming}
          busy={
            confirming.actionTargetRef !== null &&
            pendingTargets.has(confirming.actionTargetRef)
          }
          onCancel={() => setConfirming(null)}
          onConfirm={() => {
            const targetRef = confirming.actionTargetRef;
            if (!targetRef) return;
            void requestProcessAction(targetRef, "forceStop").finally(() =>
              setConfirming(null),
            );
          }}
        />
      )}
    </main>
  );
}
