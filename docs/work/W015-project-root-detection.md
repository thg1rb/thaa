# W015 — Project Root Detection

## Status

Status: User accepted — integration pending

Branch: `feature/project-root-detection`

Base: `develop` at `34b3bd8dad499abe9d3ec7161d4c042f1961a921`

Requirement: FR-012 (frozen wording)

## Objective

Detect and display the nearest plausible project root using the process
working directory already present in runtime process metadata. Detection is
best-effort, local-only contextual metadata. It must not affect listener
visibility, process identity, or process actions.

## Resolved Detection Policy

- Start at the canonicalized process working directory and inspect that
  directory before its parents.
- The nearest directory containing any supported marker is the project root.
  Directory proximity has precedence over marker category: a nested project
  marker wins over a more distant `.git` or workspace marker.
- Supported markers are `.git` (file or directory), `package.json`,
  `pnpm-workspace.yaml`, `Cargo.toml`, `go.mod`, `pyproject.toml`, `pom.xml`,
  `build.gradle`, `build.gradle.kts`, `settings.gradle`,
  `settings.gradle.kts`, `composer.json`, and direct-child `*.sln` or
  `*.csproj` files.
- Lockfiles, `README.md`, `Makefile`, `setup.py`, and `requirements.txt` alone
  are not project-root evidence. Marker contents are never parsed or run.
- Multiple markers in the same directory identify the same root. Marker
  category does not override directory proximity; same-directory evidence is
  treated as an unordered set because the selected path is identical. The
  public runtime value is the path only.
- A workspace marker identifies its directory when no nearer supported
  marker exists. W015 exposes only one project-root value, not separate
  repository or workspace roots.
- Canonicalize the starting path once. A missing/non-directory/unavailable
  working directory, a canonicalization failure, or an access error while
  checking a candidate directory produces no project root. Symlinked starting
  paths resolve to their actual directory; marker symlinks count only when
  their metadata resolves to an expected regular file/directory type. Marker
  checks are metadata-only.
- Traverse ancestors only, stopping at the filesystem root/volume boundary.
  The platform adapter compares Unix device IDs or the Windows volume mount
  path before each candidate is inspected; the first different filesystem
  ends traversal. Never recursively scan siblings, home directories, or mounted
  volumes.
- Reuse the existing working-directory field. The current Windows provider
  reports it unavailable, so Windows root detection is unavailable unless
  that existing field is provided; W015 will not add another process
  inspection mechanism or use PEB/WMI/shell/elevation.

## Architecture

Implement a small filesystem-backed detector in the application layer, with
temporary-directory fixtures. OS-specific filesystem identity checks live in
the platform adapter. During each runtime scan, derive an optional root from
each uniquely inspected process and attach it to `RuntimeEntry`.
It is presentation metadata only and is excluded from `ProcessIdentity` and
`ProcessActionTarget`. The runtime DTO carries an optional `projectRoot` path;
the existing Runtime Card shows a distinct Project Root value when present.
Detection failures do not fail a scan or remove a listener. No dependency or
external command is expected.

## Non-goals

Git branch/repository enrichment, runtime/framework detection, project-file
parsing, filesystem mutation, shell execution, project-root search, a second
root/workspace field, new Windows working-directory inspection, and all
subsequent work items.

## Security and Privacy

Paths and marker names are untrusted local metadata. Only existence/type
inspection is performed; project files are not read or executed. Paths stay
inside the local snapshot/UI and are not logged or transmitted externally.
An unavailable root is normal and never affects Stop/Force authorization.

## Test Plan

- Cover current-directory, parent, distant ancestor, no marker, root boundary,
  unavailable/nonexistent/non-directory paths, and multiple markers.
- Cover nested project precedence over distant `.git`, workspace-root
  behavior, every supported marker family, and deterministic root selection
  when a directory has multiple markers.
- Verify generic documents/lockfiles alone and nested .NET markers do not
  create a root; verify long/control-containing paths remain bounded/safe in
  the Runtime Card.
- Cover symlinked starting directories where supported and metadata errors
  where reproducible. Avoid brittle permission manipulation; mounted-volume
  fixture coverage is deferred because deterministic mount setup is not
  available to unit tests.
- Verify runtime DTO/UI flow, graceful absence, long-path layout, and that
  listeners/actions remain available when detection is unavailable.
- Preserve FR-005 Search semantics and process-action identity behavior.

## Validation Evidence

- Local Rust suite: 76 unit tests and all integration test binaries passed.
- `cargo fmt --check`, Clippy with `-D warnings`, frontend format/typecheck/lint,
  48 frontend tests, production build, docs link check, dependency audit, and
  `git diff --check` passed on the implementation tree.
- RustSec audit passed for the configured macOS and Windows targets. It reports
  the repository's existing allowed advisories for `proc-macro-error` and
  `glib`; no new dependency was introduced.
- The macOS `pnpm tauri build --debug --no-bundle` build passed.
- A full Windows cross-target check on this macOS host stopped in Tauri's
  Windows resource build because `llvm-rc` is unavailable. The first hosted
  Windows Native Validation exposed a volume-boundary mismatch: the detector
  compared an original DOS path with its canonical extended-length path. The
  detector now uses the canonical start path consistently for volume
  comparisons. Corrected Windows Native Validation passed on final head
  `91001d1096489799f1a122db326f53c0d3a8f3b7`.
- A bundled macOS debug app was copied to an isolated temporary app identity
  and launched, avoiding the pre-existing dev server on port 1420. A controlled
  Python listener from
  `/private/tmp/thaa-w015-manual/project/nested` showed the distinct Project
  Root `/private/tmp/thaa-w015-manual/project`. A second controlled listener
  from an unmarked directory also remained visible after Refresh and showed no
  Project Root row. Refresh retained both entries. No Stop/Force action was
  invoked during this metadata validation.
- Manual nested Git-root precedence, long-path display, narrow-window
  validation, and action execution were not separately exercised in the
  packaged UI. Narrow-window interactive validation remains NOT RUN / deferred
  because native resizing was unavailable in the validation environment.
  Interactive Windows visual validation remains NOT RUN / deferred; the
  existing Windows provider does not provide working-directory metadata.
- Packaged macOS Search regression check matched the controlled listener by
  exact port `43871`; Clear restored the 20-listener view. No process action was
  invoked by Search.
- Final PR-head CI run `37264993742` passed Shared Quality, macOS Native
  Validation, and Windows Native Validation. The final read-only re-review of
  `91001d1` found no actionable findings.

## Final User Acceptance

Manual validation: **PASS** — the user reports no blocking issue and accepts
W015. Accepted behavior includes detection from the existing process Working
Directory, nearest supported marker selection by ancestor-only traversal,
nearer project markers taking precedence over more distant Git/workspace
markers, no whole-filesystem scan or Git CLI dependency, optional
presentation-only Project Root metadata, no effect on process identity or
Stop/Force authorization, listener visibility when root metadata is absent,
and continued Search/Runtime Inspector operation.

The user's overall manual acceptance does not change the record of checks that
were not individually performed in the controlled fixture session: narrow-
window validation and interactive Windows validation remain **NOT RUN /
deferred**. The current Windows provider does not expose process Working
Directory metadata. The automated Windows Native Validation passed.

## Manual Validation Plan

On macOS, use controlled listeners launched from a simple project, a nested
project inside a Git root, and a directory with no supported markers. Check
Refresh, Search, existing actions, and long project paths. Record narrow-window
and interactive Windows validation honestly if unavailable. Hosted Windows
Native Validation passed on the final PR head; interactive Windows visual
validation remains deferred.

## Acceptance Criteria

- FR-012 policy is implemented as documented and remains ancestor-only.
- Root detection is optional and failure-safe; listener and process action
  behavior remain unchanged.
- W014 Search and W013/W013.1 UI behavior remain intact.
- Rust/frontend tests, local quality/security/docs checks, final PR-head native
  CI, read-only review, and user manual acceptance pass.
- PR targets `develop`; merge and post-merge verification remain pending.

## Evidence

Implementation is on the dedicated branch. The final read-only review and
final-head PR CI and user acceptance are complete. The packaged macOS
simple-root/no-root, Refresh, Search-by-port, and Clear checks are recorded
above. Narrow-window and interactive Windows visual validation remain
deferred. W015 status becomes Done on merge after integration and post-merge
verification.
