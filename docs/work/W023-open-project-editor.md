# W023 — Open Project in Editor

## Status

Status: Complete; merged to `develop`

Branch: `feature/w023-open-project-editor`

Base: `develop` at `65c8832c4ec116b575998dc2527f0e8dcfb382f6`

Implementation PR: [#69](https://github.com/thg1rb/thaa/pull/69)

Reviewed head: `e3843f4fe2701cb9f6a503803bd018449f48028d`

Merge commit: `3a0c80e9455e8de857102debc502b9925235de63`

Requirement: FR-014 (project-oriented editor action; P1)

## Objective and approved scope

Add one **Open in Editor** action per eligible Runtime Inspector process node.
The MVP targets Visual Studio Code on macOS and opens the backend-validated
Project Root using VS Code's normal folder/session behavior. The selected
editor is fixed for this work item; there is no editor setting or picker.
Windows Working Directory / Project Root discovery remains unavailable and
is not part of W023. The action is unavailable there.

The user approved this editor strategy, the macOS-first scope ahead of the
separately tracked Windows and Unicode-path gaps, and editor-default window
behavior during planning. If VS Code cannot be resolved or launched, the
action reports a recoverable error without opening a fallback application.

## In scope

- Reuse `RuntimeInspector::project_root_for_entry` for current-snapshot,
  canonical directory, and PID/start-time identity validation.
- Add a thin Tauri command and a macOS adapter that targets the fixed VS Code
  application identity and passes the validated root as path data.
- Add an accessible process-level action and Tauri command permission.
- Cover stale/invalid entries, platform capability, launcher failure, path
  arguments, process grouping, and existing-feature regressions.
- Correct stale W022 closeout/FR-014 traceability statements found during
  roadmap discovery without changing frozen requirement wording.

## Non-goals

- Other editors, editor selection/preferences, application installation or
  discovery UI, Windows Project Root discovery, project-file inspection,
  editor trust changes, project commands, builds, or workspace generation.
- Fixing macOS `lsof` Unicode working-directory decoding or changing W022's
  accepted Terminal window behavior.
- Version, tag, release, or `main` changes.

## Architecture and security

The frontend sends only an opaque accepted-snapshot `entryRef`. The backend
resolves the selected process's own Project Root through the W021/W022
validator, which rechecks PID/start time and the canonical directory before
launch. A fixed application identity and a structured path argument avoid
shell/PATH selection. No project contents are read and no project commands
are executed by Thaa. VS Code extensions, workspace tasks, and shell behavior
remain editor/user controlled; workspace trust is not bypassed.

## Acceptance criteria

- **AC-01:** One accessible action appears for a process node only when the
  snapshot has a Project Root and the build supports the editor adapter.
- **AC-02:** IPC accepts an opaque entry reference only; the backend resolves
  the current entry and revalidates PID/start time and Project Root.
- **AC-03:** On macOS, Visual Studio Code opens the exact validated Project
  Root using its normal folder/session behavior.
- **AC-04:** Stale references, missing/deleted/non-directory roots, missing
  VS Code, and launcher failures fail safely without invalidating the snapshot
  or opening a fallback app.
- **AC-05:** Spaces, quotes, shell metacharacters, leading hyphens, and Unicode
  remain path data; no shell command or project command is launched by Thaa.
- **AC-06:** The action is process-level and preserves Process Tree, Search,
  Refresh, metrics, Project Root, Git, Network Exposure, Runtime Detection,
  Reveal, Terminal, Stop, and Force Stop behavior.
- **AC-07:** Unsupported platforms do not expose a nonfunctional action; the
  Windows build and native validation pass.
- **AC-08:** Disposable interactive macOS validation through the actual Thaa
  UI observes VS Code opening the expected Project Root, with VS Code both
  stopped and already running where the environment safely permits.
- **AC-09:** Work record, architecture/security/test traceability, review, and
  required CI accurately describe the final implementation and evidence.

## Validation plan

Use deterministic backend tests with a mock launcher for valid/stale snapshot
references, PID reuse, root disappearance/type change, unavailable editor, and
launcher failure. Verify the native command uses a fixed executable and
application identity with a structured root argument and no shell. Frontend
tests cover capability/root visibility, one action across multiple listeners,
entry-reference-only invocation, accessible naming, errors, and regressions.
On macOS, use a disposable marker project and loopback listener with Working
Directory nested below Project Root, then activate the real UI action and
verify the editor's opened folder. Windows receives compile/native CI only;
Project Root remains unavailable under the current provider.

## Roadmap and limitations

FR-014 remains partial because the Windows Project Root limitation remains.
W023 completed the macOS editor action; the Windows provider gap remains a
separate platform limitation. FR-015 framework/package-manager/version
work and FR-016 desired-port conflict assistance are P2; FR-017 is P3. The
macOS `lsof` UTF-8 escape limitation can hide Project Root for some Unicode
working directories and remains a separate provider-correctness candidate.
W022's extra default Terminal startup window when Terminal was not running is
an accepted separate UX improvement candidate.

## Progress

- Discovery verified FR-014 P1, W021/W022 reuse, and the current accepted
  `develop` base. W022 is complete; PRs #67 and #68 are merged.
- A read-only planning review found the W023 direction sound and identified
  stale W022 traceability statements corrected in this work item.
- Initial read-only security and implementation reviews found no material
  blockers. The implementation review identified that Launch Services' actual
  missing-bundle diagnostic (`LSCopyApplicationURLsForBundleIdentifier()`)
  was not classified as an unavailable editor. The classifier now recognizes
  that diagnostic, with a regression test; the launcher argument tests also
  cover a leading-hyphen path, the frontend has an unavailable-editor feedback
  test, and the editor test-catalog statuses have been aligned with evidence.
- Read-only security and implementation re-reviews were requested for these
  final corrections and passed with no remaining material findings.
- Implementation and acceptance evidence will be recorded below as work
  proceeds.

## Implementation and validation evidence

The implementation adds the `open_project_in_editor` Tauri command and
main-window capability permission. The command accepts an `entryRef` only and
resolves the Project Root through `RuntimeInspector::project_root_for_entry`,
which enforces current-snapshot membership, process identity, and canonical
directory revalidation. The macOS adapter calls fixed `/usr/bin/open` with
`com.microsoft.VSCode` and the path as a separate argument. It does not use
shell execution or a `code` command lookup. Windows reports no editor
capability and does not render the action.

Automated validation on the development Mac passed: `cargo test` (128 library
unit tests plus the native/integration suites), frontend tests (90 tests),
TypeScript typecheck, frontend build, lint, Rust formatting, documentation
formatting, local documentation-link validation, and `git diff --check`.
Clippy with warnings denied passed. The JavaScript audit found no high-severity
vulnerability. Target-specific RustSec scans exited successfully with five
allowed advisory warnings: unmaintained `proc-macro-error`
(RUSTSEC-2024-0370), `zbus_names` (RUSTSEC-2026-0371), `zvariant`
(RUSTSEC-2026-0370), `zvariant_derive` (RUSTSEC-2026-0372), and the
previously tracked `glib` iterator soundness advisory (RUSTSEC-2024-0429).
W023 adds no dependencies and does not expand into dependency remediation.
The native debug `.app` built with the temporary validation identity
`io.github.thg1rb.thaa.w023validation`; `codesign --verify --deep --strict`
passed. The application version and release configuration were not changed.

### Interactive macOS validation

Environment: macOS 27.0.1 (arm64, build 26A434). The isolated debug app was
`Thaa W023 Validation.app`, bundle identifier
`io.github.thg1rb.thaa.w023validation`. Visual Studio Code 1.142.0 was
initially not running.

For the first scenario, a disposable `package.json` project was created under
`/var/folders/zt/dypcvyd117s7fbxrnkt76vbm0000gn/T/thaa-w023-validation-95ml8sq3/thaa-w023-validation`,
with a loopback Python listener (PID 93819, TCP port 57067) running from its
`nested` subdirectory. The snapshot showed the nested Working Directory and
the canonical Project Root under `/private/var/folders/.../thaa-w023-validation`.
There was one accessible Open project in editor action on the process. The
action was activated in the isolated Thaa UI. VS Code launched and its
Explorer showed the fixture root and its `nested` and `package.json` entries;
the folder was in VS Code Restricted Mode, which remained enabled. This
confirms the selected Project Root opened and that Thaa did not trust the
workspace. No project command was invoked by Thaa.

For the already-running scenario, the listener was replaced with a second
loopback fixture whose Project Root contained spaces, quotes, semicolon,
ampersand, and parentheses. The listener (PID 3016, TCP port 57224) was
detected with the expected nested Working Directory and canonical Project
Root. Activating Open project in editor while VS Code was running left the
Explorer on the literal fixture path. This exercised a real path containing
shell metacharacters through the actual UI and structured argument handoff.
No fallback application or error was observed.

After stopping the second listener and refreshing Thaa, its process row and
editor action disappeared. The first disposable listener was also stopped;
both temporary fixture trees and the listener metadata file were removed.
Windows interactive validation was not run. The existing Windows provider
does not supply Project Root metadata, so the action remains unavailable
there. The separate macOS `lsof` Unicode-working-directory limitation remains
documented and was not altered by W023.

The desktop inventory briefly resolved a generic `Thaa` app request to the
installed application's identity before the uniquely named validation app
was built. No action was clicked in that app and no bundle or settings were
changed. All W023 action validation was performed in the uniquely identified
isolated debug app.

### Acceptance criteria status

- **AC-01 — Action eligibility:** PASS. One process-level accessible action
  appeared with Project Root and macOS capability; no action is rendered when
  root metadata or platform capability is absent (frontend tests).
- **AC-02 — Snapshot validation:** PASS. Backend tests confirm invalid and
  stale references do not reach the launcher; the shared resolver rechecks
  process identity and canonical root.
- **AC-03 — Correct Project Root:** PASS. The actual VS Code Explorer showed
  the exact canonical fixture folder for both normal and special-character
  paths.
- **AC-04 — Safe failure:** PASS for invalid/stale roots and unavailable or
  failed launch mappings in automated tests; editor errors remain recoverable
  and omit private paths.
- **AC-05 — Path safety:** PASS for structured arguments including a leading-
  hyphen path and the interactive special-character path. No shell or project
  command is invoked by Thaa.
- **AC-06 — Existing integration:** PASS in frontend regression tests; process
  grouping, child process association, search context, and Refresh were also
  observed in the live app.
- **AC-07 — Platform boundary:** PASS. The Windows capability is false, the
  action is not shown, and Windows Native Validation passed.
- **AC-08 — Interactive validation:** PASS. VS Code opened the fixture while
  initially stopped and accepted the second fixture while already running.
- **AC-09 — Documentation, review, and CI:** PASS. Documentation, read-only
  reviews, PR CI, and post-merge CI are complete.

### Review and local quality

The initial read-only reviews found two low-severity findings: Launch Services'
observed missing-bundle diagnostic was not classified as an unavailable editor,
and the test catalog had stale planned/in-progress statuses. Both were corrected.
The final read-only security and implementation re-reviews passed with no
remaining material findings.

Local checks passed on the current tree: Rust formatting, Clippy, the full Rust
test suite (128 unit tests plus applicable integration tests), frontend format,
lint, typecheck, tests (92), production build, documentation format/link checks,
`git diff --check`, and the macOS Tauri debug bundle build. The debug app passed
`codesign --verify --deep --strict`. `pnpm audit` found no known vulnerabilities.
Target-specific RustSec audit exited successfully with five previously accepted
warnings: `proc-macro-error`, `zbus_names`, `zvariant`, `zvariant_derive`, and
`glib`; W023 adds no dependencies. Shared Quality and macOS/Windows Native
Validation passed on the final PR head.

### Final closeout

- Security review and implementation review: PASS after resolving the missing-
  editor diagnostic classification and test-catalog status findings.
- PR #69 CI run `39194680201`: Shared Quality, macOS Native Validation, and
  Windows Native Validation all passed on reviewed head `e3843f4`.
- Post-merge CI run `39195127501`: Shared Quality, macOS Native Validation,
  and Windows Native Validation all passed on merge commit
  `3a0c80e9455e8de857102debc502b9925235de63`.
- The feature PR merged to `develop`; no release or `main` change was made.
- W024 was not selected or started.
