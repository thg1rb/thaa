# W027 — Interactive Tray Redesign

Status: In progress
Requirement: FR-010
Priority: P1 usability follow-up
Type: Feature
Branch: `feature/w027-interactive-tray`

## Objective

Make the system tray a useful, discoverable way to inspect the current local
listener snapshot without opening the full Runtime Inspector. A primary click
opens or dismisses a compact interactive view; the native context menu remains
available for Show, Refresh, and Quit.

## Background and dependencies

W013 provides the existing tray icon, runtime snapshot, and native context
menu. The prior menu repeated one item per endpoint and did not support search
or process navigation. W014 defines local search as case-insensitive process
name matching or exact numeric port matching. W018 provides process-tree
context, W019 provides factual bind-scope labels, and W026 provides desired
TCP-port assistance in the Main Window. W027 does not change these behaviors.

The Runtime Inspector owns the accepted snapshot. The tray view reads that
snapshot and subscribes to its existing update event. Opening the view does not
start a scan. A user-requested refresh uses the existing refresh command and
publishes the accepted result to both windows. The tray sends only snapshot
generation and opaque entry reference when asking the Main Window to navigate.
The shared runtime state also reports whether the most recent refresh failed,
so a popover opened after that failure can label its retained snapshot as stale.

## Scope

- Replace repeated runtime rows in the native menu with a compact interactive
  WebView popover.
- Open and dismiss the popover on a primary tray-icon click on macOS and
  Windows; preserve the native context menu for right-click and system
  commands.
- Show observed endpoint and identity-known process counts, local search,
  process-grouped endpoint badges, snapshot age/completeness, explicit refresh,
  and View All.
- Group only entries with matching PID and available process start time;
  preserve each endpoint and keep entries with unknown identity separate.
- Navigate a selected entry into the existing Main Window, expand its process
  ancestors, and highlight the entry after validating the current snapshot.
- Keep the tray capability read-only: no Stop, Force Stop, filesystem opener,
  Terminal, or Editor commands.
- Support system light/dark appearance using the shared design tokens.

## Out of scope

Favorites, notifications, background polling, a second provider pipeline,
destructive process actions in the popover, advanced network diagnostics,
layout customization, and a redesign of the Main Window.

## Security and consistency

The tray capability is scoped to `tray-popover` and grants only accepted
snapshot read/refresh, entry navigation, show-main, event subscription, and
window hide. Navigation validates the snapshot generation and exact opaque
entry reference in the backend. The frontend supplies no PID or filesystem
path. Existing process action authorization remains available only to the Main
Window. Snapshot counts describe observed data and do not claim complete system
visibility.

The tray consumes the shared runtime snapshot; opening it does not scan. A
manual refresh continues to use the application refresh coordinator and
publishes the accepted snapshot. Failed refresh state is retained in shared
runtime state and is returned with the tray snapshot; an event updates an
already-open popover, and a later successful snapshot clears the stale status.
The Main Window is reused rather than
duplicated. Clicking outside or losing focus hides the popover; Escape and
window-close also hide it. Native monitor work-area geometry positions and
clamps the popover.

## Acceptance criteria

| ID    | Criterion                                                                                                                                 | Validation                                   | Status                                                                    |
| ----- | ----------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------- | ------------------------------------------------------------------------- |
| AC-01 | Primary click toggles one compact popover on supported macOS and Windows tray surfaces; rapid clicks do not create duplicate windows.     | Native and interactive tests                 | macOS build passes; interactive macOS and Windows NOT RUN                 |
| AC-02 | The native context menu retains Show, Refresh, and Quit and is not replaced by the popover.                                               | Native and interactive tests                 | Implementation reviewed; CI pending; interactive NOT RUN                  |
| AC-03 | Endpoints are grouped only by validated process instance identity; each distinct endpoint remains visible.                                | Frontend unit tests                          | PASS locally                                                              |
| AC-04 | Search follows W014 semantics, counts describe the full accepted snapshot, and partial/stale states remain explicit.                      | Frontend tests                               | PASS locally, including stale state before and after popover subscription |
| AC-05 | Selecting a process or endpoint opens and focuses the existing Main Window and navigates only to the current matching snapshot entry.     | Backend and frontend tests; interactive test | Automated tests pass; interactive NOT RUN; first-open event race noted    |
| AC-06 | Refresh uses the shared accepted snapshot and does not create an independent scan or continuous tray polling.                             | Backend and frontend tests                   | PASS locally                                                              |
| AC-07 | Popover placement is clamped to the selected monitor work area and dismissal does not terminate the app.                                  | Native geometry tests and interactive tests  | Native geometry tests pass; interactive macOS and Windows NOT RUN         |
| AC-08 | The tray window has no destructive process or project-opening permissions.                                                                | Capability test and security review          | PASS locally; independent review PASS                                     |
| AC-09 | Light/dark appearance, keyboard labels, focus, and readable compact layout are supported.                                                 | Frontend tests and interactive review        | Frontend tests pass; interactive macOS and Windows NOT RUN                |
| AC-10 | Existing Runtime Inspector Search, Process Tree, Refresh, metrics, network context, project actions, Stop, and Force Stop remain correct. | Regression suite and CI                      | Local regression suite passes; CI pending                                 |
| AC-11 | macOS and Windows native CI pass; interactive behavior is reported separately for each available desktop.                                 | CI and platform validation                   | NOT RUN                                                                   |

## Validation plan

- Unit-test safe process grouping, unknown identity separation, and endpoint
  preservation.
- Test popover loading/empty/partial/error states, W014 filtering, counts,
  refresh, navigation failures, and accessible labels.
- Test backend generation/reference validation and capability scoping.
- Test tray placement at work-area edges, nonzero monitor origins, and display
  scale factors.
- Run frontend formatting, lint, typecheck, tests/build, Rust formatting,
  Clippy/tests, documentation checks, security/dependency checks, and
  `git diff --check`.
- Validate the isolated debug app interactively on macOS and Windows where
  available. CI is not a substitute for desktop acceptance.

## Progress

- Confirmed FR-010 already requires platform-appropriate tray access, quick
  listener/runtime overview, and basic supported actions. W027 is a P1
  usability follow-up to completed W013, not a replacement for W013.
- Implemented the read-only snapshot popover, native context menu, refresh and
  Main Window navigation paths, identity-safe grouping, and monitor-aware
  placement in the isolated feature worktree.
- Persisted refresh-failure state alongside the shared runtime service and
  covered both an already-open popover and a popover opened after a failed
  refresh. The display retains the last accepted snapshot and clears its
  stale warning when a newer snapshot arrives.
- Frontend (127 tests) and Rust (137 unit plus native integration tests),
  Clippy, formatting, typecheck, production frontend build, macOS debug build,
  docs checks, and dependency audits pass locally. Independent UX and
  architecture/security reviews pass after the stale-state correction.
- Interactive tray click, placement, and dismissal acceptance remains NOT RUN
  on macOS and Windows; Windows native CI and final PR CI remain outstanding.
- The required high-severity JavaScript audit found the existing Vite lockfile
  version affected by GHSA-587h-7wmj-h5pm. The lockfile now resolves patched
  Vite 8.3.5 within the existing toolchain; `pnpm audit --audit-level high`
  reports no known vulnerabilities. This changes no application dependency or
  runtime feature.
- No user-installed `/Applications/Thaa.app` was used or modified. Interactive
  acceptance must target a verified isolated debug build.
