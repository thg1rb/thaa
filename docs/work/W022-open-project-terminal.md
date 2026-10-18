# W022 — Open Project in Terminal

## Status

Status: In progress

Branch: `feature/w022-open-project-terminal`

Base: `develop` at `88731c88e665f77b317e3bc2cfddbf407bb89bee`

Requirement: FR-014 (project-oriented actions; P1)

The isolated worktree protects the original checkout, which contains three
user-owned untracked README screenshots. They were intentionally excluded; no
user-owned untracked file was modified, moved, staged, copied, or committed.
The W022 worktree started clean from the verified accepted `origin/develop`
SHA above.

## Objective and approved scope

Add one explicit **Open in Terminal** action per eligible process node. When
the current accepted snapshot contains a validated Project Root for the
selected process, the action asks Apple Terminal.app to create a new window
whose shell starts in that directory.

Approved product decisions: macOS with Apple Terminal.app; always request a
new window; use the Project Root; do not run project commands; no terminal
configuration UI. Windows Working Directory / Project Root discovery, editor
integration, custom terminal selection, and W023 are excluded.

## Architecture and safety

The frontend sends only the opaque snapshot `entryRef`. The backend resolves
that reference through `RuntimeInspector::project_root_for_entry`, which
validates the current generation, stored root, canonical directory, and
snapshot consistency. It also re-inspects the selected process at action time
and requires the same PID and start time, failing closed when identity evidence
is missing or changed. This prevents a stale snapshot from surviving PID reuse
until Refresh. The existing W021 boundary is reused; no frontend path is
accepted. A blocking worker performs validation and launches a
fixed `/usr/bin/osascript` executable with a constant AppleScript program and
the validated UTF-8 path as a separate argument. AppleScript asks Terminal.app
to make a new window, then runs only a quoted `cd -- <path> || exit` command.
Thaa does not intentionally execute project scripts or commands. The shell's
own startup files remain outside Thaa's control.

`NSAppleEventsUsageDescription` explains the explicit action. macOS TCC
authorization is respected; denial is reported as a generic recoverable
error. No broad permission, privilege escalation, external call, additional
scan, persistent cache, or dependency is introduced. Windows returns an
unsupported result and the UI action is absent when no Project Root exists.

The path is data in an argv slot, then quoted using AppleScript's standard
shell quoting. Non-UTF-8 paths are rejected instead of lossily converted.
Output streams are discarded to avoid path disclosure. The normal filesystem
time-of-check/time-of-use race between canonical validation and Terminal's
later directory change remains bounded and documented.

## Acceptance criteria

- **AC-01:** Action appears once per process node only when Project Root is
  present in an accepted snapshot.
- **AC-02:** IPC accepts only the opaque entry reference and backend resolves
  the current snapshot's Project Root.
- **AC-03:** A new Apple Terminal.app window starts in the validated Project
  Root; an existing Terminal session is not replaced.
- **AC-04:** Stale entries, missing/non-directory roots, unsupported paths,
  automation denial, and launcher errors fail safely.
- **AC-05:** Special characters remain literal; Thaa does not execute project
  commands, interpolate an untrusted path into the AppleScript source, or
  invoke a shell wrapper.
- **AC-06:** Process Tree, Search, Refresh, metadata, and process-action
  behavior remain unchanged; Windows does not expose a nonfunctional action.
- **AC-07:** Automated tests, documentation, read-only reviews, required CI,
  and post-merge verification pass.
- **AC-08:** Interactive macOS validation observes a new window and verifies
  its actual `pwd`, both when Terminal is already running and when it is not.

## Validation plan and evidence

Automated tests cover the snapshot resolver and action-time process identity,
that invalid/unavailable entry resolution does not reach the launcher, the
backend-resolved root is the only path passed to the launcher, reference-only
IPC, one-action-per-process UI, AppleScript's static structure, and path quoting
for spaces, apostrophe, quotes, metacharacters, newline, Unicode, and leading
hyphen. The exact macOS Terminal behavior must additionally be checked
interactively; pure quoting tests do not establish that GUI behavior.

### Current interactive evidence (2026-10-18)

The debug bundle `Thaa W022 Validation.app` was built with the W022 source and
isolated bundle identifier `io.github.thg1rb.thaa.w022validation`. A loopback
Python listener (PID `7828`, port `53501`) ran from a nested directory under a
disposable root containing a valid `package.json`. The path contained spaces,
single quotes, and a harmless `$(touch /private/tmp/thaa-w022-INJECTED)`
sentinel expression. After Refresh, the application showed Working Directory
`/private/tmp/thaa-w022 validation $(touch /private/tmp/thaa-w022-INJECTED) 'quoted'/nested`,
the expected Project Root ending in ` 'quoted'`, and one accessible Open in
Terminal action for the Python process. This distinguishes Working Directory
from Project Root and confirms the action is attached to the correct process.

Clicking the real UI action returned the generic recoverable error. No
Terminal window was opened by that action; the shell-injection sentinel was
not created. Terminal.app was initially not running. A separate diagnostic
invocation then launched Terminal.app but remained waiting for Apple Events;
it was terminated without using it as feature-validation evidence. It also
did not create the sentinel. The app-to-Terminal workflow therefore has **not
passed** interactive validation: a new window and its actual `pwd` were not
observed, and the already-running case remains pending. The isolated test app
is entitled and includes `NSAppleEventsUsageDescription`; the remaining
permission/integration failure is being investigated. A Unicode-containing
fixture path could not provide Project Root metadata because the existing
macOS `lsof` provider escaped its UTF-8 bytes and correctly marked that path
unavailable; the pure AppleScript quoting tests still include Unicode.

The disposable listener and its tree node remain available for retry after
the macOS automation authorization path is resolved. No Stop or Force Stop
was used.

### Automated validation and review update

Rust tests pass (117 unit tests plus the platform integration suites), Clippy
passes with warnings denied, and frontend tests pass (84 tests). Frontend
formatting, lint, TypeScript, production build, documentation formatting/link
checks, and `git diff --check` pass. `pnpm audit --audit-level high` reports no
known vulnerabilities. `cargo audit` exits successfully with five warnings:
the previously recorded `proc-macro-error` unmaintained and `glib` iterator
soundness advisories, plus current unmaintained advisories for `zbus_names`,
`zvariant`, and `zvariant_derive` (published 2026-10-16). W022 adds no
dependencies and does not change these advisories.

Read-only security review and implementation review found no remaining
material code findings after adding action-time PID/start-time revalidation and
tests proving stale or unavailable snapshot entries do not reach the launcher.
Both reviewers agree that the failed real UI attempt remains a merge-blocking
validation gap until a new Terminal window and its exact `pwd` are observed.

## Platform and known limits

- macOS: Apple Terminal.app only; TCC automation permission may be required.
- Windows: no Project Root is currently provided by its process provider, so
  this action remains unavailable. No Windows shell integration is included.
- A terminal shell may execute its own configured startup files. Thaa does
  not invoke project-specific commands.
- Project roots that cannot be represented as UTF-8 are refused.
- Terminal.app and macOS determine whether automation succeeds; the app does
  not retry, elevate, or choose another terminal.

## Final status

Pending implementation validation, review, CI, and merge.
