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
arguments reports them unavailable. W015 derives an optional project-root
path from available working-directory metadata in the application runtime
snapshot; it does not change `ProcessInfo` or process identity. W016 adds a
separate optional `GitContext` with repository root and a named/detached branch
from the same working-directory metadata. Git context does not change
`ProcessInfo`, process identity, or action authorization.

W017 adds `ProcessResourceSample` as optional, provider-collected observation
data on `ProcessInfo`: cumulative CPU time, resident-memory bytes, and a
monotonic sample instant. These fields are not identity evidence. The
application derives `ProcessResourceMetrics` for a runtime snapshot: CPU
percentage of total logical CPU capacity (0–100, unavailable until a valid
second sample), resident bytes, and uptime from observed process start time.
Resource values remain outside `ProcessActionTarget` and cannot authorize or
change Stop or Force Stop behavior.

W020 adds `RuntimeKind` as optional process-presentation metadata on a runtime
snapshot entry. The application derives it from the already-observed process
name using exact, case-insensitive runtime-host rules. The controlled set is
Node.js, Python, Java/JVM, Ruby, and PHP; missing or non-Unicode names remain
unclassified. Runtime classification does not update `ProcessInfo`, process
identity, resource sampling, project/Git context, or `ProcessActionTarget`.
It is convenience metadata, not evidence of source language or process trust.

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

W011.2's Windows provider reports the executable basename without extension
as the process name, derived only from a successful executable-path query.
Windows FILETIME creation timestamps are converted to `SystemTime`; this is
identity evidence, not authorization. Windows command-line strings are not
converted into structured argument vectors, and its working-directory field
is unavailable under the current public-API strategy.

At the transport boundary, expose only a stable category and safe user message. Keep native details in bounded, privacy-aware diagnostics; do not forward command arguments, raw stacks, or secret-bearing strings.

## Platform capabilities

`PlatformCapabilities` describes platform-wide support for command argument
reads, working-directory reads, graceful stop, and force stop using
`CapabilitySupport::{Supported, Unsupported}`. Parent-process access is
optional W018 process metadata rather than a platform-wide action capability.
These flags do not assert that a particular process is
readable or actionable; per-process outcomes remain authoritative. The
`PlatformCapabilitiesProvider` port in `domain::capabilities` exposes this value independently of
`ProcessProvider`.

## Binding scope

`classify_binding` is a pure function over `Option<IpAddr>` and describes the
observed listener's bind scope, not remote reachability:

- `LoopbackOnly` for IPv4/IPv6 loopback addresses, including IPv4-mapped
  IPv6 loopback
- `WildcardIpv4` for only the IPv4 unspecified address `0.0.0.0`
- `WildcardIpv6` for the IPv6 unspecified address `::`
- `SpecificAddress` for another known address not covered by those categories,
  including mapped IPv6 addresses that are not loopback
- `Unknown` when the address is unavailable

Multicast addresses and IPv4 limited-broadcast addresses, including their
IPv4-mapped IPv6 forms, are `Unknown`; they are not presented as ordinary
specific binds. `SpecificAddress` describes only that the address is known and
does not imply unicast behavior or reachability.

The listener retains its original address for display. A specific address is
not further classified as private, link-local, or globally routable. IPv6
wildcard does not imply IPv4 acceptance; Thaa does not inspect socket options
that determine dual-stack behavior. Wildcard and specific bindings do not
prove LAN or Internet reachability, firewall permission, NAT forwarding, or
container/VM forwarding. The classifier does no interface enumeration, DNS,
network probing, firewall inspection, or external lookup. Scoped IPv6
addresses with nonzero scope identifiers remain unknown because W006's
`IpAddr` does not preserve the scope identifier. W019 implements the binding
explanation portion of FR-016 while retaining PR-009's factual boundary.

## Identity-bound process actions

W012.0 adds `ProcessActionTarget`, which can only be created from an observed
identity with positive PID and available start time. If executable path was
observed it is carried as additional evidence. Process name, arguments, and
working directory are mutable or presentation-oriented and are excluded.
Fresh identity validation requires the same PID and start time, and the same
executable path when one was observed. Missing or conflicting evidence fails
closed. Structural equality of `ProcessIdentity` remains distinct from this
explicit policy check.

`ProcessController` accepts only this target plus one `ProcessAction`
(`GracefulStop` or `ForceStop`); it has no PID-only operation. Platform
controllers own revalidation adjacent to the native action. `Requested` means
the OS accepted a request and does not assert that exit completed;
`AlreadyExited` is explicit. Errors are stable and platform-neutral. W012.0
introduced these values without an implementation; W012.1 adds the native
macOS controller below the unchanged shared contract.

PID is not durable identity. macOS uses SDK-defined `kinfo_proc.p_starttime`
from `sysctl(KERN_PROC_PID)` with microsecond timeval precision; the later
signal still targets a positive PID, leaving a residual check-to-signal race.
Windows can compare creation time and executable identity through the same
process handle used for `TerminateProcess`, narrowing PID reuse across the
operation. Neither policy authorizes future action without fresh platform
validation.

W012.1 implements macOS Graceful Stop as a `SIGTERM` request and Force Stop as
a separate `SIGKILL` request. It shares the provider's precise start-time
query, requires a positive representable PID, and refuses a mismatched or
unavailable identity. `Requested` means `kill(2)` accepted the signal; it does
not confirm exit. The syscall still targets a PID after a separate `sysctl`
query, so a narrow PID reuse race remains. W012.2 implements Windows Force Stop
through the same HANDLE used for identity revalidation, while Graceful Stop
remains unsupported. Windows also returns `Requested` without waiting for
exit.

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

## Parent process context (W018)

`ProcessInfo.parent_process_id` is optional OS-reported snapshot metadata. It is
not durable identity and must never authorize process actions. Runtime Inspector
builds a forest only from unique listener-owner processes in the accepted
snapshot. Parent links are joined only when the referenced process is uniquely
represented; missing/ambiguous parents, self-links, and cycles do not invalidate
process or listener data.
