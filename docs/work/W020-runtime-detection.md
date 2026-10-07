# W020 — Runtime Detection

Status: IN PROGRESS
Requirement: runtime-family subset of FR-015
Priority: P2
Dependencies: W013 runtime snapshot, W018 Process Tree, W019 Network Exposure

## Objective and scope

Infer a small set of runtime families from the already-observed process name
and present the result as optional process metadata. This work implements only
runtime-family detection from FR-015. Framework, package-manager, and version
detection remain deferred; the frozen FR-015 requirement is not redefined.

Supported runtime families are Node.js, Python, Java/JVM, Ruby, and PHP. The
classifier does not use project files, project root, command arguments,
executable paths, listener ports, or parent process names. Missing or
non-Unicode process names produce no runtime classification.

## Detection rules

- Matching is exact and ASCII case-insensitive. A terminal `.exe` is removed
  before matching to normalize Windows process names.
- Node.js: `node`, `nodejs`.
- Python: `python`, `python2`, `python3`, and `python2.x` / `python3.x` names
  whose version suffix consists only of nonempty numeric dot-separated
  components.
- Java/JVM: `java`, `javaw`.
- Ruby: `ruby`, `rubyw`.
- PHP: `php`, `php-cgi`.
- Everything else is unclassified. Wrappers and near-matches such as `npm`,
  `npx`, `java-wrapper`, `ruby-shim`, and `node-helper` remain unknown.

The classifier is pure and runs once for each inspected process in the current
snapshot. Its optional value is carried on listener entries and shown once at
the process-tree node (or on the first listener card for a leaf). Runtime
metadata does not change process identity, actions, metrics, tree relationships,
search matching, project/Git context, or listener exposure.

## Platform, privacy, and security

Both platforms use the existing observed process name; no provider calls or
platform-specific behavior are added. The macOS provider may return a
truncated kernel command name, so runtime detection can have false negatives.
An executable can also deliberately use a supported name, so this is
convenience metadata, not proof of runtime, source language, or trust.

No command line, executable path, project file, or filesystem tree is read.
Thaa does not execute inspected binaries, shell commands, or project code, and
does not elevate privileges. A missing classification does not affect the
listener, metrics, actions, or other metadata.

## Acceptance criteria

| ID    | Criterion                                                                                                                 | Evidence                                         | Status  |
| ----- | ------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------ | ------- |
| AC-01 | Supported runtime names and aliases classify deterministically.                                                           | Domain classifier cases                          | PENDING |
| AC-02 | Near-matches, wrappers, missing names, and non-Unicode names remain unclassified.                                         | Negative and unavailable-name tests              | PENDING |
| AC-03 | Runtime is optional snapshot presentation metadata, serialized with a stable closed set of values.                        | Snapshot and DTO tests                           | PENDING |
| AC-04 | Runtime is shown once per process and remains distinct for parent/child processes and multiple listeners.                 | Process-forest and UI tests                      | PENDING |
| AC-05 | Search, actions, process identity, metrics, project/Git context, tree, and exposure behavior remain unchanged.            | Existing and focused regression suites           | PENDING |
| AC-06 | No process-provider calls, filesystem/project scan, shell, inspected-binary execution, elevation, or dependency is added. | Diff review and provider/action regression tests | PENDING |
| AC-07 | Documentation describes the evidence boundary, limitations, and deferred FR-015 scope.                                    | Documentation checks and review                  | PENDING |

## Validation and known limitations

Automated tests cover the supported names, aliases, case normalization,
Windows `.exe` normalization, Python numeric version suffixes, false-positive
near-matches, unknown names, DTO values, and tree/UI integration. Shared,
macOS Native, and Windows Native CI validate the final reviewed PR head.
Interactive runtime validation is recorded only if actually performed.

The classifier can miss version-manager shims, truncated macOS names,
versioned Ruby/PHP executables, and Python free-threaded names with a `t`
suffix. It can classify a deliberately misleading process name. It does not
detect frameworks, package managers, or runtime versions and must not be used
for security decisions.

## Progress

- Discovery verified FR-015's exact wording and P2 priority. W020 delivers a
  bounded runtime-only slice; framework, package-manager, and version
  detection remain deferred.
- Implemented a pure exact-name classifier and optional snapshot/DTO metadata;
  no provider calls, dependencies, path reads, or subprocesses were added.
- Local validation: 104 Rust unit tests and all native integration tests pass;
  75 frontend tests pass; TypeScript, lint, frontend production build, Rust
  format, Clippy, docs format/link checks, `git diff --check`, and the macOS
  Tauri debug no-bundle build pass. pnpm audit reports no known
  vulnerabilities. RustSec reports only the two already documented allowed
  warnings (`RUSTSEC-2024-0370`, `RUSTSEC-2024-0429`).
- Interactive runtime/UI validation is NOT RUN. The existing desktop Thaa
  instance was left untouched, and the local browser automation surface was
  unavailable. Windows native validation remains pending CI.
- Read-only review, required PR CI, merge, and post-merge verification remain
  pending.
