# W011 — Shared Process Inspection

## Status

Done on merge — [PR #25](https://github.com/thg1rb/thaa/pull/25) is the
reviewed implementation and integration record.

## Objective

Implement the shared application flow that inspects caller-supplied process IDs
through W010 `ProcessProvider`, retaining an independent outcome per process.

## Scope

- Add a platform-neutral application use case using W006 process types and the
  W010 provider contract.
- Deduplicate IDs per invocation, preserve first-seen order, and retain
  successful and failed outcomes by requested ID.
- Add deterministic provider-double tests and update architecture/test
  documentation.

## Out of Scope

Native process inspection, process enumeration, listener-to-process joining,
refresh coordination, caching across invocations, parallelism, process
control, Tauri IPC, frontend behavior, project/Git/runtime detection, resource
metrics, and new dependencies.

## Requirements

Implements shared application orchestration required by FR-006 and supports
FR-011's process-ended/error handling. It does not implement native process
inspection or mark FR-006/FR-011 complete. The frozen requirements baseline
remains `THAA-REQ-0.1`.

## Dependencies

W006 process models, W010 `ProcessProvider`, W003 application-layer boundaries,
W004 security/testing requirements, and the integrated W008/W009 platform
listener providers govern this work. Apply Project Skills `rust-patterns` and
`security-and-hardening`.

## Design Notes

`application::process_inspection::inspect_processes` accepts `&dyn
ProcessProvider` and `&[ProcessId]`, returning one `ProcessInspectionOutcome`
per unique requested ID. Each outcome contains the requested ID and either
normalized `ProcessInfo` or its original `ProcessProviderError`. The provider
is borrowed for the call; no `Arc`, global, service locator, or framework is
introduced.

Duplicate IDs are removed with a standard-library set while preserving
first-seen order. Empty input returns an empty vector and makes no provider
calls. Every error category is retained per ID and inspection continues: W010
does not encode provider-wide versus per-process scope, so the use case does
not infer fail-fast behavior. A successful result whose identity PID differs
from the requested PID is converted to `ProviderFailure` for that requested
ID. Partial field availability remains successful `ProcessInfo`.

The flow is synchronous and sequential, has no cross-call cache or retries,
and leaves listener joining and refresh lifecycle to future application
orchestration. PID remains a lookup key, not process-action authorization.

## Acceptance Criteria

- [x] Application-layer use case consumes a borrowed W010 provider and a
      process-ID slice; output associates each result with its requested ID.
- [x] Unique IDs are inspected once in first-seen order; empty input returns
      empty output without provider calls.
- [x] Success, partial metadata, disappearance, permission denial, and
      provider failures remain explicit per-ID results; later IDs are still
      inspected.
- [x] Provider identity mismatch becomes a safe per-ID `ProviderFailure`.
- [x] Deterministic tests cover all success, duplicates, mixed errors,
      partial metadata, empty input, mismatch, and hostile argument data.
- [x] Application code is synchronous, provider/platform-neutral, uncached,
      and has no process-control, refresh, Tauri, or frontend responsibility.
- [x] Architecture/testing docs are accurate; frozen requirements are
      unchanged and no native process functionality is claimed.
- [x] Read-only review completed with no findings; Shared Quality, macOS
      Native Validation, and Windows Native Validation passed on PR #25.
- [ ] Merge, post-merge CI, and clean synchronized `develop` verification
      complete.

## Validation

Local checks passed: Rust format, Clippy (`-D warnings`), Rust tests (24 unit,
3 macOS listener integration, 4 port-contract, 8 W011, and 6 W010 tests),
frontend format/lint/typecheck/tests/build, macOS Tauri no-bundle build,
`pnpm audit --audit-level high`, Markdown links/formatting, and diff integrity.
Local RustSec was not run because `cargo-audit` is not installed; Shared
Quality passed its pinned RustSec audit. PR #25 passed Shared Quality, macOS
Native Validation, and Windows Native Validation (including shared Rust
tests and the Windows Tauri debug build). Tests use provider doubles only for
W011 and do not establish native process inspection.

## Security Considerations

Process names, paths, and command arguments remain inert structured data. The
use case does not log metadata, execute commands, construct URLs, perform
filesystem actions, or treat PID/identity equality as authorization. Errors
remain safe W010 categories. A mismatched successful provider identity is
rejected rather than attributed to the requested PID.

## Platform Considerations

The use case contains no OS-specific behavior and can use either future
native provider without changes. Platform field availability remains inside
W006 `ProcessInfo`; native provider differences are not flattened or inferred
here.

## Documentation Impact

Update application architecture, data flow, directory structure, testing
strategy/catalog, and this work record. Do not alter frozen requirements,
provider contracts, native adapters, or frontend documentation.

## Review Findings

Mandatory read-only review completed with no findings. The reviewer confirmed
the application boundary, per-PID error preservation, deduplication and
ordering, mismatch handling, deterministic tests, and scope.

## Known Limitations

W011 does not collect live process metadata, combine listeners with process
results, or provide refresh snapshots. It has no provider-wide error scope in
W010 and therefore continues after each per-ID error; a later contract change
would be needed to express a reliable provider-wide fail-fast condition.
