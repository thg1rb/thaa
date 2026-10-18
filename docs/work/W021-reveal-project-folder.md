# W021 — Reveal Project Folder

## Status

Status: Done on merge

Branch: `feature/w021-reveal-project-folder`
Pull request: [#65 — feat: add project folder reveal action](https://github.com/thg1rb/thaa/pull/65)

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
and canonicalizes to the captured project root before asking Tauri's native
opener to open that validated directory in the default file manager.

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
  medium/low/nit findings). PR #65 required CI passed and it merged to
  `develop` at `9d5f7b362108b3fb2adec371be174b951e379426`.

## Validation follow-up (2026-10-18)

The initial interactive attempt above remains recorded as incomplete. A
disposable fixture was then created outside the Thaa checkout at
`/private/tmp/thaa-w021-validation.EvZcgA/project`. Its recognized
`package.json` marker and `nested/` child directory let the provider distinguish
Working Directory from Project Root. A Python standard-library TCP listener
bound only to `127.0.0.1` on ephemeral port `52894`; its PID was `74254`, and
its Working Directory was
`/private/tmp/thaa-w021-validation.EvZcgA/project/nested`.

The exact source build was launched as a separate app bundle named
`Thaa W021 Validation`, avoiding the installed release app and the user's
active application session. It displayed the listener, runtime `Python`, the
expected Working Directory and Project Root, and one accessible
`Reveal project folder for Python` action. The backend validated the current
entry and project root. Finder then opened a window titled `project` containing
`package.json` and `nested`, confirming that the opened directory exactly
matched `/private/tmp/thaa-w021-validation.EvZcgA/project`. The earlier manual
attempt had shown that `reveal_item_in_dir` selects a directory inside its
parent; the W021 requirement is to open the Project Root itself.

The corrective change uses Tauri's native `open_path` API with the
backend-resolved canonical project directory. It retains the opaque entry
reference, current-snapshot resolution, directory validation, and generic
error handling. Paths that cannot be represented as Unicode are refused
without lossy conversion. No shell or project code is executed, and no
frontend-supplied path is trusted.

Interactive checks passed for Project Root visibility and correctness, the
accessible Reveal action on the correct process node, Finder opening the exact
Project Root, retaining the metadata across Refresh, exact-port Search, and
removal of the listener and its action after process exit and another Refresh.
The process was stopped with Ctrl-C after checking. No Stop/Force Stop action
was used. The existing process tree was flat for this fixture, so a child-node
Reveal interaction and a multi-listener process were not manually exercised;
deduplication and entry-reference behavior remain covered by frontend tests.
Windows interactive validation is `NOT RUN`; the documented lack of Windows
working-directory metadata remains unchanged. A deterministic unit test also
verifies Unicode/space preservation and rejection of non-Unicode paths.

### Roadmap review after W021

| Requirement | Priority | Status after W021             | Completed work                                          | Remaining work                                                                                                                     |
| ----------- | -------- | ----------------------------- | ------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| FR-012      | P1       | Partial platform availability | W015 project-root detection                             | Windows provider reports working directory unavailable by design; no safe supported cwd query is currently selected.               |
| FR-013      | P1       | Partial platform availability | W016 Git root and branch                                | Windows working-directory limitation prevents context; do not guess.                                                               |
| FR-014      | P1       | Partial                       | W017 metrics, W018 tree, W021 exact Project Root reveal | Terminal action and preferred IDE/editor action are still explicit product-scope items; Windows root metadata remains unavailable. |
| FR-015      | P2       | Partial                       | W020 runtime-family recognition                         | Reliable framework, package-manager, and version context remains.                                                                  |
| FR-016      | P2       | Partial                       | W019 factual bind-scope explanation                     | Desired-port conflict assistance remains.                                                                                          |
| FR-017      | P3       | Deferred                      | None                                                    | Opt-in watch/favorite/notification and limited container context require separate requirements and safety review.                  |

No unresolved P0/P1 correctness or security blocker was found. The Windows
working-directory gap is an established platform limitation rather than a
failure of the macOS Reveal flow; W011.2 records why invasive process-memory
inspection, shell tooling, or elevation are outside the current design. PR #65
also records the platform's graceful unavailability.

**Proposed next work item (not started): W022 — Open Project in Terminal.**
FR-014 is the highest-priority incomplete requirement, and the product prompt
explicitly includes opening a terminal at a project. This can be bounded to an
explicit action that uses only a Project Root already present in the accepted
snapshot, follows a native platform adapter, and is hidden when no root is
available. It must not execute project code, accept arbitrary frontend paths,
change process identity/actions, or implement editor discovery. On Windows,
the action would remain unavailable until a validated project root exists;
Windows Working Directory discovery should stay a separate prerequisite
decision rather than be silently bundled.

Before implementation, product scope should decide which terminal is meant
(the operating-system default versus a configured terminal) and whether the
action requests a new window or tab. The existing architecture prompt names a
`TerminalIntegration` boundary but does not settle those user-facing choices.
The editor action remains a later FR-014 slice due additional editor-selection
and availability questions. FR-015/FR-016 P2 work and FR-017 P3 work remain
lower priority. This recommendation does not create a W022 branch or change
project status; implementation awaits user approval.
