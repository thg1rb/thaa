# R001 — Closed Alpha Release Readiness

## Status

In progress. README/MIT presentation is complete. The release workflow and
packaging path are being prepared, but Apple signing/notarization credentials,
eligible signed-candidate validation, clean-machine installation evidence,
NFR-006 measurements, final read-only review, and user acceptance remain
outstanding. Windows unsigned package distribution has been explicitly approved
for selected testers; fresh release-profile CI and interactive install evidence
remain outstanding.

Branch: `feature/r001-closed-alpha-readiness`

Base: `develop` at `e03249aa1543aa08af72e96169ffed70205e53bf`

Target version: `v0.1.0` (release metadata should mark it as a pre-release;
do not add an alpha suffix to the version).

## Objective

Prepare the accepted macOS and Windows feature set for a first selected-tester
release. Validate installable packages and first-run behavior, document the
actual tester experience, and preserve explicit release approval before
publishing. Do not promote `develop` to `main`, tag, publish, or distribute
artifacts as part of this pre-acceptance change.

## Current Baseline

- W013.1, W014, and W015 are integrated into `develop`; W015 is Done on merge.
- Version metadata is `0.1.0` in `package.json`, `src-tauri/Cargo.toml`, and
  `src-tauri/tauri.conf.json`.
- Tauri bundling is enabled with application icon resources. macOS PR and
  develop/manual jobs build with `--no-bundle`; the Windows PR job is now set to
  build the release-profile unsigned NSIS package, while Windows develop/manual
  jobs use `--no-bundle`.
- The repository is public; future GitHub Release assets will be publicly
  downloadable to the selected early-tester audience. Release classification
  is GitHub **Pre-release**; version/tag are exactly `0.1.0` / `v0.1.0`.
- `.github/workflows/release.yml` now defines manually dispatched candidate
  and final-release paths from trusted `main`. It is not dispatched here. PR CI
  is being changed to build a release-profile Windows NSIS installer.
- No local Developer ID identity, GitHub release environment, Actions secret,
  or Actions variable is configured. No secret values belong in this repo.
- The README previously had no screenshots or end-user installation guidance.
  CONTRIBUTING also incorrectly said product inspection features were not
  implemented.

## Decisions

- Target the first release at Apple Silicon (`aarch64`) on macOS 15 or later,
  packaged as a Developer ID signed, notarized, and stapled DMG.
- Target Windows 11 x64, packaged only as an unsigned NSIS setup executable
  named `Thaa_0.1.0_x64-setup.exe`; no MSI. User explicitly approved the
  unsigned selected-tester package with SmartScreen disclosure and notice that
  Smart App Control or managed policy may block execution. Authenticode is not
  required for this selected-tester decision.
- Use the MIT License for Thaa with copyright year 2026 and the established
  public repository-owner handle `thg1rb`. Preserve third-party licenses and
  notices.
- The README tagline is “Know what's listening.” Its installation sections
  must say downloads are not available until the release is published.
- The intended release is public, marked Pre-release, and versioned/tagged
  `0.1.0` / `v0.1.0`; no alpha/beta suffix.
- The selected notarization mechanism is an App Store Connect API key.
- **Promotion sequence:** after R001 acceptance and integration into `develop`,
  prepare a separate reviewed PR from `develop` to `main`, require release
  validation, merge it, then create `v0.1.0` at the accepted `main` commit.
  Finally dispatch this workflow from `main` in `publish` mode. Protected
  environment approvals gate signing and publication. This sequence is
  documented only; no promotion or release action occurs in this task.
- User-approved release evidence plan uses four-hour per-platform stability
  runs. This is a test duration, not a new NFR threshold.
- The app bundle must advertise macOS 15.0 to match the selected Apple Silicon
  tester target. Package inspection found Tauri's default 10.13 minimum and
  `tauri.conf.json` now sets 15.0 and explicitly enables Hardened Runtime.
- Draft release notes and a selected-tester guide live under `docs/releases/`;
  both clearly state packages are not available.

## Scope

- Build and validate platform-specific installable candidate packages, signing,
  notarization/stapling, artifact handling, release notes, and an approval-gated
  release workflow without triggering publication in this work.
- Validate installation, first launch, permissions/errors, uninstall and
  reinstall on clean supported machines.
- Collect NFR-006 idle CPU/memory, scan duration, and longer-run stability
  evidence on macOS and Windows before making performance claims.
- Publish a professional user-facing README with real sanitized screenshots,
  accurate release installation steps, feature/capability guidance, known
  limitations, security/contribution/feedback links, and a separate developer
  setup section.
- Add the standard MIT `LICENSE`, set project SPDX metadata, reconcile
  project-owned license statements, and preserve third-party notices.
- Update release, installation, security, test, and project-state
  documentation as needed.

## Non-goals

No new product features, W016, Linux release, automatic updater, telemetry,
license changes to third-party materials, public release, tag creation,
`develop` to `main` promotion, or tester distribution before final user
acceptance.

## Release Readiness Gates

- [x] Define public GitHub Pre-release distribution for selected testers.
- [ ] Provision protected Developer ID and notarization credentials and run a
      signed/notarized/stapled Apple Silicon DMG build.
- [x] Select Windows 11 x64 NSIS-only packaging and unsigned selected-tester
      distribution with explicit SmartScreen/Smart App Control disclosure.
- [ ] Inspect signatures, notarization tickets, package contents, application
      identity/version/icon, and generated checksums.
- [x] Define the reviewed `develop` → `main` release PR and post-promotion
      `v0.1.0` tag/manual-publication sequence. Do not execute it during R001
      pre-acceptance.
- [ ] Validate installation, first launch, permission/error handling, and
      uninstall/reinstall on clean macOS 15+ Apple Silicon and Windows 11 x64
      machines.
- [ ] Record NFR-006 measurements on both platforms and set only evidence-based
      budgets or claims.
- [x] Prepare draft `v0.1.0` release notes and tester instructions.
- [ ] Configure credentials/protected GitHub environments, produce a real
      signed/notarized/stapled macOS candidate, and pass Gatekeeper verification.
- [ ] Produce and install-test the Windows release-profile NSIS candidate.
- [ ] Complete isolated clean-install/reinstall validation and NFR-006
      measurements on the required platforms.

## README and Open-source Presentation

- [x] Use the generated approved Thaa application icon in the README header.
- [x] Add a concise product description, tagline, port-conflict introduction,
      and short Thai name explanation.
- [x] Document implemented features, safe Stop/Force semantics, platform
      differences, known limitations, user workflow, security, contribution,
      feedback, and separate development setup.
- [x] State that `v0.1.0` is planned and no downloads are currently published.
- [x] Capture and review a real application screenshot using controlled,
      loopback-only listener data; add it under `docs/assets/readme/`. The
      screenshot contains no personal process path or unrelated listener data.
- [x] Verify the rendered README in GitHub Light Mode. The live PR README was
      inspected in Chrome; logo sizing, badges, section hierarchy, screenshot,
      tables, code blocks, and links render correctly. Local links pass
      `pnpm docs:check`; public release/issues/workflow links returned HTTP 200.
- [ ] Verify GitHub Dark Mode. Deferred: GitHub exposes appearance selection
      only through a signed-in profile in the available browser session; no
      credentials were available, so dark rendering was not claimed as passed.
- [x] Add root MIT `LICENSE` with 2026 copyright and public owner handle
      `thg1rb`.
- [x] Declare SPDX `MIT` in Cargo and npm project metadata.
- [x] Record the project-license decision and third-party-notice boundary in
      ADR-007.
- [x] Review repository-owned licensing statements and third-party notices for
      consistency; do not alter third-party license terms.

The README presents the approved application icon, tagline, accurate current
capabilities, a port-conflict introduction, release status, installation flow,
usage guidance, platform limits, Security/Contributing/feedback links, separate
source-build instructions, a sanitized screenshot, and the MIT notice. GitHub
Light Mode rendering was visually reviewed; Dark Mode remains deferred because
the available logged-out session cannot switch GitHub's site appearance.
`pnpm docs:check` verifies local Markdown links.

## Testing and Security Plan

### Frozen NFR-006

> **Refresh/performance:** scans do not overlap, can be cancelled where
> practical, stay off the UI thread, avoid unbounded work and unnecessary
> subprocess launches, and use a conservative cadence. Record idle CPU/memory,
> scan duration, and longer-run stability on both target OSes before release
> claims.

The NFR evidence rule requires OS/tool versions, listener/process count, idle
duration, refresh cadence, scan duration, CPU/memory observations, and
limitations. Do not invent numeric budgets. A four-hour soak on each platform
is the current repeatability plan; it is not a new requirement. Measurements
are **NOT RUN** on both platforms, so no performance claims are made.

### Release workflow

`.github/workflows/release.yml` is separate from ordinary CI. Dispatch must run
the workflow definition from trusted `main`. Candidate mode validates and pins
the current `develop` commit SHA, builds that exact commit, and uploads seven-day
workflow artifacts only. GitHub exposes
manual dispatch only after the file exists on default branch (`main` here), so
the candidate workflow will not be dispatched until a later approved promotion
puts it there; the user selected static validation before that point. Final
mode accepts only the existing `v0.1.0` tag, validates its ancestry to `main` in
a no-secrets job before signing credentials are exposed, and builds the exact
validated tag commit SHA. The protected `release-publish` environment gates publication of
the public Pre-release. Pushing a tag alone cannot publish. Global permissions
are `contents: read`; only the publishing job receives `contents: write`.
Signing variables/secrets are available only to the macOS job through the
protected `release` environment. PR CI does not trigger this workflow or
receive release credentials.

GitHub environments/reviewers and branch/tag restrictions are administrative
configuration, not repository files; they are currently absent. Configure
`release` and `release-publish` as reviewer-protected environments that allow
workflow dispatch from `main`; configure a tag ruleset that restricts
`v0.1.0` creation to release maintainers and blocks updates/deletion. See
[credential checklist](../releases/v0.1.0-release-credentials.md) for exact
Apple secret/variable names and acquisition notes. Before approving a
candidate's protected `release` environment, the reviewer must inspect the
source commit SHA in the `Validate trusted workflow ref and release source`
job summary and compare that exact commit with the reviewed `develop` state.
The build jobs check out that immutable SHA, not the moving branch name.
Before final publication, the tag ruleset must also prevent `v0.1.0` from
being updated or deleted; the publishing job rechecks the tag's commit
immediately before creating the release.

- Run the existing formatting, lint, typecheck, frontend test/build, Rust
  format/Clippy/test, dependency audits, RustSec, docs/link, and diff checks.
- Exercise Tauri bundle generation for the selected `dmg` and `nsis` targets.
  The local macOS DMG is ad-hoc signed only and is not distributable. PR CI now
  builds the release-profile unsigned NSIS installer and verifies its exact
  name and Authenticode status.
- Verify release workflow permissions are least-privilege, secrets are read
  only from a protected environment, PRs cannot access signing secrets, and
  release publication cannot occur from ordinary PR/develop validation.
- Inspect dependencies, bundled notices, package contents, signing output,
  checksums, and the release asset set.
- Review screenshots for local usernames, private paths, unrelated processes,
  or command data before committing.
- Complete read-only Sub-agent review of release security and README/license
  accuracy. Main Agent applies fixes and repeats checks/review as required.

## Manual Validation Plan

- Install and launch each platform package on a clean supported machine.
- Verify first-run launch, tray/menu behavior, runtime inspection, Search,
  process icons, copy/open actions, capability-aware process actions, and
  permission/error messaging.
- Verify package signature, macOS Gatekeeper/notarization/stapling, Windows
  signature/installer behavior, and uninstall/reinstall.
- Record OS version, architecture, installer hash, and actual result. Do not
  infer interactive Windows behavior from CI compilation.
- Capture sanitized app screenshots from controlled process data for README.

## Current Implementation Evidence

- README, MIT metadata, ADR-007, project notices, draft release notes, and a
  tester guide are included in this R001 branch.
- `pnpm tauri build --bundles dmg` succeeded on the local Apple Silicon Mac
  and produced an unsigned local candidate at
  `src-tauri/target/release/bundle/dmg/Thaa_0.1.0_aarch64.dmg`. `hdiutil
verify` passed. The mounted app bundle reports version `0.1.0`, identifier
  `io.github.thg1rb.thaa`, and `LSMinimumSystemVersion` `15.0`.
- The candidate app has an ad-hoc signature with no Team ID. It is not Developer
  ID signed, notarized, or stapled and must not be distributed.
- Local signing is unavailable: `security find-identity -v -p codesigning`
  reported zero valid identities; no GitHub Actions release environments,
  secrets, or variables are configured. The release workflow fails closed
  unless the required macOS credentials are present.
- `README.md` accurately marks installers as not yet published and describes
  the selected package paths as planned, not available releases.
- No Git tag, GitHub Release, or tester distribution has been created. The
  unsigned local candidate remains only in the ignored Cargo target folder.
- The locally built DMG SHA-256 is
  `01812275b38a2d89afec6df25cb94789aa32695eefd75ff8885e871e5331fe6e`; this
  checksum is evidence for the local unsigned candidate only, not a release
  checksum.
- Local gates passed: Rust fmt, Clippy with `-D warnings`, 76 Rust unit tests
  and integration tests, frontend format/lint/typecheck, 48 frontend tests,
  production frontend build, npm audit, both RustSec target audits, maintained
  documentation formatting, local Markdown links, and `git diff --check`.
- External README release/issues/CI/badge URLs returned HTTP 200. The README
  rendered in GitHub Light Mode was reviewed in Chrome. Dark Mode is deferred:
  the available logged-out session did not expose the GitHub appearance
  selector, and no account credentials were available.
- RustSec reported two allowed dependency advisories: unmaintained
  `proc-macro-error` (RUSTSEC-2024-0370) and the `glib::VariantStrIter`
  unsoundness advisory (RUSTSEC-2024-0429).
- A real app screenshot was captured with a temporary loopback-only Python
  listener at port `43127`. It displays the fixture path `/private/tmp/` and
  contains no username or unrelated process details; dynamic PID and ambient
  listener count were removed from the committed image.
- Clean-machine install/reinstall, Gatekeeper/notarization, interactive Windows
  install validation, and NFR-006 performance measurements remain unperformed.
  Windows interactive validation is NOT RUN / deferred; Windows Working
  Directory/Project Root remain unsupported by the current provider.

## Read-only Review

The first read-only review found no defects in the MIT text/metadata, feature
claims, platform limitations, or README links. It identified outstanding R001
gates rather than implementation errors:

- The public GitHub Pre-release distribution for selected testers is now an
  explicit user decision; assets will be publicly downloadable.
- Rendered GitHub Dark Mode README review is still outstanding. Light Mode was
  inspected in the live PR; a sanitized real screenshot is included.
- Developer ID signing/notarization/stapling and clean-machine validation are
  outstanding. Windows Authenticode is intentionally not configured: user
  approved unsigned NSIS distribution with explicit warning.
- A manual/tag release workflow and release-profile Windows NSIS validation are
  now configured in source; they require fresh CI/review. No candidate was run
  and no release workflow was dispatched.

No code changes were required from review. The reviewer also confirmed that
project license statements and third-party notice boundaries are consistent.

The follow-up screenshot review found no personal path or unrelated listener
data. Its suggestion to remove the fixture PID and ambient-listener count was
applied to the screenshot; the stale screenshot-status text was reconciled.
The final README/license review found no remaining findings in that scope.
GitHub Light Mode rendering was visually checked; Dark Mode remains explicitly
deferred because the logged-out browser session cannot change GitHub's site
appearance.

R001 remains in progress and user acceptance remains pending.

## Final Acceptance

Pending. R001 is not complete until release packages, signing and platform
validation, clean-machine installation evidence, performance evidence,
documentation, read-only review, and explicit user acceptance are complete.

## Stop Boundary

Keep the R001 PR open for user acceptance. Do not merge it, promote to `main`,
create the `v0.1.0` tag, publish a GitHub Release, distribute artifacts, or
begin W016 before the user explicitly accepts the completed R001 result.
