# Domain Model

**Applies to:** `THAA-REQ-0.1` · **Status:** Initial architecture baseline

The domain uses normalized, platform-neutral values. The following are conceptual contracts, not production Rust declarations; implementation may refine names without changing frozen requirement meaning.

## Listener

`NetworkListener` represents one observed listening TCP endpoint:

- protocol (`TCP` for the P0 requirement)
- local IP address and port, kept separate so IPv4/IPv6 are unambiguous
- owner resolution: known process identity or explicitly unresolved
- observed binding scope, derived from the address only when determinable

P0 collection returns listening TCP endpoints, so listener state is implicit; do not introduce a state enum that only has one possible value. If a later provider genuinely returns multiple states, extend the model with evidence and tests.

An unresolved PID is not an error and must not be replaced by a guessed owner. Duplicate platform rows may be normalized in infrastructure only when endpoint and ownership semantics are preserved.

## Process identity and information

`ProcessIdentity` is a target handle, not merely a display name. It contains the OS process identifier plus the identity evidence observed with it, such as normalized process name, executable path, and start time when available. Evidence fields can be absent; they are not assumed universally available.

`ProcessInfo` associates that identity with process metadata. P0 fields include process name, executable path, command/arguments, and working directory where available. PID belongs to identity. Working directory is P0 context; project root and Git context are P1 and are not part of this baseline's P0 use case.

## Per-field availability

Each optional metadata field uses a value-or-unavailable representation:

- `Available(value)` when observed
- `Unavailable(reason)` for permission denied, unsupported, inaccessible/protected, or provider limitation

Never fabricate an empty string, zero, or guessed value to mean unavailable. A field-specific permission restriction is metadata, not a failure of the entire process query. Whole-query provider/OS failures are application errors. A process that ended during inspection has a distinct process-disappeared outcome.

At the transport boundary, expose only a stable category and safe user message. Keep native details in bounded, privacy-aware diagnostics; do not forward command arguments, raw stacks, or secret-bearing strings.

## Platform capabilities

`PlatformCapabilities` is a small description of supported operations relevant to P0: command-line access, working-directory access, parent-process access, graceful stop, and force stop. Represent support as available/unsupported (and permission restrictions where they are target-specific).

Separate platform-wide support from per-process outcomes. A global capability does not promise that every process is readable or actionable. The observed field/action result is authoritative for a particular process. Do not grow this into a registry for hypothetical P1–P3 features.

## Binding scope

A pure classifier maps a normalized local address to:

- `LoopbackOnly` for loopback addresses such as `127.0.0.1` and `::1`
- `PotentiallyReachable` for wildcard or non-loopback interface addresses such as `0.0.0.0`, `::`, or a specific non-loopback address
- `Unknown` when the address or classification evidence is unavailable

`PotentiallyReachable` means bound beyond loopback; it does not claim LAN reachability or Internet exposure. Classification is separate from socket collection and must be unit-testable without OS access. This supports FR-003 and PR-009; the fuller P2 network-awareness feature remains deferred.

## Process actions and results

Product-level actions are `Stop` and `ForceStop`; do not expose Unix signal names. A stop request carries the previously observed `ProcessIdentity`. Immediately before action, the application asks the platform boundary to resolve the current process and compare every available stable identity signal. If signals conflict, reject the action. If the platform cannot establish a sufficiently safe target, report an action rejection/unsupported outcome instead of silently acting on PID alone. Never target by process name or silently escalate graceful stop to force stop.

`ProcessActionResult` distinguishes completed, rejected, disappeared, permission-denied, unsupported, and provider/OS failure outcomes. The UI can then refresh and show the current snapshot instead of assuming the process state.

## Error boundaries

| Boundary | Examples | Treatment |
|---|---|---|
| Platform/infrastructure | OS API failure, subprocess launch/exit, malformed native output | Map to typed provider error; retain bounded internal context, redact sensitive values |
| Application | provider failure, process disappeared, action rejected | Orchestrate and preserve semantic category; no platform branching |
| Transport | invalid input, stable error code and safe message | DTO suitable for frontend; no raw stack/native text |
| Presentation | understandable permission, unavailable, stale, or failure state | No backend internals; allow retry/refresh when meaningful |

Required semantic categories are unsupported capability, permission denied, process disappeared, provider failure, parse failure, OS API failure, action rejected, and invalid input. Do not collapse all metadata absence into an application error.

## Traceability

The models support FR-002/003/006/008/009/011, NFR-003/004/005/008/009, and PR-002/004/005/006/007/009/011. They define no implementation status and do not move FR-012/013 into P0.
