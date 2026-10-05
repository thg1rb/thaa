# W014 — Search and Filter

## Status

Status: In progress

Branch: `feature/search-filter`

PR: Pending

## Objective

Implement FR-005 so users can filter currently visible TCP listener entries by
process display name or port. Search is a presentation-only operation over the
current runtime snapshot and must not modify runtime state or invoke process
actions.

## Baseline and Dependencies

W013 and W013.1 are integrated into `develop`. The current runtime snapshot is
owned by the `useRuntimeInspector` feature hook; `RuntimeList` renders entries
from that snapshot and the Tauri client has no search command. Search remains
frontend-side. The canonical requirement is frozen in `THAA-REQ-0.1`, FR-005;
W003's application/native layering is unchanged.

## Scope

- Add one labeled Search field for process display name or exact port.
- Derive visible entries from the authoritative snapshot without mutation.
- Preserve the query across refreshes and apply it to each new snapshot.
- Distinguish no listeners from listeners that do not match Search.
- Preserve original entries, action targets, icons, ordering, and existing UI behavior.
- Update FR-005 traceability, test catalog, project status entry, and this work record.

## Out of Scope

Backend/Tauri filtering, process-state changes, new dependencies, port/address/
protocol/PID filters, fuzzy or regular-expression matching, sorting, persistence,
saved searches, project/Git/runtime metadata, and all subsequent work items.

## Architecture

The page owns the query alongside the existing runtime-inspector state. A pure
feature utility derives visible entries from `snapshot.entries` and the query.
`RuntimeList` receives the original snapshot for capabilities plus the derived
entry array. Cards continue receiving the same `RuntimeEntry` objects and
opaque action references. No backend command, DTO, domain model, process
identity, or controller changes are required.

## Search Semantics

- One controlled field searches process display names and ports together.
- Comparison trims leading/trailing whitespace, collapses whitespace runs,
  and lowercases for deterministic case-insensitive substring matching.
- The visible process display name, including the existing unavailable-name
  fallback, is the name-search value. Missing metadata is safe and does not
  create process identity.
- A digits-only query in the inclusive range 0–65535 matches only a listener
  whose numeric port is equal to that value. Invalid numeric-looking queries do
  not match ports but may still match a process display name.
- Name and port matching use OR semantics. Empty/whitespace-only queries show
  the complete current listener collection.
- Filtering preserves source order and does not mutate entries. Query text is
  not rewritten while typing.
- A refresh retains the query and filters the new snapshot. Clearing Search
  immediately restores the current full snapshot without a refresh.

## UX and Accessibility

The Search control sits between the Runtime Inspector introduction and the
listener results, uses the existing Thaa design tokens, and has a programmatic
label independent of its placeholder. A keyboard-accessible Clear control
appears for non-empty text and returns focus to the input. The header shows
matching listeners out of the snapshot total while Search is active; otherwise
it retains the total count. A no-results panel is distinct from the existing
no-listeners state. Search is not announced on every keystroke. Input editing,
selection, clipboard, and keyboard behavior remain native; no-results and
runtime action semantics remain accessible.

## Process Actions and Data Integrity

Search changes only which existing cards are rendered. It does not call Stop,
Force Stop, process inspection, refresh, or any native action command. Search
text never constructs or alters a process action target. A filtered card keeps
its original validated references and capability behavior.

## Testing

- Pure helper tests cover empty/whitespace input, name substring/case/space
  normalization, missing display names, exact valid port matching, invalid and
  partial numeric input, OR behavior, stable ordering, and source immutability.
- Feature tests cover labeled input, typing, clear/focus, filtered count,
  no-results versus no-listeners, refresh with an active query, original action
  target preservation, and no process-action IPC during Search changes.
- Existing icon, Skeleton, Toast, refresh, tray, copy/open, and action tests
  remain required.
- The testing catalog maps frontend evidence to TC-005 / TC-UI-003.

## Manual Validation Plan

On macOS, test full/partial/case-insensitive process names, exact and partial
numeric ports, empty and whitespace queries, Clear, no-results, refresh while
searching, filtered-row actions, normal input editing/selection/shortcuts, and
narrow-window layout. Verify no horizontal scrolling, including with a Toast
visible. Windows Native Validation remains required; interactive Windows UI
validation is not required for this frontend-only change unless available.

## Acceptance Criteria

- FR-005 name/port filtering follows the semantics above.
- Active Search survives refresh and results remain derived from the latest
  authoritative snapshot in stable order.
- True no-listener and no-match states are distinct and recover without refresh.
- Search does not change runtime/process state or invoke native process actions.
- Existing action, icon, refresh, Toast, Skeleton, tray, and W013.1 responsive
  behavior remains intact; the input is accessible and works at narrow widths.
- Automated checks, native CI, read-only review, and manual macOS validation
  pass before user acceptance is requested.

## Validation and Implementation Evidence

The filter helper and Runtime Inspector integration are implemented on
`feature/search-filter`. The frontend suite passes (44 tests); formatting,
lint, typecheck, production frontend build, Markdown-link validation, docs
formatting, and `git diff --check` pass. `pnpm audit --audit-level high`
reports no known vulnerabilities. Local Rust formatting, Clippy, Cargo tests,
RustSec audit, and Tauri native build could not run because `cargo` is absent
from this environment; the Tauri build stopped at `cargo metadata`. Hosted
Shared Quality and macOS/Windows Native Validation, read-only review, and real
macOS manual validation remain pending. No manual macOS acceptance is claimed.

W014 remains unmerged pending those gates and explicit user acceptance.

## Risks and Limitations

Search is limited to the process display/fallback name and exact numeric port.
It is intentionally in-memory and transient; it is not persisted and does not
affect tray snapshots or native discovery. No separate `PROJECT-STATUS.md`
exists, so `docs/README.md` is the current project-status summary.
