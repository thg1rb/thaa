# W026 — Desired-Port Conflict Assistance in Search

Status: In Progress
Requirement: FR-016
Priority: P2
Type: Feature

## Objective

Help a user investigate a desired TCP port by interpreting exact listener
matches in the current accepted Runtime Inspector snapshot. The feature reports
observed evidence and its limits; it does not determine or guarantee whether a
future bind will succeed.

## Background and dependencies

W014 Search filters the current snapshot by process display-name substring or
exact numeric port. W019 explains each observed listener's bind scope. FR-016
also calls for help resolving a desired-port conflict; that remaining slice
must add useful interpretation without duplicating Search or overstating what
the provider observes.

The existing model represents TCP listening endpoints with local port,
optional local address, bind classification, and optional process ownership.
macOS and Windows use their existing providers. Windows Project Root remains
unavailable under the W025 feasibility decision, but port assistance does not
depend on Project Root metadata.

## Scope

- Reuse the existing Search field and accepted listener snapshot.
- For a digits-only query representing a port from 1 through 65535, derive
  exact endpoint matches independently of Search's process-name OR behavior.
- Show the number of exact TCP matches and explain that a match may be relevant
  to a binding conflict but cannot establish that another bind will fail.
- Clarify when the existing Search results also contain process-name matches
  that are not endpoints on the desired port.
- Explain that no observed match does not guarantee availability.
- Qualify results when the snapshot is partial or marked stale.
- Keep Search available when the current snapshot contains no listeners.
- Preserve existing endpoint rows and process actions without changing their
  identity, authorization, or behavior.

## Out of scope

UDP enumeration, address-specific conflict simulation, socket-option
inspection, bound non-listening sockets, port availability guarantees, active
probes or bind attempts, firewall/NAT checks, background polling, provider or
IPC changes, automatic process termination, and new process actions.

## Design and safety

Desired-port assistance is a local, presentation-only derivation over the
accepted snapshot. It matches `entry.port` exactly; it does not infer a match
from Search's visible results because Search also matches process names. All
endpoints remain distinct, including multiple addresses or owners on the same
port. Missing owner/address information remains unavailable.

The current providers observe TCP listeners, not every socket that could
affect a bind. Address-family behavior, socket reuse settings, reserved-port
rules, permission gaps, snapshot age, and concurrent changes can affect bind
results. W026 therefore describes a match as an observation to investigate and
an empty result as “not observed in this snapshot.” It does not probe the
network or imply that wildcard binding proves Internet reachability.

Search remains immediate and local. No command, native scan, process action,
filesystem access, external request, persistence, or new dependency is added.

## Acceptance criteria

| ID    | Criterion                                                                                                                                                               | Validation                           | Status                                                                                                                                                        |
| ----- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| AC-01 | Valid desired-port queries cover 1–65535; empty, invalid, and zero queries do not produce port-availability assistance. Existing W014 query behavior remains unchanged. | Utility and UI tests                 | PASS — 53 focused and 112 full frontend tests                                                                                                                 |
| AC-02 | Assistance counts only exact endpoint-port matches, even when Search also shows process-name matches.                                                                   | Utility and UI tests                 | PASS — exact-port/name-overlap regression tests                                                                                                               |
| AC-03 | Multiple matching endpoints are counted independently; no owner or address is fabricated.                                                                               | Utility and UI tests                 | PASS — multiple endpoint and missing metadata cases                                                                                                           |
| AC-04 | Match and no-match copy never guarantees conflict or availability.                                                                                                      | Component tests and review           | PASS — component assertions and read-only review                                                                                                              |
| AC-05 | Partial and stale snapshots qualify the displayed evidence.                                                                                                             | Component and UI tests               | PASS — component and app tests                                                                                                                                |
| AC-06 | Search remains available for an empty snapshot and existing Search, Refresh, tree, and actions remain unchanged.                                                        | App regression tests                 | PASS — app regression suite                                                                                                                                   |
| AC-07 | macOS and Windows use the same observed-TCP meaning without native provider changes.                                                                                    | Shared tests and required native CI  | PASS — Shared Quality, macOS Native Validation, and Windows Native Validation passed in run 39230145266 on PR head `19973a0b33598211afedc25512856bca2d91f7be` |
| AC-08 | The interface exposes assistance as a labeled, keyboard-readable region without announcing every Search keystroke.                                                      | Testing Library and read-only review | PASS — accessibility assertions and review                                                                                                                    |
| AC-09 | Documentation records semantics and limitations; no probing or automatic process action is introduced.                                                                  | Documentation checks and diff review | PASS — docs checks and scope review                                                                                                                           |

## Validation plan

- Unit-test valid boundaries, exact matching, leading zeros, multiple
  endpoints, process-name-only matches, and snapshot immutability.
- Test match/no-match, partial/stale qualifiers, empty snapshot query, port
  zero, invalid numeric queries, and unchanged process-name Search behavior.
- Verify Search changes do not invoke process actions and refresh uses the new
  accepted snapshot while retaining the query.
- Run frontend formatting, lint, typecheck, tests/build, documentation checks,
  Rust formatting/Clippy/tests, dependency audits, `git diff --check`, and
  native macOS/Windows validation.
- Record interactive macOS and Windows validation separately from CI if
  suitable desktop environments are available.

## Progress

- Requirement review confirmed FR-016 is P2 and W019 covers common bind-scope
  explanations; W026 addresses only the remaining desired-port assistance
  slice. FR-016 remains partially satisfied because this bounded feature does
  not enumerate UDP or all binding conditions.
- The approved product direction for this work is contextual help in existing
  Search and TCP-only evidence from the existing providers.
- Implemented assistance as a local derivation from the accepted snapshot. No
  native provider, IPC, schema, or dependency changes were required.
- Independent read-only review found that numeric process-name matches needed
  an explicit explanation beside port results. The component now labels those
  additional rows and regression tests cover both positive and no-match cases.
- Focused frontend suite: 3 files, 53 tests PASS. Full frontend suite: 12 files,
  112 tests PASS. Lint, typecheck, build, formatting, documentation formatting,
  documentation links, and `git diff --check` PASS.
- Rust formatting, Clippy, and Rust tests PASS. Native macOS integration tests
  PASS locally; Windows runtime behavior is not interactively validated on this
  macOS host. Dependency audits report only previously allowed advisories.
- PR #75 checks PASS on both implementation head `19973a0b33598211afedc25512856bca2d91f7be`
  (run `39230145266`) and the following documentation-evidence head
  `e0a1c4502e5a8d6f8b91c053788b6816c143a383` (run `39235657405`): Shared
  Quality, macOS Native Validation, and Windows Native Validation. The final
  PR head is separately checked before merge.
- Interactive desktop acceptance was not run; platform-native CI remains a
  required PR gate. This feature reuses existing listener snapshot data and
  does not add a platform-specific integration.
