# W021 — Reveal Project Folder

## Status

Status: Ready for review

Branch: `feature/w021-reveal-project-folder`

Base: `develop` at `c84901689eac8f399fc64c71229ee7e3465273c6`

Requirement: FR-014 (project-folder reveal subset; P1)

The isolated worktree is used because the original checkout contains three
user-owned, untracked README screenshots. Those files were intentionally
excluded. No user-owned untracked file was modified, copied, moved, staged, or
committed. The feature branch was created from the verified accepted
`origin/develop` SHA above.

## Roadmap decision

The post-W020 requirement review found no newer numbered work item and no
P0/P1 correctness or security blocker. FR-014 is the highest-priority
unfinished implementation requirement. W017 completed optional CPU, memory,
and uptime metadata; W018 completed snapshot-scoped process-tree context. The
remaining FR-014 project-oriented actions include folder reveal, terminal
launch, and editor launch. This work selects only folder reveal because W015
already provides an optional project root and THR-008 / TC-SEC-004 already
specify the path-opening security boundary.

Other considered candidates remain lower priority: W020 covers only runtime
families in FR-015 (framework, package-manager, and version details remain P2
deferrals); W019 covers binding explanations in FR-016 (desired-port conflict
assistance remains P2); FR-017 watch/favorite behavior is P3 and requires
separate requirements. Windows interactive and narrow-layout checks remain
validation debt, not implementation scope. Developer ID/notarization and
Authenticode remain intentionally deferred release hardening. R001 tester
validation remains independent.

## Objective and scope

Provide an explicit Runtime Inspector action that reveals the process's
existing, snapshot-derived Project Root in the operating-system file manager.
The action is available only when that metadata exists. The current Windows
provider does not provide working-directory metadata, so the action remains
unavailable there without guessing or adding another process-inspection path.

The action appears at most once per represented process tree node, even when
the process owns several listener rows. It sends a snapshot-scoped opaque entry
reference to the backend, which resolves the project root from the accepted
snapshot. The backend verifies the directory still exists, is a directory,
and canonicalizes to the captured project root before calling Tauri's native
`reveal_item_in_dir` API.

## Non-goals

- Terminal/editor launch, project-file parsing, or executable project code.
- New Windows working-directory discovery or privilege elevation.
- Shell commands, raw frontend paths, or process-action changes.
- Changes to process identity, Search, Refresh, tree construction, listener
  grouping, metrics, Git Context, Network Exposure, or runtime detection.
- New dependencies, release changes, `main` promotion, or W022 work.

## Architecture and security

The UI sends only an `entryRef`. The application layer resolves it against the
current accepted snapshot and returns only the stored project root after
canonical-path and directory checks. The Tauri command uses the existing
opener plugin's native reveal API; it does not construct shell text or execute
project content. Errors are generic and do not expose filesystem paths. Root
resolution runs on a blocking worker. The operation is explicit and uses the
current app privileges.

The directory may change after validation and before the OS file manager acts;
this filesystem TOCTOU condition remains a documented residual risk. If the
path disappears, ceases to be a directory, or resolves through a changed
symlink target, the request fails safely and the user can refresh.

## Acceptance criteria

- **AC-01:** A reveal action is available only for a current snapshot entry with
  a project root; missing metadata does not affect listener visibility or
  process actions.
- **AC-02:** The frontend sends an opaque entry reference, and the backend
  resolves only the current snapshot's project root and verifies its canonical
  directory before reveal.
- **AC-03:** Stale/malformed references, absent/deleted/non-directory roots,
  and changed symlink targets fail safely without disclosing the path.
- **AC-04:** Paths containing spaces, quotes, Unicode, or control characters
  remain inert; no shell, arbitrary project execution, external service, or
  privilege elevation is used.
- **AC-05:** Each process tree node exposes at most one accessible reveal
  action across its listener rows; Stop/Force Stop targets and existing
  metadata behavior remain unchanged.
- **AC-06:** Relevant tests, documentation checks, read-only review, required
  PR CI, merge to `develop`, and post-merge verification pass.

## Validation plan

- Rust tests for current/stale references, present/absent roots, deleted roots,
  symlink redirection where supported, and unusual path characters.
- Frontend tests for action visibility, per-process deduplication, accessible
  labeling, and entry-reference-only invocation.
- Full shared, Rust, frontend, security/dependency, documentation, and native
  CI gates required by the repository.
- Interactive macOS validation with a disposable project folder where the
  environment permits it. Windows interactive validation is recorded as
  `NOT RUN`; Windows Native Validation verifies graceful unavailability.

## Progress and evidence

- Roadmap discovery confirmed FR-014 is P1 and remains partial. W021 is limited
  to project-folder reveal; terminal and editor actions remain deferred.
- The isolated feature worktree started clean at the recorded `develop` base.
- Interactive macOS validation used disposable local HTTP listeners. The
  application detected both listeners, but the native provider returned no
  project-root metadata for those processes, so the reveal action was not
  available to exercise in the live UI. The existing user checkout path was
  not opened or used. Interactive Windows validation is `NOT RUN`.
- Rust filesystem/application tests and frontend action tests cover available
  and unavailable project roots, stale references, symlink redirection,
  unusual path characters, action visibility, and per-process deduplication.
- Local validation passed: 108 Rust unit tests and 30 native/integration tests;
  77 frontend tests; Rust format, Clippy with warnings denied, frontend format,
  lint, typecheck, production build, docs format/link checks, `git diff
--check`, and a macOS native debug build. `pnpm audit --audit-level high`
  reported no known vulnerabilities. RustSec passed under repository policy
  with five allowed advisories: `proc-macro-error`, `glib`, `zbus_names`,
  `zvariant`, and `zvariant_derive`; no dependency changed for W021.
- Dedicated read-only review found no material findings (zero blocker/high/
  medium/low/nit findings). PR CI is still pending.
- Final status will be `Done on merge` with the PR link, following the agent
  workflow.
