# Testing Strategy

**Applies to:** `THAA-REQ-0.1` and W003 architecture · **Status:** Initial strategy; W005 bootstrap checks are established

## Purpose and evidence

Tests provide evidence that behavior, safety rules, provider contracts, and platform differences meet requirements. Prefer observable behavior and stable contracts over implementation-detail coverage, fragile snapshots, or arbitrary coverage percentages. Every bug fix should add regression coverage when practical.

Tests never target arbitrary existing user/system processes. Integration tests create only controlled resources and clean them up. Test data uses neutral names/paths and no private endpoints or secrets. The requirements catalog retains existing `TC-001`–`TC-011`; W004 introduces grouped identifiers only for material contract/security scenarios in [TEST-CASES.md](TEST-CASES.md).

## Layers and ownership

| Layer                   | Primary target                                                                       | Expected evidence                                                                                                        | Typical execution                             |
| ----------------------- | ------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------- |
| Unit                    | Pure normalization/classification, identity comparisons, errors, parsing, validation | Deterministic table/fixture tests without live OS state                                                                  | Shared CI and local                           |
| Provider contract       | `PortProvider`, `ProcessProvider`, `ProcessController` behavior                      | Deterministic doubles for shared semantics; same contract suite against each native implementation; differences explicit | Shared CI and native macOS/Windows runners    |
| Integration             | Real listener/process and OS adapter lifecycle                                       | Controlled listener/child process, permission/error cases, cleanup                                                       | Native OS runner/machine                      |
| Tauri boundary          | Input validation, DTO/enum serialization, error mapping                              | Invalid input rejected and transport representation stable/safe                                                          | Rust boundary tests; exact framework deferred |
| Frontend                | User-visible states/interactions/accessibility                                       | Behavior tests for loading/data/errors/actions/keyboard, limited snapshots                                               | Frontend runner selected at bootstrap         |
| Reliability/performance | Repeated refresh and resource use                                                    | Recorded environment, scenario, measurements, failures/limitations                                                       | Native systems before release claims          |

Pure behavior belongs below Tauri. Commands should have little direct logic and only boundary-focused tests. Frontend tests must not mock away every behavior; use component/contract-level data to prove user-visible states while backend behavior is separately tested.

## Unit tests

Keep testable logic independent of live OS state. Cover address/binding classification, listener and process normalization, identity comparison, field availability/capability mapping, input validation, error-to-transport mapping, and parsers for any isolated platform output. Project/runtime detection tests apply only when those later requirements are scheduled; do not implement them in W004.

Use deterministic input tables for IPv4/IPv6 loopback, wildcard, specific non-loopback, and unknown values. Explicitly test unavailable/permission-denied/unsupported/disappeared/provider-failure distinctions. No coverage percentage is imposed.

## Provider contracts and native integration

W007 and W010 contract tests use deterministic provider doubles to verify shared port/process semantics without live OS access. Future native implementations must run equivalent behavior assertions on macOS and Windows. Process-provider semantics include requested PID preservation, normalized `ProcessInfo`, explicit per-field unavailability, and query-level disappearance/failure. Native process integration will use only test-created child processes and must cover identity, supported metadata, and unavailable outcomes. Controller contracts use only a test-created child process and cover graceful/force stop, mismatch, disappearance, permission denial, unsupported action, and revalidation failure.

W011 application tests use a deterministic `ProcessProvider` double to verify
first-seen deduplication, per-PID success/error preservation, continued
inspection after errors, mismatched identity handling, partial metadata, and
empty input. They do not invoke native process APIs or establish live process
inspection.

W012.0 unit tests validate identity-bound action targets and stable outcomes
with no destructive OS calls. Its macOS native provider regression test also
queries a test-owned child through the SDK-backed `KERN_PROC_PID` start-time
path; it does not signal that child. Actual controller/action tests remain
future work and must use controlled child processes only.

Mocks/fakes establish application orchestration deterministically but do not replace native integration. Compilation alone is not platform validation. Do not claim parity where an OS cannot provide a capability; document and test the supported/unavailable result.

## Tauri and frontend

Tauri-boundary tests validate inputs, DTO optional/unavailable fields, enum serialization, and safe error mapping. Internal stacks/native text must not appear in transport errors. Do not duplicate use-case tests inside thin commands.

Frontend behavior coverage includes loading, empty, permission-denied, unavailable metadata, process-ended, provider error, filtering/search, refresh, copy/open actions, destructive confirmation, graceful stop result and explicit force-stop path. Test accessible names, keyboard operation, focus/error announcement, and platform conventions. Prefer behavior assertions; avoid broad brittle snapshots. Choose test framework at bootstrap, not W004.

## Concurrency, race, reliability, and performance

Test W003's observable refresh invariants: no overlap, coalesced requests, one canonical coordinator shared by consumers, cancellation where practical, older results rejected after a newer generation, and provider failure preserving a truthful snapshot/error state. Use a controlled fake provider with barriers/latches instead of timing sleeps.

Race cases include listener close, process disappearance, PID/identity mismatch, ownership change, metadata permission change, and action target disappearance between observation and action. Deterministic fakes test exact sequencing; native tests use only created resources and report limitations when OS timing cannot be made deterministic.

Before performance claims, collect idle CPU/memory, refresh duration, resource growth, subprocess frequency, UI responsiveness, and process/handle leakage over repeated refreshes on both target OSes. Record versions, scenario, counts, cadence, duration, measurements, and limitations. No numeric budget or soak duration is invented here.

## Determinism, isolation, and flake policy

- Use controlled children/listeners, ephemeral ports, explicit readiness, bounded waits, and cleanup on every exit path.
- Avoid fixed ports, developer-specific absolute paths, unrelated running processes, long blind sleeps, privileged fixtures, and machine-specific state.
- Tests operate only on resources they create; action tests terminate only their own child process.
- Investigate flaky tests at the root cause. Do not mask failures with indefinite retries. Temporary quarantine requires an owner, reason, tracking issue/work item, and removal condition; otherwise keep the failure blocking.

## Security and regression evidence

Use the security adversarial fixtures in [TEST-CASES.md](TEST-CASES.md): hostile strings, shell metacharacters, paths, URL inputs, stale/mismatched identities, permission outcomes, log canaries, and invalid native results. Threat-to-test links are normative targets for future implementation, not executed W004 tests.

Each defect report/work item links a regression test when practical. If no automated test can safely reproduce it, document the manual/platform verification and limitation.

## Quality gates and merge policy

Required gate categories when their code exists/configuration is available:

- **Rust:** format check, lint/static analysis, unit tests, provider/integration tests.
- **Frontend:** formatting, lint, typecheck, behavior tests, build.
- **Security/dependencies:** ecosystem advisory audit, dependency/license review, secret scan, security review triggers.
- **Documentation:** Markdown/link/reference validation.
- **Platforms:** native contract/integration and app build on macOS and Windows; one platform's pass never implies the other's.

W005 selects Vitest 4 with jsdom and Testing Library for frontend behavior, Tauri's official mock IPC API for controlled command responses, and Rust's built-in test harness for DTO serialization. Prettier and ESLint 10 provide frontend/config formatting and linting; rustfmt and Clippy provide Rust checks. The Rust/TypeScript smoke DTO is mirrored manually and guarded by the Rust serialized-shape assertion plus frontend response fixture; generated bindings remain deferred until real contract breadth justifies them.

Build, required test, native contract, typecheck, lint/static, or Critical/High security failure blocks merge when applicable and configured as required. A missing required platform test is not a pass. Medium findings block if a required safety/acceptance condition remains open; other issues require disposition.

Report every check as `PASS`, `FAIL`, `BLOCKED`, `NOT APPLICABLE`, or `NOT RUN`. For platform checks record platform/environment, test class, result, and limitation. Never call `NOT RUN` a pass.

## CI and review relationship

W008.1 establishes shared GitHub Actions checks on Ubuntu 24.04, macOS native
validation on macOS 15 arm64, and Windows validation on Windows Server 2025
x64. Shared CI runs frontend formatting, lint/typecheck, tests/build,
documentation links, and dependency audits. Native runners run Rust
format/Clippy/tests and a Tauri build; macOS tests include the W008 controlled
listener integration. Windows currently validates shared contracts and the
application build only; W009 must add and execute Windows native provider
tests. See [CI](../development/CI.md) and [Platform Testing](PLATFORM-TESTING.md)
for the actual runner/evidence boundaries. Automated checks supplement, never
replace, the mandatory read-only Sub-agent PR review. Branch protection is not
configured yet, so PR reviewers must verify the relevant workflow run directly.
