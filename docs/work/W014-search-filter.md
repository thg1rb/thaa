# W014 — Search and Filter

## Status

Status: In progress — visual refinement underway; awaiting user acceptance

Branch: `feature/search-filter`

PR: [#34](https://github.com/thg1rb/thaa/pull/34) → `develop` (open; not merged)

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

## W014 Visual Refinement

Search remains immediate with no debounce: typing updates the controlled query,
then visible entries are derived locally. The listener collection is small,
and filtering triggers no I/O or native operation, so delaying it would add
latency without a demonstrated performance benefit.

The Search wrapper previously stopped at 560px despite its row spanning the
content column. It now fills that column. Right padding is compact when the
query is empty and reserves the Clear button's actual area only while that
button is present. The Search input remains a regular editable control.

The header uses the existing Tauri-generated `src-tauri/icons/128x128.png`
application icon as a decorative image beside the text heading. Its accepted
canonical source and 824/1024 geometry are unchanged. Menu Bar and runtime
process icons remain separate.

The document scrollbar is styled in the global stylesheet: the track uses the
application background, the rounded thumb uses the existing muted-text token,
hover uses secondary text, and active drag uses the interaction-active token.
The thumb remains distinguishable against the background. Standard scrollbar
properties cover supported engines, with WebKit selectors for WKWebView and
WebView2. macOS overlay scrollbars may hide until scrolling; when visible, the
thumb retains a usable width and contrast.

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
  target preservation, and no process-action IPC during Search changes. New
  checks cover immediate per-keystroke results and the decorative app icon.
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

The base implementation was reviewed on its earlier PR head; that review
reported no findings for FR-005 matching, derived-state integrity,
refresh/query behavior, action isolation, empty states, accessibility, and
responsive layout. This visual refinement requires its own read-only review.
PR #34 targets `develop` and remains open pending explicit user acceptance.

The base implementation checks passed: `cargo fmt --check`,
`cargo clippy --all-targets -- -D warnings`, `cargo test` (61 unit tests plus
platform integration suites), frontend tests (44 tests), `pnpm format:check`,
`pnpm lint`, `pnpm typecheck`, `pnpm build`, `pnpm audit --audit-level high`,
`pnpm docs:check`, `pnpm docs:format:check`, and `git diff --check`. RustSec
audit passed for macOS and Windows targets with the repository's two existing
allowed advisories (RUSTSEC-2024-0370 and RUSTSEC-2024-0429). The local Tauri
release binary and macOS app bundle builds passed.

PR CI run [37251383479](https://github.com/thg1rb/thaa/actions/runs/37251383479)
passed Shared Quality, macOS Native Validation, and Windows Native
Validation on the documentation-evidence update head. Confirm all three
checks remain green on the latest reviewed PR head before merge.

Manual macOS validation used the bundled app and a controlled local Python
listener on port 43001. Process-name search was checked with `PYTHON` and
surrounding whitespace; exact port `43001` returned the listener, while
partial numeric query `430` returned no match. Refresh preserved the active
query and re-filtered the returned snapshot. Keyboard editing with Cmd+A and
typing worked; Clear restored all 17 then-visible listeners without another
refresh and returned focus to Search. The no-results message was distinct from
the no-listener state. The controlled listener was stopped after validation.
No process action was invoked during Search testing. Narrow-window manual
validation was not completed because the available native UI controls did not
provide a reliable resize path; this remains a deferred manual check for user
review. Automated structural tests cover responsive-width safeguards.

Windows interactive UI validation was not performed; Windows Native
Validation passed. This is not a W014 merge blocker because the feature is
frontend-side and the native Windows validation job passed.

### Visual refinement evidence

The user had already accepted W014's functional process-name/port Search
behavior. Refinement inspection found no debounce, timeout, deferred query, or
delayed state: `RuntimeInspectorPage` updates the controlled query directly
and derives visible entries with `useMemo`. Search therefore remains immediate;
filtering is local over the small current snapshot and does no I/O.

The Search field's right-side blank area came from its 560px maximum width
while the surrounding content row was wider. The field now fills the content
column. Empty queries use compact right padding; the Clear affordance's space
is reserved only while a query is present. A macOS app run showed the empty
field, a short query, and a long query without Clear overlap or horizontal
overflow. Rapid character-by-character typing updated results immediately;
Clear restored the full current snapshot.

The temporary text `T` branding mark is replaced by the approved generated
128px Thaa application icon, rendered as a decorative, non-draggable image.
The canonical artwork and accepted 824px-on-1024px production composition are
unchanged. The rebuilt macOS app displayed the icon sharply beside the Thaa
heading. Tray and inspected-process icons are unchanged.

Scrollbar styling is centralized in `src/styles/global.css`, uses existing
background/text/interaction tokens, and leaves the vertical scrollbar usable.
In the rebuilt macOS WebView, the rounded muted thumb was visible with runtime
rows, and vertical scroll content remained present. macOS controls scrollbar
visibility with its overlay-scrollbar behavior; the application does not hide
it. No horizontal scrollbar appeared. Narrow-window manual validation remains
deferred because the available native window-resize controls failed with
`noWindowsAvailable`; no evidence is claimed for that check.

Refinement tests increased the frontend suite from 44 to 46 tests, all passing
locally. The final native UI check used the rebuilt `.app`; screenshots were
not retained because the full runtime list can expose unrelated local process
metadata. Narrow-window layout remains the only deferred macOS visual check.

Final local refinement checks passed: Rust format, Clippy with warnings denied,
all 61 Rust unit tests and the macOS integration suites; all 46 frontend tests;
frontend format, lint, typecheck, and production build; pnpm audit; documentation
link and format checks; and `git diff --check`. The Tauri release app bundle was
built and launched for the native visual checks. RustSec audits for macOS arm64
and Windows x64 completed with the same two existing allowed advisories,
RUSTSEC-2024-0370 and RUSTSEC-2024-0429. Final PR CI and refinement review are
pending on the pushed refinement head.

W014 is not merged and remains pending explicit user acceptance.

## Risks and Limitations

Search is limited to the process display/fallback name and exact numeric port.
It is intentionally in-memory and transient; it is not persisted and does not
affect tray snapshots or native discovery. No separate `PROJECT-STATUS.md`
exists, so `docs/README.md` is the current project-status summary.
