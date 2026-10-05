# W013 — Tray Runtime List

## Status

Done on merge. PR #32 is ready for integration after the user-confirmed manual
macOS acceptance, clean read-only re-review, and successful required CI on the
final reviewed implementation. This status takes effect when PR #32 merges to
`develop`.

## Objective

Connect the existing listener, process inspection, and safe process action
backends to a usable Tauri/React runtime list and system tray overview. This is
the First Usable Build milestone, not completion of every P0 requirement.

## User-facing Acceptance Scenario

Start a controlled local TCP listener, launch Thaa, see its address/port and
owning process, refresh the list, and request only actions supported by the
platform. Graceful Stop is available on macOS; Force Stop is explicitly
confirmed on both current platforms. A `Requested` result is followed by a new
observation and is never presented as proof that the process has exited.

## Scope

- Compose the current native `PortProvider`, `ProcessProvider`,
  `ProcessController`, and capabilities provider once at the platform boundary.
- Add a shared runtime snapshot use case/coordinator with a single port scan,
  deduplicated process inspection, per-listener outcomes, coalesced refresh,
  and stale-generation protection.
- Expose typed Tauri snapshot, refresh, validated local URL open, and
  identity-reference-based action commands.
- Replace the smoke screen with an accessible runtime list, manual refresh,
  ten-second refresh while the main window is visible, supported actions,
  clipboard actions, and Force Stop confirmation.
- Add a compact tray listener overview, refresh/show/quit menu, and hide-on-close
  behavior.
- Update architecture, testing, and traceability documentation.

## Out of Scope

Search/filter (W014), project/Git/runtime detection, metrics, process trees,
background refresh while the main window is hidden, tray process actions,
automatic Force escalation, and any weakening of W012 controller
revalidation. No PID-only action command or arbitrary URL input is introduced.

## Architecture

`application::runtime_inspection::RuntimeInspector` composes domain provider
ports and owns one canonical snapshot and action-target reference map. Tauri
commands map explicit DTOs and dispatch synchronous providers through
`tauri::async_runtime::spawn_blocking`. `lib.rs` is the sole target-specific
composition root. React consumes snapshot DTOs and opaque snapshot references;
native APIs and shared domain paths do not cross IPC as action evidence.

## Runtime Snapshot Model

One row is retained for every normalized listener. Owner PIDs are inspected
once per refresh through W011. Unknown ownership and per-process errors remain
visible beside the listener. Successful partial metadata remains successful.
Port scan completeness and platform action capabilities accompany each
snapshot. URLs use a fixed `http` scheme and only current snapshot listener
references can be opened. Wildcard addresses map to loopback for URL actions;
IPv6 zones are escaped for URL formatting.

## Refresh Semantics

Manual refresh and the ten-second visible-window tick call the shared
coordinator. Only one synchronous provider scan runs at a time; concurrent
requests coalesce to the latest generation. Superseded results are discarded.
If an action completes during an existing UI scan, one follow-up observation
is queued so that pre-action scan cannot be the only post-action view.
The last good snapshot stays visible after scan failure with a safe warning.
Tray activation/refresh requests a new shared snapshot and updates the compact
menu. No scan is started by the tray as a separate coordinator.

## IPC Contract

`get_runtime_snapshot`, `refresh_runtime_snapshot`, `open_listener_url`, and
`request_process_action` are thin commands. Snapshot DTOs include a monotonic
scan generation so React ignores a delayed older IPC response after receiving
a newer tray event. DTOs use lower-camel fields and explicit tagged variants.
Native paths and arguments become lossy
text only for display. Those display strings never round-trip into process
action identity. Actions accept a bounded opaque action-target reference, not a
PID or path. The backend resolves only a target retained for the current
successful snapshot and the platform controller performs fresh identity
revalidation. URL opening resolves a current listener reference in Rust and
uses the fixed observed local address/port.

## Process Action Flow

The backend exposes actions only when the snapshot provides a validated W012
`ProcessActionTarget` and the platform capability reports support. macOS shows
Graceful Stop (SIGTERM) and Force Stop (SIGKILL); Windows shows Force Stop
(TerminateProcess) only. Force Stop requires a deliberate confirmation. After
`Requested`, the UI says the request was sent, refreshes, and does not claim
exit. Identity mismatch and already-exited results prompt observation without
retry or escalation.

## Platform Capability Behavior

Capabilities are included in the backend snapshot and drive controls; React
does not infer the OS from the user agent. macOS generic graceful and force
actions are supported; Windows generic graceful action is unsupported and
force action is supported. Per-target permission/identity failures remain
possible on either platform.

## UI States

The screen includes initial loading, complete empty, partial scan, provider
error with retry, per-process unavailable metadata, refresh-in-progress,
action-in-progress, and action-result messages. Long process values are
truncated, hostile control/bidi characters are replaced in visible text, and
Force Stop confirmation is keyboard-addressable with a clear process/PID
context. Rows retain port and binding hierarchy without claiming Internet
exposure.

## Accessibility

Semantic headings, status/alert regions, labeled icon buttons, visible focus,
keyboard-operable actions, non-color-only binding labels, responsive layout,
and reduced-motion handling are implemented. Manual keyboard and visual
validation are still required.

## Security

No snapshot logging or process-derived command execution is added. The opener
command does not accept a URL; it resolves a current backend listener
reference. Clipboard writes occur only after explicit user action. Frontend
action confirmation is not the security boundary: action references resolve
backend-retained identity and W012 controllers revalidate immediately before
native actions. No privilege elevation or bulk/tree action is added.

## Testing

Application tests cover shared PID deduplication, per-process outcomes,
partial/empty results, refresh failure retaining the prior snapshot and later
recovery, reference expiry, concurrent initial scan coalescing, overlapping
refresh coalescing, and stale-generation result rejection. Native snapshot
integration tests run on macOS and Windows with a test-owned loopback listener.
Frontend tests cover loading/data/empty/partial/error, capability-aware
controls, force confirmation, and post-action refresh. Existing platform
provider/controller suites remain required. CI and remaining coordinator/IPC
test work are recorded in the execution report before merge.

## Manual Validation

The user manually tested the W013 application on macOS and confirmed that real
listening ports and processes appear correctly, manual refresh works, and both
Graceful Stop and Force Stop behave as expected. This is user-confirmed
acceptance evidence; the Agent did not independently observe the interactive
session. The user also confirmed that action behavior matches expectations.

Windows interactive UI validation was not performed. Windows backend/native
and Tauri build validation is provided by the Windows CI job; this is not a
claim of visual GUI acceptance on Windows.

## Known Limitations

- W013 does not include search/filter; W014 owns it per the approved work plan.
- Auto refresh runs only while the main document reports visible; tray
  activation triggers refresh separately.
- Display paths are lossy strings and are never used as identity evidence.
- Tray overview is intentionally compact and limited to eight rows.
- Windows interactive visual validation was not performed.
- Manual macOS evidence is supplied by user confirmation rather than an Agent
  screen recording or retained screenshot.

## Acceptance Criteria

- [x] Shared snapshot combines native listeners, deduplicated owner metadata,
      capabilities, and safe per-row outcomes.
- [x] Platform composition selects native implementations centrally.
- [x] Tauri IPC exposes snapshots and actions without PID-only authorization.
- [x] React provides a real runtime list, refresh, supported actions, and
      Force Stop confirmation.
- [x] Tray menu provides listener overview, refresh, show, and quit.
- [x] Auto refresh is ten seconds while visible; no hidden-window timer scan.
- [x] No Search/Filter or P1 runtime intelligence was added.
- [x] Rust, frontend, audit, documentation, and three required CI gates pass
      on the reviewed final implementation head.
- [x] Read-only review passes and the P2 finding is resolved; re-review found
      no remaining findings.
- [x] User confirms the macOS First Usable scenario and process actions work.

## Validation

The final reviewed implementation passed Rust/frontend validation and the
Shared quality, macOS native validation, and Windows native validation jobs
(GitHub Actions run `37131667714`; final PR head at validation:
`e214c127e99d4c91b59c37f1397c42f28b92d07e`). The user confirmed manual macOS
acceptance. The status is `Done on merge`; integration and post-merge checks
are recorded in GitHub/Git history after merge rather than through a separate
closeout PR.

## Review Findings

The read-only review found that an action-triggered observation could be
dropped while another UI refresh was running. The frontend now queues one
follow-up scan after the active scan, and a deferred-promise test verifies the
post-action result is applied. Re-review at `e0561b1` found no remaining issues.

## Documentation Impact

Documents the concrete application coordinator, DTO boundary, tray behavior,
refresh lifecycle, capability-aware actions, and W014 search/filter deferral.
The frozen requirements baseline remains THAA-REQ-0.1; planned-work
assignments were clarified to match W013 Tray Runtime List and W014 Search and
Filter.
