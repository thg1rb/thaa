# Domain Model

**Applies to:** `THAA-REQ-0.1` · **Status:** Initial model implemented in W006

The domain uses normalized, platform-neutral Rust values under
`src-tauri/src/domain/`. It has no Tauri, frontend, OS API, provider-output,
or shell dependency. These types are foundations for future providers and use
cases; they do not perform inspection or implement a user-facing capability.

## Listener

`NetworkListener` represents one observed listening TCP endpoint. W006 defines
`NetworkProtocol::Tcp` as the only protocol in the current P0 model, an
`IpAddr` when the local address is known, a `NonZeroU16` local port, and an
optional `ProcessId` owner. No owner is valid unresolved data; it is distinct
from a failed provider query. Port zero cannot represent an observed listener.

`ListenerState` is omitted because provider results are defined as listening
TCP endpoints. A binding scope is derived from the optional address instead of
stored alongside it, avoiding contradictory address/scope values.

## Process identity and information

`ProcessId` is a distinct `u32` value to prevent confusion with other integer
values. It does not reserve zero as an absence sentinel; absent IDs use
`Option<ProcessId>`.

`ProcessIdentity` contains the PID and field-availability values for observed
identity evidence: process name, executable path, and start time. Names use
`OsString` and filesystem paths use `PathBuf`, avoiding a forced UTF-8
conversion. Start time is identity evidence only here; resource reporting and
uptime remain outside W006. Structural equality is value equality, not an
authorization check for destructive actions.

`ProcessInfo` associates the identity with command argument elements and
working-directory metadata. Arguments are stored as `Vec<OsString>`, not
reconstructed into a shell string. A provider that cannot reliably normalize
arguments reports them unavailable. Project root and Git context remain P1
and are not part of the P0 model.

## Per-field availability

`FieldAvailability<T>` is the field-level value-or-unavailable representation:

- `Available(value)` when observed
- `Unavailable(reason)` for permission denied, unsupported, inaccessible, or provider limitation

An empty observed string remains a value; it is not an absence sentinel. Field
permission or support limitations do not fail the entire query. Process
disappearance and whole-query provider/OS failures belong to future
provider/application error contracts and are not represented by this enum.

W010's `ProcessProvider` consumes one `ProcessId` and returns `ProcessInfo`.
Its query-level `ProcessProviderError` distinguishes disappearance and whole
operation failures from field availability. A successful `ProcessInfo`
retains the requested PID and may contain unavailable fields. The error
categories contain stable semantic messages only, never native error text.

At the transport boundary, expose only a stable category and safe user message. Keep native details in bounded, privacy-aware diagnostics; do not forward command arguments, raw stacks, or secret-bearing strings.

## Platform capabilities

`PlatformCapabilities` currently describes platform-wide support for command
argument reads, working-directory reads, graceful stop, and force stop using
`CapabilitySupport::{Supported, Unsupported}`. Parent-process access is P1
and is not included. These flags do not assert that a particular process is
readable or actionable; per-process outcomes remain authoritative. W010 keeps
this platform-wide value separate from `ProcessInfo`; no capability query or
action-support result is added to `ProcessProvider`.

## Binding scope

`classify_binding` is a pure function over `Option<IpAddr>`:

- `LoopbackOnly` for IPv4/IPv6 loopback addresses
- `PotentiallyReachable` for unspecified or other non-loopback addresses
- `Unknown` when the address is unavailable

`PotentiallyReachable` means only that the binding extends beyond loopback. It
does not establish interface reachability, LAN access, or Internet exposure.
The classifier does no interface enumeration and supports FR-003/PR-009
without implementing collection or the fuller P2 network-awareness feature.

## Future process actions and results

W006 does not implement process actions or action-result types. When those are
introduced, product-level actions are `Stop` and `ForceStop`; do not expose
Unix signal names. A stop request carries the previously observed
`ProcessIdentity`. Immediately before action, the application asks the
platform boundary to resolve the current process and compare every available
stable identity signal. If signals conflict, reject the action. If the
platform cannot establish a sufficiently safe target, report an action
rejection/unsupported outcome instead of silently acting on PID alone. Never
target by process name or silently escalate graceful stop to force stop.

`ProcessActionResult` distinguishes completed, rejected, disappeared, permission-denied, unsupported, and provider/OS failure outcomes. The UI can then refresh and show the current snapshot instead of assuming the process state.

## Error boundaries

| Boundary                | Examples                                                        | Treatment                                                                             |
| ----------------------- | --------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| Platform/infrastructure | OS API failure, subprocess launch/exit, malformed native output | Map to typed provider error; retain bounded internal context, redact sensitive values |
| Application             | provider failure, process disappeared, action rejected          | Orchestrate and preserve semantic category; no platform branching                     |
| Transport               | invalid input, stable error code and safe message               | DTO suitable for frontend; no raw stack/native text                                   |
| Presentation            | understandable permission, unavailable, stale, or failure state | No backend internals; allow retry/refresh when meaningful                             |

Future provider/application/transport layers must preserve semantic categories
such as unsupported capability, permission denied, process disappeared,
provider failure, parse failure, OS API failure, action rejected, and invalid
input. W006 defines only field-level metadata availability; it does not
implement this full error architecture. Do not collapse all metadata absence
into an application error.

## Traceability

W006 implements domain foundations for FR-002/003/006 and supports future
FR-008/009/011 work through identity evidence, capability, and availability
types. It does not discover listeners/processes, perform actions, or mark any
functional requirement implemented. It does not move FR-012/013 into P0.
