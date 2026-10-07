# W016 — Git Context

## Status

Status: Complete; PR #55 was merged to `develop` on 2026-10-07.

Requirement: FR-013 (frozen wording)

## Objective and scope

Show the repository root and current Git branch associated with a listener's
process, using only the process working directory already available in
`ProcessInfo`. This is optional local presentation metadata. It does not add
Git client functionality, project actions, repository status/history, or
global filesystem discovery.

The Git repository root is separate from W015's nearest plausible Project
Root. The two values may differ in nested projects and workspaces.

## Dependencies and platform behavior

- W003 architecture, W004 security baseline, W006 domain models, W010 process
  provider, W013 runtime snapshot/IPC/UI, and W015 project-root enrichment are
  integrated in `develop`.
- The macOS provider supplies process working directories where permitted.
- The Windows provider currently reports working directory as unavailable;
  W016 does not add PEB/WMI/shell/elevation-based discovery.
- Git must be installed and available to the application process. Missing Git
  or an unavailable process directory produces no Git context and does not
  fail the runtime scan.

## Design decision

Use the installed Git CLI through a shared platform adapter, behind a
platform-neutral `GitContextProvider` contract. Invoke only fixed built-in
read-only queries with a structured argument array:

- `git -C <working-directory> rev-parse --show-toplevel`
- `git -C <working-directory> branch --show-current`

Do not invoke a shell, parse repository files, run project code, or inspect
repository status. Clear inherited `GIT_*` environment overrides so discovery
is determined by the observed process working directory. Git handles nested
repositories, `.git` files used by worktrees, symlinks, and branch semantics.
An empty current branch is represented as detached HEAD. Bare repositories
without a working tree do not produce context.

Lookups are deduplicated by exact working-directory path within one runtime
scan. The scan reserves a two-second total budget for optional Git context;
each provider call receives the remaining budget and the Git child is bounded
to the smaller of the remaining time and 750 ms. If the budget expires,
remaining contexts are absent for that snapshot. There is no persistent
cache, so a later refresh can reflect a branch change. Git lookup failure is
local to that process's optional metadata.

## Security and privacy assessment

The working directory and repository metadata are untrusted and may be stale,
inaccessible, deleted, symlinked, Unicode, or contain spaces or shell
metacharacters. Paths are passed as one OS argument and never interpolated
into shell syntax. Only the Git executable is run; project hooks, scripts,
workflows, status checks, and file contents are not requested. Git's
environment overrides are removed. Standard output is bounded and validated
as UTF-8; stderr is discarded and paths are not logged.

Repository root and branch are local, user-visible filesystem metadata and may
reveal private project names. They remain in the local runtime snapshot/UI and
are not uploaded or included in default logs. They do not enter process
identity, action targets, or authorization. PID reuse protections and all
process-action validation remain unchanged.

Potential risks include kernel or remote-filesystem operations that delay
process creation or path access and an untrusted Git executable earlier on the
user's `PATH`. Each Git child is terminated after at most 750 ms and the
optional Git work for one scan is bounded to two seconds. Git is an optional
local dependency and missing/failed queries degrade to no context. No new
dependency or privileged behavior is introduced.

## Acceptance criteria

| ID    | Criterion                                                                                                                                                                        | Verification                               |
| ----- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------ |
| AC-01 | For a process working directory inside a Git working tree, show the Git repository root and current branch.                                                                      | Git fixture and runtime/DTO/UI tests       |
| AC-02 | Resolve nested paths to the nearest Git repository root; support Git worktrees and nested repositories through Git semantics.                                                    | Controlled repository fixtures             |
| AC-03 | Represent detached HEAD explicitly rather than as an empty branch.                                                                                                               | Detached-HEAD fixture and UI test          |
| AC-04 | Non-repository paths, missing Git, missing/inaccessible working directories, and unsupported platform metadata produce no context without hiding listeners or affecting actions. | Failure fixtures and runtime tests         |
| AC-05 | Handle spaces, Unicode, and shell metacharacters without shell interpolation or project-code execution.                                                                          | Adversarial path fixture and code review   |
| AC-06 | Deduplicate lookup by identical working directory within a scan; do not add persistent state or a broad scan.                                                                    | Runtime provider-call test and code review |
| AC-07 | Git metadata remains optional presentation data and is excluded from process identity/action authorization.                                                                      | Domain/runtime tests and review            |
| AC-08 | Frontend displays repository root and branch as secondary metadata, safely handles detached HEAD and untrusted text, and preserves core runtime actions.                         | Frontend tests and review                  |
| AC-09 | Bound optional Git discovery to a two-second budget per runtime scan; expire remaining lookups as absent context.                                                                | Budget contract and provider tests         |

## Non-goals

Git status, diffs, commits, staging, branch switching, remotes, history,
repository discovery outside process working directories, project actions,
Windows working-directory acquisition, and W017 or later work.

## Validation

- Rust: 86 unit tests and all integration test binaries pass, including the
  review follow-up for a per-scan Git budget. Coverage
  includes branch/root lookup, detached HEAD, nested repositories, linked
  worktrees, non-repository/missing paths, unavailable Git, spaces, Unicode,
  newlines, shell metacharacters, and deduplicated scan behavior.
- Frontend: 50 tests, typecheck, lint, and production build passed. UI coverage
  includes named branch, detached HEAD, and hostile display text.
- Formatting, docs formatting/link checks, frontend dependency audit, RustSec
  audits, and macOS debug native build passed. The two previously accepted
  RustSec advisories remain documented.
- Strict Clippy, local Rust tests, and macOS debug native build pass. Final PR
  CI run `37571864268` passed Shared Quality, macOS Native Validation, and
  Windows Native Validation. Post-merge CI run `37572286692` also passed all
  three jobs. Windows Git context is expected to remain unavailable because
  its working-directory provider does not supply the required input; Windows
  runtime behavior is not claimed as locally tested.

Review: read-only W016 review found no remaining material findings on final
head `83070632bdb8f8e8f7072ce23e3eb678439cbd88`. It noted one non-blocking
edge case: an untrusted Git executable could leave a descendant holding the
stdout pipe after the direct child exits, delaying reader completion. Normal
fixed Git built-ins do not spawn descendants; the executable-on-PATH residual
risk remains documented.

## Acceptance evidence

| ID    | Result | Evidence                                                                                                                                                                    |
| ----- | ------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| AC-01 | PASS   | Rust provider fixture and runtime/DTO/UI coverage; local Rust and frontend suites pass.                                                                                     |
| AC-02 | PASS   | Nearest nested repository and linked worktree fixtures pass.                                                                                                                |
| AC-03 | PASS   | Detached-HEAD provider fixture and UI test pass.                                                                                                                            |
| AC-04 | PASS   | Non-repository, missing path/Git, and runtime-row/action preservation tests pass. Windows input remains unavailable by documented capability.                               |
| AC-05 | PASS   | Spaces, Unicode, newline, and shell-metacharacter fixtures pass; direct structured process args, no shell.                                                                  |
| AC-06 | PASS   | Per-scan exact-path deduplication test passes; no persistent cache or broad scan added.                                                                                     |
| AC-07 | PASS   | Git context is optional presentation data; identity/action tests pass and context is not used for authorization.                                                            |
| AC-08 | PASS   | Named/detached rendering and hostile-text sanitization tests pass; typecheck, lint, and build pass.                                                                         |
| AC-09 | PASS   | Scanner passes its decreasing remaining duration to provider calls; provider skips zero budget and bounds each child to the remaining time or 750 ms, whichever is shorter. |

## Progress and evidence

PR #55 merged to `develop` with merge commit
`6bab18e31b6bc8abb1451f09914f2de68c6e2cc9`; the resulting `develop` SHA was
`6bab18e31b6bc8abb1451f09914f2de68c6e2cc9`. Post-merge CI passed. W016 does
not alter application version, `main`, release tags/assets, R001, or the
published `v0.1.1` provenance.
