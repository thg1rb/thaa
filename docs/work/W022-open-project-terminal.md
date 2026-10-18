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
to activate and run only a quoted `cd -- <path> || exit` command without an
explicit window target. Terminal.app creates a new window for that `do script`
request; this behavior is covered by the interactive evidence below.
Thaa does not intentionally execute project scripts or commands. The shell's
own startup files remain outside Thaa's control.

`NSAppleEventsUsageDescription` explains the explicit action. macOS TCC
authorization is respected; an observed Apple Events denial is reported as a
recoverable authorization error, while other native failures receive a
separate generic launch error. No broad permission, privilege escalation,
external call, additional scan, persistent cache, or dependency is introduced.
Windows returns an unsupported result and the UI action is absent when no
Project Root exists.

The path is data in an argv slot, then quoted using AppleScript's standard
shell quoting. Non-UTF-8 paths are rejected instead of lossily converted.
Standard output is discarded; standard error is captured with a 4 KiB cap,
used only to recognize known authorization-denial wording, then discarded.
Raw diagnostics and paths are not logged or returned. The normal filesystem
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

The first PR CI run passed Shared Quality and macOS Native Validation but
Windows Native Validation found that the macOS-only `InvalidPath` and `Failed`
error variants were unused on Windows under warnings-as-errors. The variants
now carry target-specific dead-code allowances; this changes no Windows
behavior. Both read-only reviewers approved that narrow correction. The
rerun, GitHub Actions run `39136381940` on code commit `4e3a44c`, passed Shared
Quality, macOS Native Validation, and Windows Native Validation. This work
record remains in progress because interactive Terminal behavior has not yet
passed.

### W022.1 diagnosis and follow-up (2026-10-18)

The prior generic UI error was reproduced with a freshly rebuilt isolated
debug bundle. The exact error was Tauri's permission rejection:
`open_project_in_terminal not allowed. Permissions associated with this
command: allow-open-project-in-terminal`. The command was rejected by the
main-window capability before snapshot resolution or native launch. Therefore
the original failure was not evidence of TCC denial. The missing
`allow-open-project-in-terminal` capability grant is now present, with a
regression test that parses the capability file and checks the permission.

A separate direct AppleScript diagnostic of the old explicit `make new
window` form returned Terminal AppleEvent handler error `-10000`; this was a
diagnostic of that form, not the original UI failure. The integration now
uses Terminal's `do script` without a window target. In the interactive test
with Terminal already running, this created a distinct window and started in
the requested directory. Native stderr is captured with a 4 KiB bound,
discarded after classifying known Apple Events authorization-denial text, and
never returned raw to the UI or logged. The UI now presents the backend's
safe recoverable message. A unit test distinguishes the observed denial
wording from unrelated Apple Events errors. No TCC database, system setting,
installed app, signing identity, or entitlement was changed.

The validation app was `Thaa W022 Validation.app`, bundle identifier
`io.github.thg1rb.thaa.w022validation`, ad-hoc signed, with the existing
`com.apple.security.automation.apple-events` entitlement and this bundle
usage description: “Thaa uses Terminal automation only when you choose Open
in Terminal, to create a new Terminal window at the detected project
folder.” Unified logging associated the Apple Event request with this debug
bundle and `/usr/bin/osascript`. The action succeeded without a permission
prompt or denial; this establishes effective authorization for this test,
not that a new authorization prompt was shown or approved during it.

With Terminal already running, the UI action created a distinct window. For
the disposable project root `/private/tmp/thaa-w022-acceptance space`, the
listener ran from its `nested` child directory and the new window's observed
`pwd` was exactly `/private/tmp/thaa-w022-acceptance space`. A second action
used the literal project path
`/private/tmp/thaa-w022 $(touch /private/tmp/thaa-w022-INJECTED); ampersand & (quoted)'`.
The resulting window's `pwd` exactly matched that path and the sentinel file
`/private/tmp/thaa-w022-INJECTED` was absent. Pre-existing Terminal window
IDs remained present; only the two windows created by these tests were
closed. The debug app showed an isolated, disposable loopback listener, its
nested Working Directory, the expected project-root marker, and one action.
After stopping only the fixture listeners and refreshing, their rows and
actions disappeared.

An additional fixture with `café` in its path did not expose Working
Directory or Project Root: the existing macOS `lsof` provider escaped the
UTF-8 bytes as `\\xc3\\xa9`, which the current provider intentionally treats
as unavailable instead of lossy path conversion. No W022 action was offered.
This is a metadata-provider limitation; Unicode remains covered by the
automated shell-quoting tests. The automation denial path is covered through
error classification tests; real TCC denial was not induced.

Terminal.app was already running when validation began. Creating a
not-running scenario would require closing the user's existing Terminal
session, which was deliberately left untouched. Consequently the
already-running/new-window and exact-`pwd` case passes, but the not-running
case is **NOT RUN**. AC-03 is demonstrated for the running case; AC-08
remains pending because it explicitly requires both running states. PR #67
must remain Draft until that acceptance gap is resolved without disturbing
the user's session. The original three screenshots and original checkout
were not touched.

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
