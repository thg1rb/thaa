import { useState } from "react";
import { Skeleton } from "../../../shared/ui/skeleton/Skeleton";

export function ProcessIcon({
  source,
  unknown,
  loading = false,
}: {
  source: string | undefined;
  unknown: boolean;
  loading?: boolean;
}) {
  const [failedSource, setFailedSource] = useState<string | null>(null);
  const resolvedSource = source && source !== failedSource ? source : undefined;
  if (loading && !unknown) {
    return (
      <span
        className="runtime-app-icon process-icon-loading"
        aria-hidden="true"
      >
        <Skeleton className="process-icon-skeleton" />
      </span>
    );
  }
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
