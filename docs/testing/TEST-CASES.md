# Initial Test and Verification Catalog

**Baseline:** `THAA-REQ-0.1` · **Status:** Planned verification catalog; implementation evidence is recorded per work item

This catalog anchors requirement and threat verification. Per-case status records which deterministic contract checks have run and which provider/platform evidence remains planned; no test status implies a product feature is implemented. Frozen requirement-level cases `TC-001`–`TC-011` remain authoritative in [Functional Requirements](../requirements/FUNCTIONAL-REQUIREMENTS.md). The grouped IDs below complement those IDs and intentionally omit trivial unit tests.

## Existing P0 acceptance cases

| ID            | Requirement   | Future evidence                                                                      | Status                                                                                                 |
| ------------- | ------------- | ------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------ |
| TC-001–TC-004 | FR-001–FR-004 | Native port discovery, normalization/binding, and refresh behavior                   | Not implemented                                                                                        |
| TC-005        | FR-005        | Search by process display name or exact numeric port; filtering is presentation-only | W014 implementation, read-only review, local checks, and initial PR CI passed; user acceptance pending |
| TC-006–TC-007 | FR-006–FR-007 | Process presentation, validated local URL/copy behavior                              | Implemented in W013; prior evidence retained                                                           |
| TC-008–TC-009 | FR-008–FR-009 | Safe graceful and explicit force-stop outcomes                                       | Not implemented                                                                                        |
| TC-010–TC-011 | FR-010–FR-011 | macOS menu-bar/Windows tray and required UI/error states                             | Not implemented                                                                                        |

## Provider and action contracts

| ID                   | Scenario and expected evidence                                                                                                                                                             | Platform / fixture                                         | Links                       | Status                                                                                                               |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------- | --------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| TC-PORT-001          | Controlled random-port TCP listener appears while bound with normalized protocol/address/port, then disappears after close                                                                 | Native macOS and Windows; test-owned socket                | FR-001/002, THR-013         | macOS PASS (W008 IPv4 loopback); Windows NOT RUN                                                                     |
| TC-PORT-002          | PID owner resolves when provider reports it; otherwise owner remains explicitly unresolved                                                                                                 | Native platforms                                           | FR-002, THR-013             | macOS PASS (W008 controlled test process); Windows NOT RUN                                                           |
| TC-PORT-003          | IPv4/IPv6 loopback, wildcard, specific, and unknown inputs classify conservatively                                                                                                         | Pure unit + supported native fixtures                      | FR-003, PR-009, THR-013     | Pure model tests PASS; macOS loopback/wildcard IPv4+IPv6 PASS; Windows NOT RUN                                       |
| TC-PORT-PARSE-001    | Verified NUL-delimited lsof records normalize IPv4/IPv6, family-specific wildcards, PID/file boundaries, and reject malformed required fields                                              | Deterministic parser byte fixtures                         | FR-001/002, THR-013         | Tested in W008                                                                                                       |
| TC-PORT-MAC-001      | Native provider discovers controlled IPv4 loopback listener and PID, then complete scan observes its closure                                                                               | Native macOS, test-owned ephemeral socket                  | FR-001/002, THR-013         | Tested in W008 on macOS 27.0 arm64                                                                                   |
| TC-PORT-MAC-002      | Native provider discovers controlled IPv4/IPv6 wildcard and IPv6 loopback endpoints                                                                                                        | Native macOS where address families are available          | FR-001/003, THR-013         | Tested in W008 on macOS 27.0 arm64                                                                                   |
| TC-PORT-WIN-001      | Native provider discovers controlled IPv4 loopback/wildcard listeners with correct ephemeral port and owner PID                                                                            | Native Windows; test-owned ephemeral sockets               | FR-001/002, THR-013         | Implemented in W009; native CI evidence pending                                                                      |
| TC-PORT-WIN-002      | Native provider discovers controlled IPv6 loopback/wildcard listeners with correct address, port, and owner PID                                                                            | Native Windows where supported                             | FR-001/002/003, THR-013     | Implemented in W009; native CI evidence pending                                                                      |
| TC-PORT-WIN-003      | Controlled listener disappears from a subsequent native scan after its socket closes                                                                                                       | Native Windows; bounded retry using test-owned socket      | FR-001, THR-013             | Implemented in W009; native CI evidence pending                                                                      |
| TC-PROC-001          | Controlled child resolves to expected PID/name and supported metadata                                                                                                                      | Native macOS and Windows                                   | FR-006, THR-001             | macOS PASS (W011.1); Windows PASS (W011.2 controlled child, PR CI 37113239481)                                       |
| TC-PROC-002          | Denied, unsupported, protected, or unavailable metadata has explicit field status; query failure stays distinct                                                                            | Fake and safe native denial where reproducible             | FR-006/011, THR-005/006     | macOS unavailable fields PASS; Windows unsupported fields PASS; protected-process denial NOT RUN                     |
| TC-PROC-003          | Process exit between observation and inspection returns process-disappeared, without crash/fabricated fields                                                                               | Fake + controlled child                                    | FR-011, THR-004             | macOS PASS (W011.1); Windows PASS (W011.2 controlled child, PR CI 37113239481)                                       |
| TC-ACTION-001        | Graceful Stop sends only the supported graceful request to the matching controlled child; Windows generic capability is Unsupported and causes no force action                             | Native controlled child; capability-aware fake             | FR-008, THR-003/005         | macOS SIGTERM passed locally/hosted; Windows Unsupported with child alive (W012.2, PR CI 37123555206)                |
| TC-ACTION-002        | Force Stop is distinct, explicitly confirmed, capability-gated, and never an automatic substitute/escalation                                                                               | Application/command/UI tests + child process               | FR-009, PR-006, THR-003     | macOS SIGKILL passed locally/hosted; Windows same-HANDLE TerminateProcess passed (W012.2, PR CI 37123555206)         |
| TC-ACTION-003        | PID-only/missing identity evidence, PID reuse, changed start time, or changed originally observed executable rejects action and leaves unrelated process untouched                         | Deterministic fake; native controlled child where feasible | FR-009, PR-005, THR-003     | macOS changed-time refusal passed locally/hosted; Windows time/path mismatch leave child alive (PR CI 37123555206)   |
| TC-ACTION-004        | Already-exited target, permission denial, unsupported action, and revalidation failure are reported distinctly; accepted request is not reported as confirmed exit                         | Fake + safe native integration                             | FR-008/009/011, THR-004/005 | macOS already-exited/error mapping passed; Windows already-exited and permission mapping tested; live denial NOT RUN |
| TC-ACTION-POLICY-001 | Action targets require a positive PID and start time; changed/missing required identity evidence fails closed; graceful and force requests and accepted-vs-exited outcomes remain distinct | Deterministic domain tests                                 | FR-008/009, THR-003/004     | Tested in W012.0; no native action performed                                                                         |
| TC-ACTION-WIN-001    | Force Stop revalidates creation time and observed executable path through the same process HANDLE used by TerminateProcess; mismatch refuses and child survives                            | Native Windows test-owned child                            | FR-009, THR-003/004/005     | PASS on `windows-2025`, PR #31 run 37123555206                                                                       |
| TC-ACTION-WIN-002    | Generic Graceful Stop returns Unsupported and does not terminate the controlled child                                                                                                      | Deterministic/native Windows test                          | FR-008, THR-005             | PASS on `windows-2025`, PR #31 run 37123555206                                                                       |
| TC-REFRESH-001       | Rapid/manual/automatic refresh requests do not overlap; repeated requests coalesce and all consumers share coordinator snapshot                                                            | Controlled fake provider/barriers                          | FR-004, THR-004             | Planned                                                                                                              |
| TC-REFRESH-002       | Older scan result cannot replace a newer requested generation; cancellation waits for provider work to settle                                                                              | Controlled fake provider/barriers                          | FR-004, NFR-006             | Planned                                                                                                              |

## Process provider contract semantics (W010)

These deterministic provider-double cases exercise the W010 contract only.
They do not establish that either platform can inspect a live process.

| ID                   | Scenario and expected evidence                                                                           | Fixture            | Links                        | Status         |
| -------------------- | -------------------------------------------------------------------------------------------------------- | ------------------ | ---------------------------- | -------------- |
| TC-PROC-CONTRACT-001 | Requested PID and full normalized identity/metadata are preserved through the provider trait             | Test-only provider | FR-006, THR-001/013          | Tested in W010 |
| TC-PROC-CONTRACT-002 | Field-level unavailable reasons remain distinct and do not become whole-query errors                     | Test-only provider | FR-006/011, PR-004           | Tested in W010 |
| TC-PROC-CONTRACT-003 | Missing/disappeared process and whole-query permission/provider failures remain operation-level outcomes | Test-only provider | FR-011, NFR-005, THR-004/005 | Tested in W010 |
| TC-PROC-CONTRACT-004 | Native paths, structured arguments, and hostile-looking metadata are preserved as inert data             | Test-only provider | FR-006, PR-007, THR-001      | Tested in W010 |

## Shared process inspection semantics (W011)

These deterministic cases validate application orchestration over the W010
contract. They do not establish native process inspection.

| ID                  | Scenario and expected evidence                                                                    | Fixture                   | Links                        | Status         |
| ------------------- | ------------------------------------------------------------------------------------------------- | ------------------------- | ---------------------------- | -------------- |
| TC-PROC-INSPECT-001 | Multiple process IDs produce associated successful per-ID results in first-seen order             | Test-only provider double | FR-006                       | Tested in W011 |
| TC-PROC-INSPECT-002 | Duplicate IDs are inspected once and output retains first-seen unique order                       | Call-counting test double | FR-006, NFR-005              | Tested in W011 |
| TC-PROC-INSPECT-003 | Disappearance, permission denial, and provider errors remain attached to their IDs; later IDs run | Mixed-result test double  | FR-011, NFR-005, THR-004/005 | Tested in W011 |
| TC-PROC-INSPECT-004 | Successful partial metadata remains success; empty input yields no calls/results                  | Test-only provider double | FR-006, FR-011               | Tested in W011 |
| TC-PROC-INSPECT-005 | Mismatched returned identity becomes a provider failure for the requested ID                      | Contract-violating fake   | FR-006, THR-013              | Tested in W011 |
| TC-PROC-INSPECT-006 | Hostile-looking argument values pass through as inert structured metadata                         | Test-only provider double | FR-006, PR-007, THR-001      | Tested in W011 |

## Port provider contract semantics

These deterministic checks establish W007's shared contract behavior only.
They do not establish that either operating system can discover live
listeners.

| ID                   | Scenario and expected evidence                                                                                          | Fixture            | Links               | Status         |
| -------------------- | ----------------------------------------------------------------------------------------------------------------------- | ------------------ | ------------------- | -------------- |
| TC-PORT-CONTRACT-001 | Successful contract call preserves IPv4/IPv6 listeners on the same port and unresolved ownership through a trait object | Test-only provider | FR-002/003, THR-013 | Tested in W007 |
| TC-PORT-CONTRACT-002 | Complete empty result remains distinct from a query-level error                                                         | Test-only provider | FR-001, NFR-005     | Tested in W007 |
| TC-PORT-CONTRACT-003 | Partial result retains usable listeners and a bounded failure category                                                  | Test-only provider | FR-001/002, NFR-005 | Tested in W007 |
| TC-PORT-CONTRACT-004 | Provider error categories are stable and do not contain platform output                                                 | Test-only provider | NFR-003, THR-013    | Tested in W007 |

## Security verification cases

| ID         | Scenario and expected evidence                                                                                                                                                   | Links                    | Status                                        |
| ---------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------ | --------------------------------------------- |
| TC-SEC-001 | Malicious names/arguments with control bytes, terminal escapes, markup, unusual Unicode, quotes, and excessive length remain text, bounded, and visibly truncated as appropriate | THR-001                  | Planned                                       |
| TC-SEC-002 | Shell metacharacters/spaces/quotes never alter a fixed subprocess argument vector; no shell interpolation or project execution occurs                                            | NFR-003, THR-002         | Planned                                       |
| TC-SEC-003 | URL opener rejects arbitrary schemes/hosts, invalid ports, malformed IPv6, and raw command-line input; valid local IPv6 formats correctly                                        | FR-007, THR-007          | Planned                                       |
| TC-SEC-004 | Path-opening fixtures with spaces, quotes, unusual Unicode, deleted paths, and symlinks cannot inject arguments or execute a project                                             | FR-014 later P1, THR-008 | Deferred until path actions are scheduled     |
| TC-SEC-005 | Canary command-line secrets, tokens, and environment values do not appear in default logs/errors/diagnostics                                                                     | NFR-004, THR-010         | Planned when logging exists                   |
| TC-SEC-006 | Invalid native return values/statuses are rejected/mapped safely; handles/resources close on success and error paths                                                             | NFR-005/008/010, THR-009 | Planned when native adapters exist            |
| TC-SEC-007 | Invalid/tampered future config cannot disable identity/privacy controls or trigger unbounded work                                                                                | THR-011                  | Deferred until config exists                  |
| TC-SEC-008 | Malformed, oversized, out-of-range, unknown-enum, or mismatched Tauri arguments are rejected before any provider/action side effect                                              | NFR-003/005, THR-014     | Planned when Tauri boundary exists            |
| TC-SEC-009 | Dependency manifests/lockfiles, licenses, advisories, workflow action pins/permissions, and release artifact controls receive the required supply-chain review                   | NFR-010, THR-012         | Planned for bootstrap/CI/release work         |
| TC-SEC-010 | Permission denial never triggers an elevation prompt/retry; ordinary inspection manifests and launch paths require no administrator/root elevation                               | NFR-002, THR-006         | Planned when app bootstrap/action paths exist |

## Traceability and execution status

W006 adds deterministic Rust unit evidence for the pure-model portions of
`TC-PORT-003` (IPv4/IPv6 binding classification) and `TC-PROC-002` (distinct
field-unavailable reasons and values). The native `TC-PROC-002` scenario is
only partially covered: field-level unavailable outcomes are verified on
macOS, while protected-process denial and Windows behavior remain untested.

W007 tests `TC-PORT-CONTRACT-001`–`TC-PORT-CONTRACT-004` through a
test-only provider implementation. W010 tests
`TC-PROC-CONTRACT-001`–`TC-PROC-CONTRACT-004` through a deterministic process
provider double. W011 tests `TC-PROC-INSPECT-001`–`TC-PROC-INSPECT-006` through
the application use case and deterministic provider double. These contract
and orchestration tests do not establish native process inspection.
W011.1 adds deterministic macOS parser/normalization tests and
controlled non-GUI child evidence for `TC-PROC-001` and `TC-PROC-003`. W011.2
adds controlled Windows child evidence for those cases and verifies
`ProviderLimitation` for Windows arguments and working directory; PR CI run
37113239481 passes both the native provider integration and Windows test
suite. W008 adds parser unit tests and native
macOS evidence for the cases above. W009 adds native Windows listener/PID/
disappearance tests and runs the shared contract suite on the Windows host.
Provider discovery infrastructure is covered on both platforms; frozen
user-facing `TC-001`–`TC-011` receive integrated validation in W013; FR-005
search/filter remains assigned to W014.

- `TC-001`–`TC-011` remain frozen requirement anchors; this catalog does not redefine acceptance criteria.
- Windows portions of `TC-PORT-001`–`TC-PORT-003`, `TC-PROC-*`, and `TC-ACTION-*` have native CI evidence as recorded above. Remaining security cases still require their scheduled fixtures.
- Each implementation PR should reference relevant FR/NFR/PR, case/threat IDs, actual platform result, and limitations.
- No W004 case is reported as `PASS`; cases are `Planned` or `Deferred` until executable tests exist and run.

## Runtime snapshot / UI integration (W013)

| ID             | Scenario and expected evidence                                                                                                               | Fixture                        | Links                       | Status                                                                                                                                   |
| -------------- | -------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------ | --------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| TC-RUNTIME-001 | One snapshot retains distinct listeners, deduplicates owners, preserves unknown owners/process errors, partial scan status, and capabilities | Deterministic providers        | FR-001–004, FR-006, TC-011  | Rust unit tests implemented; local pass, CI pending                                                                                      |
| TC-RUNTIME-002 | Controlled loopback listener appears with correct port and owner process, then disappears from a subsequent snapshot                         | Test-owned TCP listener        | FR-001/002/006, THR-013     | macOS local pass; Windows CI pending                                                                                                     |
| TC-REFRESH-001 | Concurrent initialization shares one scan; repeated refresh coalesces without overlap; superseded snapshot cannot commit                     | Barrier/call-count provider    | FR-004, NFR-005/006         | Sharing, coalescing, and supersession unit tests pass locally; CI pending                                                                |
| TC-IPC-001     | DTO field names/enums remain stable; action uses opaque backend reference and URL resolves only current observed listener                    | DTO fixtures and command tests | FR-007–009, THR-003/007/014 | DTO shape and reference validation tests pass locally; command boundary tests pending                                                    |
| TC-UI-001      | Loading, success, empty, partial, safe error/retry, process metadata failure, and capability-specific controls are observable                | Vitest / Testing Library       | FR-003/004/006/008–011      | Thirteen frontend behavior tests pass locally; CI pending                                                                                |
| TC-UI-002      | Force Stop requires confirmation, requested action is not called exit, and successful/already-exited outcomes request a fresh snapshot       | Mock Tauri IPC                 | FR-009/011, THR-003/004     | Confirmation, requested, mismatch, already-exited, permission, and refresh behavior pass locally; CI pending                             |
| TC-TRAY-001    | Tray menu presents a compact shared snapshot projection and refresh/show/quit behavior                                                       | Native app                     | FR-010                      | Existing macOS tray behavior unchanged; Windows Native Validation passed. Windows interactive tray visual validation NOT RUN / deferred. |

## Process icon enrichment (W013.1)

| ID                  | Scenario and expected evidence                                                                                                                                                            | Fixture                          | Links           | Status                                                                                                                                                         |
| ------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------- | --------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| TC-ICON-001         | Distinct listener rows for one PID share one bounded snapshot icon asset                                                                                                                  | Deterministic providers and PNG  | NFR-001, TC-011 | Rust unit test and Shared/macOS/Windows CI passed in run `37212736265`                                                                                         |
| TC-ICON-002         | Invalid/unavailable icon data leaves runtime rows intact; stale generations return no assets                                                                                              | Deterministic provider / adapter | THR-015         | Failure isolation, stale-generation tests, and Shared/macOS/Windows CI passed in run `37212736265`                                                             |
| TC-ICON-003         | macOS AppKit fallback and Windows Shell/GDI conversion return bounded PNG and release resources                                                                                           | Native CI / controlled process   | THR-015         | macOS fallback and Windows `shell32.dll` conversion passed in native CI run `37212736265`                                                                      |
| TC-W0131-VISUAL-001 | User confirms Menu Bar Light/Dark appearance, no unwanted horizontal trackpad overflow, app-shell selection policy with editable-control exceptions, and equal-width aligned action pairs | User-run macOS validation        | W013.1          | User accepted 2026-10-04; Windows interactive tray visual validation NOT RUN / deferred and nonblocking; Windows Native Validation passed in run `37212736265` |

## Search and filter (W014)

| ID        | Scenario and expected evidence                                                                                                                                                                | Fixture                       | Links              | Status                                                                                                                      |
| --------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------- | ------------------ | --------------------------------------------------------------------------------------------------------------------------- |
| TC-005    | Name matching is trimmed, whitespace-collapsed, case-insensitive substring; numeric port matching is exact and restricted to 0–65535; order/source are preserved; filtering causes no actions | Vitest helper and UI fixtures | FR-005             | Automated tests pass; immediate local filtering has no debounce; manual macOS matching checks are documented in W014 record |
| TC-UI-003 | Search field filters visible cards, clear restores results, no-results differs from no-listeners, refresh preserves query, and filtered rows retain safe actions                              | Vitest / Testing Library      | FR-005, FR-007–009 | Feature, branding, and Search-response tests pass; native visual evidence and review are tracked in W014 record             |
