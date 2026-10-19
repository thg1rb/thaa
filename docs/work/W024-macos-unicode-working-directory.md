# W024 — macOS Unicode Working Directory Fidelity

## Status

**Done on merge** — PR [#71](https://github.com/thg1rb/thaa/pull/71) targets
`develop`. The branch is `fix/w024-macos-unicode-working-directory`, based on
`develop` / `origin/develop` `6e213a9886513bb1e60f65fa3a86c9bc7d8a50b0`.

The original checkout was left untouched because it contains three user-owned
untracked README screenshots. W024 uses an isolated worktree; those screenshots
were intentionally excluded, and no user-owned untracked file was modified.

## Requirement and Motivation

W024 addresses a confirmed macOS metadata correctness gap in the frozen
requirements:

- **FR-006 (P0):** show the process Working Directory when available and make
  unavailable fields explicit.
- **FR-012 (P2):** detect the nearest plausible Project Root from the process
  Working Directory.
- **FR-013 (P2):** derive Git repository root and branch from the process
  Working Directory.
- **FR-014 (P1):** make project-oriented actions available where Project Root
  metadata is available.

With `LC_ALL=C`, the macOS `lsof` process provider escapes UTF-8 bytes in a cwd
path. The current conservative parser intentionally rejects ambiguous
escape-like values, so Working Directory becomes unavailable and downstream
Project Root, Git Context, Reveal, Terminal, and Editor enrichment cannot be
derived. A controlled local experiment confirmed that `/usr/sbin/lsof` 4.91
emits raw UTF-8 for the same test process under `LC_ALL=en_US.UTF-8`, and
escaped bytes under `LC_ALL=C`.

## Scope Freeze

### In scope

- Request deterministic UTF-8 output from the fixed macOS `/usr/sbin/lsof`
  process-metadata invocation.
- Preserve direct argument invocation, cleared environment, bounded execution,
  and process identity checks.
- Preserve fail-closed handling for ambiguous escape-like output and invalid
  path bytes; do not add lossy decoding or infer a path.
- Add parser regression coverage for raw UTF-8 paths and escaped values.
- Add macOS native controlled-child coverage proving the provider returns the
  exact Unicode Working Directory.
- Document platform behavior, requirement traceability, limitations, tests,
  and evidence.

### Out of scope

- Windows Working Directory or Project Root discovery.
- Changes to frozen requirement wording, W015 marker policy, Git semantics, or
  W021–W023 actions.
- Changes to process identity, listener discovery, process actions, or snapshot
  behavior.
- Decoding `lsof` escape sequences, lossy path conversion, project-file reads,
  shell execution, or filesystem traversal beyond existing Project Root
  detection.
- Releases, version/tag changes, or `develop` → `main` promotion.

## Architecture and Security

The change is isolated to the macOS process provider. `/usr/sbin/lsof` remains
fixed, receives fixed arguments plus a numeric PID, and runs without a shell,
with inherited environment cleared and bounded output/time. Only `LC_ALL` is
changed from `C` to the explicit UTF-8 locale `en_US.UTF-8`; `TZ=UTC` remains.
The provider continues to parse NUL-delimited output as bytes and rejects
escape-like path/name representations rather than decoding them. Invalid
native paths remain unavailable. No process identity fields or action targets
are derived from Working Directory.

The native regression test will use only a test-owned child process and
temporary directory. It must clean up the child and fixture on all exits. This
work adds no dependencies, privilege requirements, shell commands, network
requests, or sensitive logging.

## Acceptance Criteria

- **AC-01 — UTF-8 locale:** macOS process metadata invocation uses a
  deterministic UTF-8 locale with no inherited environment.
- **AC-02 — Exact path fidelity:** a controlled child whose Working Directory
  contains Unicode is reported with exact native path identity.
- **AC-03 — Conservative parsing:** ambiguous escaped output and invalid
  representations remain unavailable; no lossy decode or guessed path is
  introduced.
- **AC-04 — Downstream context:** existing Project Root and Git derivation can
  consume the faithfully reported path without changes to their semantics.
- **AC-05 — Safety and regression:** process identity, optional-metadata
  behavior, listener visibility, actions, and Windows behavior are unchanged.
- **AC-06 — Validation and review:** relevant local checks, macOS native CI,
  Windows CI, documentation checks, and dedicated read-only review pass before
  merge.

## Validation Plan

- Unit tests: UTF-8 raw bytes normalize unchanged; escape-like values remain
  unavailable; command locale remains fixed and environment-cleared.
- macOS native test: inspect only a controlled child with a Unicode cwd and
  compare the returned `PathBuf` to the canonical fixture path.
- Existing Runtime Snapshot native test: retain listener/process integration
  coverage; no listener or action behavior changes are expected.
- Full Rust format, Clippy, and test suite; frontend checks/build because the
  changes are Rust/docs only where practical; documentation format/link checks;
  dependency audit; `git diff --check`; native macOS debug build.
- PR CI: Shared Quality, macOS Native Validation, and Windows Native
  Validation, all on the final reviewed head.
- Read-only review: provider correctness, encoding and failure behavior,
  process identity, test cleanup, scope, security, and docs.

## Platform and Known Limitations

This work changes only macOS process metadata. `en_US.UTF-8` availability and
behavior must be confirmed on the repository's macOS 15 native runner. If the
locale is unavailable, provider failure must remain graceful and metadata
unavailable; no fallback to ambiguous `C`-locale output is accepted. Windows
Working Directory and consequent Project Root/Git/action limitations remain
unchanged. This work does not guarantee every macOS filesystem path can be
represented by `lsof`; ambiguous output remains unavailable.

## Evidence

Before implementation, local macOS 27.0.1 with `/usr/sbin/lsof` 4.91 was tested
using a test-owned loopback listener in a disposable Unicode-named directory.
With inherited environment cleared, `LC_ALL=C` emitted escaped UTF-8 bytes,
while `LC_ALL=en_US.UTF-8` emitted the path as raw UTF-8. The listener and
temporary fixture were stopped and removed. This is feasibility evidence, not
final acceptance evidence; native tests and CI remain required.

### Local implementation validation

- Rust full suite: PASS on macOS 27.0.1; includes the controlled-child Unicode
  cwd test and existing project-root, Git-context, and runtime snapshot tests.
- Rust format and Clippy (`-D warnings`): PASS.
- Frontend format, lint, typecheck, 92 tests, and production build: PASS.
- Documentation format, local links, and `git diff --check`: PASS.
- macOS Tauri debug no-bundle build: PASS.
- JavaScript dependency audit: PASS; no known vulnerabilities.
- RustSec: PASS with allowed advisory warnings for `proc-macro-error`, `glib`,
  and `zbus_names` / `zvariant` family advisories; W024 changes no dependency.
- PR #71 checks passed on implementation head `42f8241cea84a7ecf73423f503a21a8669afd70b`:
  Shared Quality, macOS Native Validation, and Windows Native Validation.
- Final PR-head checks after this work-record closeout commit and post-merge
  validation: pending.

### Acceptance status before PR

- AC-01 — UTF-8 locale: PASS; the fixed locale is asserted by a unit test and
  the controlled native child test verifies operation on the local host.
- AC-02 — Exact path fidelity: PASS locally; the native provider returns the
  canonical Unicode cwd for the controlled child.
- AC-03 — Conservative parsing: PASS; escaped-value negative tests remain and
  raw UTF-8 normalization is covered.
- AC-04 — Downstream context: PASS for Project Root unit coverage using a
  Unicode working directory. Existing Git-context tests independently cover
  Unicode path input; this is component-level evidence, not a full provider-to-
  Git end-to-end test.
- AC-05 — Safety and regression: PASS locally and on the implementation-head
  Shared Quality, macOS Native Validation, and Windows Native Validation jobs.
- AC-06 — Validation and review: implementation review PASS with no findings;
  final PR-head checks and merge remain pending.

### Read-only review

The dedicated W024 read-only review found no code or security findings. It
requested more precise wording for AC-04 because the Project Root and Git
Unicode evidence is component-level rather than a single end-to-end provider
flow; this record was clarified accordingly. Documentation re-review found no
further findings. The implementation review and re-review were both read-only.

## Post-merge Unicode workflow validation

An isolated debug build (`io.github.thg1rb.thaa.w024validation`) was exercised
on macOS 27.0.1 (arm64, build 26A434) after PR #71 merged. The installed
`/Applications/Thaa.app` was not used. `/usr/sbin/lsof` 4.91 was run with the
provider's fixed arguments and controlled environment (`env_clear`,
`LC_ALL=en_US.UTF-8`, `TZ=UTC`). The locale was available on this host. Raw
provider output preserved Thai, accented Latin, and Japanese path bytes; the
Working Directory matched the disposable fixture's `nested` directory, and
Project Root detection returned the exact marked ancestor. No lossy conversion
was observed.

The primary disposable project used a `package.json` marker, a nested working
directory, and a local Git repository. Its path included Thai, `é`, Japanese,
spaces, and parentheses. Runtime Inspector showed the correct listener PID and
loopback port, exact Working Directory and Project Root, Git root and `main`
branch, and rendered Unicode correctly. Searching by port returned the
listener; Refresh preserved its metadata.

The actual UI actions were exercised:

- **Reveal Project Folder:** Finder displayed the exact Project Root, with the
  marker and `nested` directory visible. It did not open the parent.
- **Open in Terminal:** Terminal.app was confirmed not running immediately
  before the UI action. The action launched Terminal.app without a permission
  prompt or Thaa error. The project shell's current directory, observed
  read-only through `lsof`, exactly matched the canonical Unicode Project
  Root. A separate default shell also appeared during cold start. The
  Computer Use interface refused access to Terminal.app, so typing `pwd` in
  that window was **NOT RUN**; the literal-command criterion therefore remains
  pending user-assisted confirmation even though the OS-reported shell cwd
  matched.
- **Open in Editor:** VS Code opened the exact project folder in Restricted
  Mode. No workspace trust was granted and no project task was started by
  Thaa.

A second disposable project combined Japanese characters, spaces, single
quotes, a dollar sign, a semicolon, and parentheses. The provider and Runtime
Inspector preserved the exact Working Directory, Project Root, Git root, and
branch. The actual **Open in Editor** action opened that exact folder in the
already running VS Code window. The workspace remained in Restricted Mode.
Automated W022/W023 path-safety coverage remains the evidence for the broader
metacharacter matrix; this interactive run did not execute a command-like
sentinel.

Both test-owned loopback listeners were stopped and their disposable
directories removed. Refresh then showed zero matches for the test port, with
no stale project actions. Existing user processes and the installed Thaa
application were not terminated or changed.

### Post-merge validation status

| Check                                             | Result  | Evidence                                                                                                                                 |
| ------------------------------------------------- | ------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| Unicode Working Directory through provider and UI | PASS    | Controlled listener with Thai, `é`, Japanese, spaces, and parentheses; exact directory displayed                                         |
| Unicode Project Root detection                    | PASS    | Marked ancestor exactly matched expected Project Root                                                                                    |
| Unicode Git Context                               | PASS    | Disposable repository root and `main` branch displayed                                                                                   |
| Search and Refresh                                | PASS    | Port search found listener; Refresh retained metadata                                                                                    |
| Reveal Project Folder                             | PASS    | Finder showed the Project Root contents                                                                                                  |
| Open in Terminal                                  | PARTIAL | Real UI action launched app; OS-reported project-shell cwd matched root; interactive `pwd` NOT RUN because CUA denied Terminal UI access |
| Open in Editor                                    | PASS    | VS Code Explorer showed exact Unicode and mixed-special-character project roots; Restricted Mode retained                                |
| Listener exit and stale-action removal            | PASS    | Both fixtures stopped, directories removed, Refresh showed no matching test listener                                                     |
| W024 PR CI                                        | PASS    | PR #71 run 39202113984: Shared Quality, macOS Native Validation, Windows Native Validation                                               |

The complete interactive W024 downstream acceptance remains **PARTIAL** until
the project shell prints `pwd` and that output is confirmed against the
fixture's canonical Project Root. This record intentionally does not infer
that check from process cwd metadata or from Terminal launch success. No
corrective implementation was indicated by the observed results.

### Locale compatibility observation

`en_US.UTF-8` was available and behaved as intended on local macOS 27.0.1
arm64 with `lsof` 4.91, and the PR's macOS Native Validation passed on the
repository's hosted macOS runner. This is evidence for those tested
environments, not a guarantee for every supported macOS installation. If the
locale or provider output is unavailable or ambiguous, metadata remains
unavailable; the provider does not fall back to lossy decoding.
