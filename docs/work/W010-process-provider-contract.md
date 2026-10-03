# W010 — ProcessProvider Contract

## Status

In Progress — implementation is on `feat/process-provider-contract`.
Change to `Done on merge` on the accepted PR head after review and required CI
are complete.

## Objective

Define a stable, platform-neutral contract for read-only inspection of one
process using the W006 process domain models.

## Scope

- Add `ProcessProvider`, query error categories, and a safe stable error type
  under `domain`.
- Add deterministic test-double contract coverage and reusable assertions for
  later native providers.
- Document lookup, field availability, disappearance, and capability semantics.

## Out of Scope

Native macOS/Windows inspection, process enumeration/batching, process
control/revalidation, port-provider changes, Tauri IPC, frontend changes,
project/Git/runtime detection, resource metrics, and new dependencies.

## Requirements

Establishes the shared provider-contract foundation required for FR-006 and
supports FR-011's process-ended/error semantics. It does not inspect a live
process or implement FR-006/FR-011. The frozen baseline remains
`THAA-REQ-0.1`.

## Dependencies

W006 process models and metadata availability, W007 provider-contract
conventions, W003/ADR-003 provider boundaries, W004 security/testing policy,
and the integrated W008/W009 platform-provider architecture govern this work.
The project-scoped `rust-patterns` and `security-and-hardening` skills apply.

## Design Notes

`ProcessProvider::inspect(ProcessId)` is synchronous, object-safe, and
`Send + Sync`, matching `PortProvider`. It returns W006 `ProcessInfo`; the
identity PID must equal the requested PID. A PID is a lookup key, not durable
identity or authorization for later process control.

`ProcessDisappeared` covers a PID that is absent at inspection or whose exit
is detected during collection. When existence remains established, field
restrictions stay inside `FieldAvailability<T>` and do not fail the entire
query. Whole-query permission denial and provider/OS failures use
`ProcessProviderError`. Errors expose semantic categories only.

The provider returns one process, does not enumerate by name, and does not
perform process actions. `ProcessInfo` preserves native paths, structured
`OsString` arguments, and W006 identity evidence. `PlatformCapabilities` is a
separate platform-wide domain value; W010 adds no capability query or action
support to this contract. Cancellation remains above the synchronous provider.

## Acceptance Criteria

- [ ] `ProcessProvider` is defined under the shared domain boundary and
      consumes one `ProcessId`.
- [ ] Successful output reuses W006 `ProcessInfo` and preserves the requested
      PID and field-level availability.
- [ ] Process disappearance, whole-query denial, and other provider failures
      are distinct, stable, and privacy-safe.
- [ ] Contract remains synchronous, object-safe, `Send + Sync`, and free of
      platform/runtime types.
- [ ] Deterministic tests cover full and partial metadata, disappearance,
      permission/provider failure, native path and structured argument
      preservation, and hostile-looking metadata as inert data.
- [ ] Reusable assertions are available to future macOS and Windows native
      provider test targets.
- [ ] Documentation distinguishes contract foundations from live process
      inspection; no functional requirement is marked implemented.
- [ ] Read-only review, all required CI, and post-merge integration pass.

## Validation

Record exact Rust, frontend, audit, documentation, and three-job CI results
here after they run. Local and CI provider-contract tests use only a test
double; they do not establish native macOS or Windows process inspection.

## Security Considerations

Process metadata is untrusted and potentially sensitive. The contract
preserves names and arguments as data, structured argument elements, and
native paths. It defines no shell, logging, transport, process-action, or
identity-authorization behavior. Whole-operation failures never become a
default `ProcessInfo`; field-level access restrictions remain explicit.

## Platform Considerations

The shared contract contains only W006 standard Rust domain types and has no
platform conditionals. Future providers must map native timestamp, path, and
argument forms into W006 types and report unsupported or inaccessible fields
explicitly. PID reuse remains possible; later action code must perform its
own immediate identity revalidation.

## Documentation Impact

Update the provider architecture, domain model, data flow, directory map,
testing strategy/catalog, and this work record to describe the implemented
contract. Do not alter frozen requirements or accepted ADRs.

## Review Findings

Pending mandatory read-only review after the PR exists.

## Known Limitations

No native provider exists in W010; metadata fidelity, per-process access
behavior, and platform capability discovery remain unverified. No
ProcessController or user-facing process inspection is implemented.
