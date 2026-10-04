import { Skeleton } from "../../../shared/ui/skeleton/Skeleton";

export function RuntimeCardSkeleton() {
  return (
    <article className="runtime-card runtime-card-skeleton" aria-hidden="true">
      <div className="runtime-details">
        <div className="process-heading">
          <Skeleton className="skeleton-icon" />
          <div className="skeleton-process-title">
            <Skeleton className="skeleton-eyebrow" />
            <Skeleton className="skeleton-process-name" />
            <Skeleton className="skeleton-process-summary" />
          </div>
          <div className="skeleton-port">
            <Skeleton className="skeleton-eyebrow" />
            <Skeleton className="skeleton-port-number" />
          </div>
          <div className="skeleton-actions">
            <Skeleton className="skeleton-action" />
            <Skeleton className="skeleton-action" />
            <Skeleton className="skeleton-action" />
          </div>
        </div>
        <div className="skeleton-metadata">
          <Skeleton className="skeleton-path" />
        </div>
        <div className="skeleton-card-bottom">
          <Skeleton className="skeleton-binding" />
          <Skeleton className="skeleton-action-wide" />
          <Skeleton className="skeleton-action-wide" />
        </div>
      </div>
    </article>
  );
}
