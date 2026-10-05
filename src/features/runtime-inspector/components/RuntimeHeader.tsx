import type { RuntimeSnapshot } from "../model/types";
import { Skeleton } from "../../../shared/ui/skeleton/Skeleton";

export function RuntimeHeader({
  snapshot,
  visibleCount,
  isSearching,
  loading,
  refreshing,
  onRefresh,
}: {
  snapshot: RuntimeSnapshot | null;
  visibleCount: number;
  isSearching: boolean;
  loading: boolean;
  refreshing: boolean;
  onRefresh: () => void;
}) {
  return (
    <>
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
          {snapshot ? (
            <span className="runtime-count">
              {isSearching
                ? `${visibleCount} of ${snapshot.entries.length} listeners`
                : `${snapshot.entries.length} listeners`}
            </span>
          ) : loading ? (
            <Skeleton className="header-count-skeleton" />
          ) : null}
          <button
            className="refresh-button"
            onClick={onRefresh}
            disabled={refreshing || loading}
          >
            <span
              aria-hidden="true"
              className={refreshing ? "refresh-icon spinning" : "refresh-icon"}
            >
              ↻
            </span>
            {refreshing ? "Refreshing" : "Refresh"}
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
        {snapshot ? (
          <p className="scan-time">
            Updated{" "}
            {new Date(snapshot.observedAtUnixMs).toLocaleTimeString([], {
              hour: "2-digit",
              minute: "2-digit",
            })}
          </p>
        ) : loading ? (
          <Skeleton className="header-time-skeleton" />
        ) : null}
      </section>
    </>
  );
}
