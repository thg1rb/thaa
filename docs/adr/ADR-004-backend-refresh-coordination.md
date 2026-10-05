# ADR-004: Backend Refresh Coordination

## Status

Accepted.

## Context

Manual and automatic refresh must not overlap, stale results must not replace newer snapshots, and the main window and tray must not start duplicate scanners. These are frozen FR-004 and NFR-006 constraints.

## Decision

Use one application-level backend `RefreshCoordinator` as the owner of in-flight work, canonical snapshot generation, pending refresh intent, and cancellation where practical. Requests from all UI surfaces share it. Overlapping requests coalesce; at most one provider scan runs at a time; only the latest accepted generation may commit.

## Alternatives Considered

- Frontend-only serialization: rejected because it cannot reliably coordinate multiple windows/tray consumers and is not a trust boundary.
- Provider-local locks only: rejected because separate providers do not coordinate complete snapshots or consumers.
- Independent scanner per view: rejected because it duplicates system calls and permits inconsistent snapshots.

## Consequences

Application tests can verify serialization and stale-result rejection with a controlled fake provider. Cancellation is best-effort and a new scan waits until prior provider work has settled. Exact async primitives, cadence, and retry policy are deferred until the application runtime and measured scan costs are known.
