# Initial Test and Verification Catalog

**Baseline:** `THAA-REQ-0.1` · **Status:** Planned verification catalog; implementation evidence is recorded per work item

This catalog anchors requirement and threat verification. Per-case status records which deterministic contract checks have run and which provider/platform evidence remains planned; no test status implies a product feature is implemented. Frozen requirement-level cases `TC-001`–`TC-011` remain authoritative in [Functional Requirements](../requirements/FUNCTIONAL-REQUIREMENTS.md). The grouped IDs below complement those IDs and intentionally omit trivial unit tests.

## Existing P0 acceptance cases

| ID            | Requirement   | Future evidence                                                          | Status          |
| ------------- | ------------- | ------------------------------------------------------------------------ | --------------- |
| TC-001–TC-004 | FR-001–FR-004 | Native port discovery, normalization/binding, and refresh behavior       | Not implemented |
| TC-005–TC-007 | FR-005–FR-007 | Filter behavior, process presentation, validated local URL/copy behavior | Not implemented |
| TC-008–TC-009 | FR-008–FR-009 | Safe graceful and explicit force-stop outcomes                           | Not implemented |
| TC-010–TC-011 | FR-010–FR-011 | macOS menu-bar/Windows tray and required UI/error states                 | Not implemented |

## Provider and action contracts

| ID                | Scenario and expected evidence                                                                                                                         | Platform / fixture                                         | Links                       | Status                                                                         |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------- | --------------------------- | ------------------------------------------------------------------------------ |
| TC-PORT-001       | Controlled random-port TCP listener appears while bound with normalized protocol/address/port, then disappears after close                             | Native macOS and Windows; test-owned socket                | FR-001/002, THR-013         | macOS PASS (W008 IPv4 loopback); Windows NOT RUN                               |
| TC-PORT-002       | PID owner resolves when provider reports it; otherwise owner remains explicitly unresolved                                                             | Native platforms                                           | FR-002, THR-013             | macOS PASS (W008 controlled test process); Windows NOT RUN                     |
| TC-PORT-003       | IPv4/IPv6 loopback, wildcard, specific, and unknown inputs classify conservatively                                                                     | Pure unit + supported native fixtures                      | FR-003, PR-009, THR-013     | Pure model tests PASS; macOS loopback/wildcard IPv4+IPv6 PASS; Windows NOT RUN |
| TC-PORT-PARSE-001 | Verified NUL-delimited lsof records normalize IPv4/IPv6, family-specific wildcards, PID/file boundaries, and reject malformed required fields          | Deterministic parser byte fixtures                         | FR-001/002, THR-013         | Tested in W008                                                                 |
| TC-PORT-MAC-001   | Native provider discovers controlled IPv4 loopback listener and PID, then complete scan observes its closure                                           | Native macOS, test-owned ephemeral socket                  | FR-001/002, THR-013         | Tested in W008 on macOS 27.0 arm64                                             |
| TC-PORT-MAC-002   | Native provider discovers controlled IPv4/IPv6 wildcard and IPv6 loopback endpoints                                                                    | Native macOS where address families are available          | FR-001/003, THR-013         | Tested in W008 on macOS 27.0 arm64                                             |
| TC-PROC-001       | Controlled child resolves to expected PID/name and supported metadata                                                                                  | Native macOS and Windows                                   | FR-006, THR-001             | Planned                                                                        |
| TC-PROC-002       | Denied, unsupported, protected, or unavailable metadata has explicit field status; query failure stays distinct                                        | Fake and safe native denial where reproducible             | FR-006/011, THR-005/006     | Planned                                                                        |
| TC-PROC-003       | Process exit between observation and inspection returns process-disappeared, without crash/fabricated fields                                           | Fake + controlled child                                    | FR-011, THR-004             | Planned                                                                        |
| TC-ACTION-001     | Explicit graceful stop terminates only the matching controlled child when supported; unsupported/denied results are accurate                           | Native controlled child                                    | FR-008, THR-003/005         | Planned                                                                        |
| TC-ACTION-002     | Force stop is a separate request/confirmation and is never an automatic graceful-stop fallback                                                         | Application/command/UI tests + child process               | FR-009, PR-006, THR-003     | Planned                                                                        |
| TC-ACTION-003     | Identity mismatch, PID reuse simulation, insufficient identity confidence, or changed executable rejects action and leaves unrelated process untouched | Deterministic fake; native controlled child where feasible | FR-009, PR-005, THR-003     | Planned                                                                        |
| TC-ACTION-004     | Already-exited target, permission denial, and revalidation failure are reported distinctly                                                             | Fake + safe native integration                             | FR-008/009/011, THR-004/005 | Planned                                                                        |
| TC-REFRESH-001    | Rapid/manual/automatic refresh requests do not overlap; repeated requests coalesce and all consumers share coordinator snapshot                        | Controlled fake provider/barriers                          | FR-004, THR-004             | Planned                                                                        |
| TC-REFRESH-002    | Older scan result cannot replace a newer requested generation; cancellation waits for provider work to settle                                          | Controlled fake provider/barriers                          | FR-004, NFR-006             | Planned                                                                        |

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
field-unavailable reasons and values). These tests do not complete either
provider/security contract; both catalog cases remain Planned until their
provider and native evidence exists.

W007 tests `TC-PORT-CONTRACT-001`–`TC-PORT-CONTRACT-004` through a
test-only provider implementation. W008 adds parser unit tests and native
macOS evidence for the cases above. The corresponding Windows discovery and
ownership evidence remains NOT RUN and cross-platform parity is incomplete
until W009 runs its native contract tests.

- `TC-001`–`TC-011` remain frozen requirement anchors; this catalog does not redefine acceptance criteria.
- Windows portions of `TC-PORT-001`–`TC-PORT-003`, `TC-PROC-*`, `TC-ACTION-*`, `TC-REFRESH-*`, and `TC-SEC-*` remain planned native contract/security evidence; W007’s deterministic contract cases are recorded above.
- Each implementation PR should reference relevant FR/NFR/PR, case/threat IDs, actual platform result, and limitations.
- No W004 case is reported as `PASS`; cases are `Planned` or `Deferred` until executable tests exist and run.
