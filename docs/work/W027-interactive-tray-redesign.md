# W027 — Interactive Tray Redesign

Status: Implementation complete; acceptance and merge gates open
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

| ID    | Criterion                                                                                                                                 | Validation                                   | Status                                                                                                                     |
| ----- | ----------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| AC-01 | Primary click toggles one compact popover on supported macOS and Windows tray surfaces; rapid clicks do not create duplicate windows.     | Native and interactive tests                 | User-confirmed T01–T03 and D06–D07 PASS on their validation builds; Windows interactive NOT RUN                            |
| AC-02 | The native context menu retains Show, Refresh, and Quit and is not replaced by the popover.                                               | Native and interactive tests                 | T04 context menu and T05 Show Thaa PASS on Toggle Validation; current-head CI pending after push                           |
| AC-03 | Endpoints are grouped only by validated process instance identity; each distinct endpoint remains visible.                                | Frontend unit tests                          | Automated grouping and endpoint-preservation tests PASS                                                                    |
| AC-04 | Search follows W014 semantics, counts describe the full accepted snapshot, and partial/stale states remain explicit.                      | Frontend tests                               | Automated tests PASS, including stale state before and after subscription                                                  |
| AC-05 | Selecting a process or endpoint opens and focuses the existing Main Window and navigates only to the current matching snapshot entry.     | Backend and frontend tests; interactive test | V04 and C01–C04 (title, empty card, icon, metadata navigation) PASS on prior builds; first-open delivery risk remains open |
| AC-06 | Refresh uses the shared accepted snapshot and does not create an independent scan or continuous tray polling.                             | Backend and frontend tests                   | Automated shared-snapshot and Refresh tests PASS                                                                           |
| AC-07 | Popover placement is clamped to the selected monitor work area and dismissal does not terminate the app.                                  | Native geometry tests and interactive tests  | Geometry tests PASS; D01–D08 dismissal PASS; direct external-display placement evidence pending                            |
| AC-08 | The tray window has no destructive process or project-opening permissions.                                                                | Capability test and security review          | Capability tests and independent security review PASS                                                                      |
| AC-09 | Shared application theme, keyboard labels, focus, and readable compact layout are supported.                                              | Frontend tests and interactive review        | UI tests PASS; C01–C10 user interaction PASS; no separate whole-design signoff recorded                                    |
| AC-10 | Existing Runtime Inspector Search, Process Tree, Refresh, metrics, network context, project actions, Stop, and Force Stop remain correct. | Regression suite and CI                      | Local regression suite PASS; new PR-head CI pending                                                                        |
| AC-11 | macOS and Windows native CI pass; interactive behavior is reported separately for each available desktop.                                 | CI and platform validation                   | Prior PR-head CI passed; rerun required on closeout head; Windows interactive NOT RUN                                      |

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
- At initial implementation, interactive Tray click, placement, and dismissal
  acceptance had not been run. Later user-confirmed test groups are recorded
  below by validation build; external-display placement and Windows desktop
  interaction remain separate acceptance items.
- The required high-severity JavaScript audit found the existing Vite lockfile
  version affected by GHSA-587h-7wmj-h5pm. The lockfile now resolves patched
  Vite 8.3.5 within the existing toolchain; `pnpm audit --audit-level high`
  reports no known vulnerabilities. This changes no application dependency or
  runtime feature.
- No user-installed `/Applications/Thaa.app` was used or modified. Interactive
  acceptance must target a verified isolated debug build.
- Refined the compact header and summary, removed duplicate Main Window
  navigation, and aligned the popover canvas and surfaces with the existing
  global graphite tokens. The current Main Window theme is dark; the popover no
  longer invents a separate light palette.
- Added three layout-matched initial Skeleton rows and a compact Refresh status
  that keeps the accepted list visible. Refresh failure continues to retain the
  last accepted data and its stale indication.
- Replaced repeated endpoint badges with compact protocol/port chips. Bindings
  sharing a protocol and port are summarized in the row while an explicit
  disclosure preserves every observed address and exact endpoint navigation
  reference.
- Added frontend coverage for initial loading and retry after an initial
  failure, in-progress Refresh, duplicate address binding disclosure with
  exact endpoint-reference navigation, and separate same-name process
  identities. At that implementation stage the frontend suite had 131 passing
  tests, and interactive visual acceptance was still pending. The final
  closeout suite and later human-assisted evidence are recorded below.
- Follow-up corrections address three user-reported usability issues. Tray
  placement now captures the macOS display bounds under the cursor at click
  time and matches that display to Tauri's monitor descriptors; the event
  rectangle and monitor work area use physical coordinates, while captured
  CoreGraphics screen bounds are compared in logical coordinates. On macOS,
  the target physical position is converted with the target monitor's scale
  and applied as a logical position; the logical size is also applied directly.
  This avoids Tao converting physical setters with the still-current scale of
  the previous display. The existing Popover is hidden, resized, and
  repositioned before it is shown on another display: the macOS reveal is
  queued on the same serial main dispatch queue after Tao's asynchronous
  geometry updates. Native and CSS minimum widths no longer override the
  calculated size for a small usable display.
  Invalid tray geometry uses a finite click location or the selected work-area
  center, and valid click/display evidence is checked before primary-monitor
  fallback. Placement tests cover negative monitor origins, mixed backing
  scales, logical setter conversion, monitor matching, work-area clamping, and
  small usable areas. Windows continues to use physical positioning and
  Tauri's physical `monitor_from_point` API.
- Port badges now show the first three distinct protocol/port pairs per
  process instance, then expand in the same badge layout. The `+N more`
  disclosure counts hidden unique protocol/port badges; endpoint totals and
  address-level binding disclosures continue to preserve every observed
  endpoint and its opaque navigation reference. Expansion is keyed to the
  existing process-instance identity.
- Process rows now reserve a stable icon slot and use the existing
  generation-scoped `get_runtime_process_icons` command and its bounded PNG
  assets. Missing or unavailable assets use the existing neutral ProcessIcon
  fallback. The command takes only the accepted snapshot generation and is
  enabled for the Main Window and read-only Popover; the Popover still has no
  process-action or project-opening permissions. The Tauri build manifest now
  registers this command so its generated ACL is present in clean builds.
- Focused frontend and Rust tests cover expanded unique-port presentation,
  icon loading/fallback, capability boundaries, monitor matching at negative
  coordinates and different scales, and clamped geometry. macOS interactive
  multi-monitor placement and Windows interactive validation remain pending.
  The user later passed C01–C10 on the Card Validation build; a separate
  whole-design visual signoff was not recorded. The first-open Main Window
  navigation delivery risk remains separately unresolved.
- Refined the Tray process cards to use the Main Window's existing
  `runtime-card` surface, border, radius, shadow, hover, and focus language with
  a compact Tray-specific density. Process identity and port hierarchy remain
  unchanged; the card now keeps process name and PID/endpoint count above one
  wrapping port row.
- Moved address-level bindings out of individual port flex items into a
  full-width subordinate panel associated with the selected protocol/port.
  The panel reuses `BindingBadge` scope semantics and preserves every endpoint
  navigation reference. One port panel may be open per process instance;
  collapsing the port preview also closes a panel for a now-hidden port.
  Focused frontend coverage verifies panel association, accurate binding
  counts, endpoint navigation, and disclosure behavior. The user later passed
  Show/Hide Bindings interactions C05/C06 on the Card Validation build; a
  separate whole-design visual signoff was not recorded.
- Corrected Tray toggling after a user report that the second left-click
  surfaced the Main Window. The Main Window is configured visible at launch,
  and native window stacking can reveal it behind the Popover when the Popover
  closes. Tray toggling now controls only Popover visibility; it does not hide,
  show, create, or focus the Main Window. Explicit Show Thaa and validated
  process navigation remain the Main Window activation paths. macOS Popover
  activation uses cooperative `NSApplication.activate()` rather than forcing
  activation regardless of the currently active app. The user later confirmed
  visibility independence in V01–V04 on the Visibility Validation build.
  Primary-button down transitions are serialized through shared Popover state;
  Windows' separate primary double-click event is treated as the second click,
  and revision-checked focus-loss dismissal cannot override a newer Tray click.
  Escape now uses a capability-scoped dismissal command to synchronize that
  state. Automated coverage passes locally. User-confirmed toggle results are
  recorded by build below.
- Previous user validation confirmed Main Window Visibility Independence for
  V01–V04. The current Tray card correction makes the noninteractive card
  surface navigate through the existing representative snapshot entry while
  preserving the process-title button as its keyboard target. Nested port,
  binding, and expansion controls retain their own actions; navigation still
  passes only the accepted snapshot generation and opaque entry reference.
  User validation on the Card Validation build confirmed C01–C10, including
  full-card pointer navigation, independent nested controls, hover feedback,
  keyboard operation, visibility behavior, and Tray toggling. The first-open
  navigation delivery risk remains unresolved.
- A user then reported that clicking outside the visible Popover did not
  dismiss it. The existing revision-checked Tauri window-focus-loss path is
  retained. macOS now also observes `NSApplicationDidResignActiveNotification`
  and requests dismissal through the same revision guard, covering app/desktop
  deactivation without global input monitoring or new permissions. Both paths
  are idempotent for a given open revision. Dismissal is armed only after the
  current open session receives native focus, so focus loss from a closed
  session cannot dismiss a newly reopened Popover before it gains focus.
  The user confirmed D01–D08 PASS on the Dismissal Validation build. D08 covers
  dismissal in an external-display scenario; it does not by itself establish
  that the Popover was positioned on the clicked external display.

## Human-assisted acceptance evidence

The user confirmed these results on separate isolated builds. They are
interactive PASS for the tested build and behavior, not a claim that the same
binary is the final pushed PR head.

| Test group | Result     | Validation build                                             | Scope of evidence                                                                                                                                                    |
| ---------- | ---------- | ------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| T01–T05    | 5/5 PASS   | Thaa W027 Toggle Validation (`w027togglevalidation`)         | T01 first click opens; T02 second closes; T03 rapid double-click; T04 context menu; T05 explicit Show Thaa                                                           |
| V01–V04    | 4/4 PASS   | Thaa W027 Visibility Validation (`w027visibilityvalidation`) | V01 visible Main stays visible; V02 hidden Main stays hidden; V03 explicit Show; V04 process navigation                                                              |
| C01–C10    | 10/10 PASS | Thaa W027 Card Validation (`w027cardvalidation`)             | C01 title navigation; C02 empty-card navigation; C03 icon navigation; C04 metadata navigation; C05/C06 Bindings; C07 hover; C08 keyboard; C09 visibility; C10 toggle |
| D01–D08    | 8/8 PASS   | Thaa W027 Dismissal Validation (`w027dismissalvalidation`)   | D01 other app; D02 desktop; D03 Main Window; D04 internal controls; D05 Escape; D06 Tray toggle; D07 rapid reopen; D08 external-display dismissal                    |

These groups validate the reported behavior on their respective builds. Direct
external-display placement, Windows interactive behavior, and a standalone
whole-design visual signoff are not established by these results.

## Closeout and remaining gates

- Closeout local checks: 141 frontend tests and 187 Rust tests pass; ESLint,
  TypeScript, frontend production build, Rust formatting, Clippy with warnings
  denied, Markdown link validation, Prettier, and `git diff --check` pass.
  `pnpm audit` reports two moderate advisories in dev-only `@vitest/mocker`
  4.1.11 (GHSA-qxg6-89mg-rqv8 and GHSA-xp26-3454-mjgp), both patched in
  4.1.12; `pnpm audit --prod` reports none. These are in the existing test
  toolchain and were not introduced by W027. `cargo audit` exits successfully
  with five allowed Rust advisories (three unmaintained-crate notices and two
  advisories for `glib`/related GTK dependencies); no dependency upgrade is
  included in this closeout.
- PR #77 CI passed on the previously pushed `3f1ac232` head (run
  `39597801214`). On implementation head `a21e88f`, Shared Quality, macOS
  Native Validation, and Windows Native Validation all passed in run
  `39969142079`. The preceding head `356ce82` had failed only Windows Clippy
  because the macOS-only monitor comparison helper was compiled but unused on
  Windows; the helper is now macOS-gated. Final docs-only closeout update CI is
  rerunning on the current PR head.
- `navigate_to_runtime_entry` validates the current generation and opaque
  entry, shows/focuses Main Window, then emits snapshot and navigation events.
  The frontend listener registers asynchronously and there is no ready
  handshake or acknowledgment. First-open delivery reliability remains a
  credible follow-up; no correction is included in this closeout.
- Windows interactive Tray validation remains NOT RUN. A normal process
  selection test passed previously, but first-open delivery has not been
  specifically tested. A documented disposition is required before merge.
