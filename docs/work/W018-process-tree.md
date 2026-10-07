# W018 — Process Tree

Status: READY FOR REVIEW
Requirement: remaining parent process/tree portion of FR-014
Priority: P1

## Objective and scope

Expose current parent/child context for processes already represented by a
listener in the accepted Runtime Inspector snapshot. Each process identity is
one node; multiple listeners stay grouped under that process. The UI presents
a collapsible forest. Search retains matching process/port rows and their
ancestor nodes as context; contextual ancestors do not become matches.

Non-goals: enumerating non-listening relatives, system-wide process management,
recursive Stop/Force Stop, subtree metrics, history, charts, sorting, process
debugging, or W019 Network Exposure.

## Decisions

- Platform adapters return an optional parent PID; application/domain code
  remains OS-independent. The accepted runtime scan is the consistency
  boundary; superseded scans cannot commit their tree.
- IPC remains flat. The frontend derives the forest from the accepted flat
  entries. Missing parents, unavailable metadata, self-links, cycles, and
  ambiguous duplicate identities produce roots rather than fabricated edges.
- Identity is scoped to the snapshot and strengthened with the process start
  time already returned by providers. A relationship is omitted if either
  endpoint lacks start-time evidence or the alleged parent started later than
  its child, which is evidence of PID reuse. Parent links are informational
  and never authorize process actions.
- macOS obtains parent PID with the existing public `sysctl(KERN_PROC_PID)`
  process snapshot. Windows uses one Tool Help process snapshot per batch of
  listener-owner inspections. No elevation or shell invocation is added.
- Expansion defaults open, persists only for the same process identity across
  accepted refreshes, and disappears with that identity. Search semantics stay
  process-name substring or exact numeric port.

## Security and performance

Process metadata is untrusted. The tree builder is bounded by the finite set of
unique listener-owner processes, deduplicates by PID, rejects cycles and
self-links, and does not recurse through the OS process table. Windows native
snapshot handles are closed deterministically. Per-process lookup failures do
not fail the listener scan. The hierarchy does not change action references,
PID checks, or W017 metric sampling.

## Acceptance criteria

| ID    | Criterion                                                                                                                    | Evidence                                 |
| ----- | ---------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------- |
| AC-01 | Parent PID is optional metadata obtained behind macOS/Windows adapters without elevation or shell commands.                  | Provider tests and native CI             |
| AC-02 | One node represents each unique listener-owner process; multiple listeners remain attached to that node.                     | Tree derivation tests                    |
| AC-03 | Missing parents, invalid/self/cyclic edges, and process churn degrade to safe forest roots without scan failure.             | Tree/provider tests                      |
| AC-04 | The UI displays a collapsible forest and preserves expansion only for stable process identities across refresh.              | Component tests and manual UI validation |
| AC-05 | Search keeps existing process-name and exact-port matching; matching descendants may show ancestors as non-matching context. | Search/tree tests                        |
| AC-06 | Stop and Force Stop continue to target only the explicitly selected process.                                                 | Action-target regression tests           |
| AC-07 | W017 metrics, ports, icons, Git context, and refresh generation semantics remain intact.                                     | Regression tests and CI                  |
| AC-08 | Documentation and platform limitations match implementation; no W019 or release work is included.                            | Documentation checks and diff review     |

## Validation plan

Run Rust formatting, Clippy and tests; frontend formatting, lint, typecheck,
tests and build; documentation checks; dependency/security checks; macOS and
Windows native CI; and `git diff --check`. Manually inspect a safe macOS
parent/child listener family if available. Interactive Windows validation is
reported separately from CI.

## Manual validation

- macOS local debug app: a disposable parent process and child process each
  opened a loopback listener. Runtime Inspector showed the child under its
  parent; collapsing the parent hid the child; searching for the child's exact
  port retained the parent as context; Refresh preserved the query; after the
  child exited, Refresh removed its listener and hierarchy result. PASS.
- Interactive Windows validation: NOT RUN; Windows Native Validation covers
  provider compilation/tests and packaging in CI.
- No release candidate or published release artifact was generated or changed.

## Progress

- Discovery and scope freeze complete.
- Implemented optional parent-PID metadata in the shared process contract; macOS
  reads it with its existing sysctl process snapshot, and Windows uses one
  Tool Help snapshot per owner batch with a deterministic handle wrapper.
- Implemented flat IPC metadata and iterative frontend forest derivation,
  process grouping, context-only search ancestors, and accessible disclosure.
- Automated forest, search, action-target, and macOS current-process parent
  coverage added.
- Local formatting, lint, typecheck, frontend tests/build, Rust tests/Clippy,
  and documentation checks pass. Shared Quality, macOS Native Validation, and
  Windows Native Validation pass on PR CI. Local Windows cross-target
  validation could not run because this host lacks `llvm-rc`.
- The disposable macOS parent/child listener UI scenario passes. Read-only
  review found no material findings.
