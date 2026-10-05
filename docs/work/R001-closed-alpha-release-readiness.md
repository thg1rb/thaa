# R001 — Closed Alpha Release Readiness

## Status

In progress. The current phase is **Gate A — release infrastructure
acceptance**. README/MIT presentation is complete. The zero-budget macOS and
Windows artifacts are intentionally unsigned for selected testers; paid
signing is deferred release hardening, not a v0.1.0 blocker. Gate B candidate
validation remains after Gate A integration and promotion. Do not merge this PR
or publish a release without explicit acceptance at the applicable gate.

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
- `.github/workflows/release.yml` defines manual candidate/publish paths from
  trusted `main`; unsigned is the explicit default and trusted macOS signing is
  an opt-in future path. The workflow has not been dispatched.
- No local Developer ID identity, GitHub release environment, Actions secret,
  or Actions variable is configured. No secret values belong in this repo.
- The README previously had no screenshots or end-user installation guidance.
  CONTRIBUTING also incorrectly said product inspection features were not
  implemented.

## Decisions

- Target the first release at Apple Silicon (`aarch64`) on macOS 15 or later,
  packaged as an unsigned/ad-hoc DMG. It is not Developer ID signed or Apple
  notarized; the user-facing disclosure and normal per-app approval guidance
  must be clear. Paid signing/notarization/stapling are deferred hardening.
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
- User-approved NFR-006 evidence method is an approximately 8-hour continuous
  macOS soak. This is test methodology, not a new requirement threshold.
  Windows interactive measurements and soak are NOT RUN / deferred.
- The app bundle must advertise macOS 15.0 to match the selected Apple Silicon
  tester target. Package inspection found Tauri's default 10.13 minimum and
  `tauri.conf.json` now sets 15.0 and explicitly enables Hardened Runtime.
- Draft release notes and a selected-tester guide live under `docs/releases/`;
  both clearly state packages are not available.

## Scope

- Complete Gate A's credential-free unsigned macOS DMG and Windows NSIS
  packaging paths, checksums, static release-workflow controls, documentation,
  read-only review, and final PR CI without publishing artifacts.
- After Gate A acceptance/integration, run Gate B from a candidate commit on
  `main`: macOS clean-machine install/first-launch/product regression,
  uninstall/reinstall, NFR-006 measurements, and the 8-hour soak. Windows
  package/CI evidence is required; interactive Windows installation and soak
  are explicitly deferred for this selected-tester preview.
- Keep Developer ID, Apple notarization/stapling, and Authenticode support
  available as optional post-v0.1.0 hardening; none is required for this
  zero-budget release.
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

### Gate A — PR #37 release infrastructure acceptance

- [x] Define public GitHub Pre-release distribution for selected testers.
- [x] Select Apple Silicon macOS 15+ unsigned/ad-hoc DMG for zero-budget v0.1.0.
- [x] Select Windows 11 x64 unsigned NSIS-only packaging; no MSI.
- [x] Implement explicit unsigned-default candidate/publish workflow while
      preserving trusted signing as an opt-in path.
- [ ] Build/verify macOS DMG and Windows NSIS installer in credential-free PR
      CI; generate and verify SHA-256 checksums.
- [x] Define the reviewed `develop` → `main` release PR and post-promotion
      candidate, accepted-SHA tag, and manual-publication sequence. Do not
      execute it during Gate A.
- [x] Update README, release notes, tester guidance, test catalog, CI and work
      records with accurate unsigned warnings and deferred hardening.
- [x] Prepare the MIT license, release notes, tester guide and announcement.
- [x] Run local quality/security/docs/workflow static checks.
- [x] Complete the read-only review of the zero-budget workflow and docs; the
      checksum guidance and candidate artifact provenance findings were fixed.
- [ ] Confirm CI passes on the final PR head, including macOS DMG and Windows
      NSIS packaging.
- [ ] Present Gate A for explicit user acceptance. Keep PR #37 open until then.

Gate A does **not** require Apple Developer Program membership, Developer ID,
notarization, stapling, Authenticode, or actual release publication.

### Gate B — release candidate acceptance after promotion

- [ ] Promote the accepted `develop` state through a separate reviewed
      `develop` → `main` PR; merge does not publish a release.
- [ ] Run the manual non-publishing candidate workflow from the exact intended
      `main` commit; record its SHA and both artifact/checksum results.
- [ ] Validate macOS 15+ Apple Silicon DMG integrity, unsigned first-launch
      behavior, clean/isolated installation, full packaged regression, quit/
      relaunch, uninstall and reinstall.
- [ ] Measure macOS idle CPU/memory and representative scan duration and
      complete the approximately 8-hour continuous packaged-app soak.
- [ ] Confirm Windows release-profile unsigned NSIS output, checksum, and
      automated CI; interactive clean-machine install and soak remain
      NOT RUN / deferred and are nonblocking for this selected-tester release.
- [ ] Recheck all user-facing release claims and receive explicit approval of
      the exact candidate SHA before creating `v0.1.0` or publishing.
- [ ] Before publication, verify the `release-publish` environment actually
      enforces its intended reviewer approval. If that control is unavailable,
      record the limitation and require explicit release-owner approval plus
      the workflow's exact-SHA/tag checks; do not make a paid GitHub plan a
      prerequisite.

Gate B does **not** require paid Developer ID, notarization, stapling, or
Authenticode credentials. Those remain deferred release hardening.

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
limitations. Do not invent numeric budgets. The approved macOS measurement
plan is an approximately **8-hour continuous soak** using the packaged app;
this is test duration, not a new NFR threshold. Windows measurements/soak are
**NOT RUN / deferred** because no interactive Windows environment is
available. Measurements have not been collected, so no performance claims are
made.

### Release workflow

`.github/workflows/release.yml` is separate from ordinary CI. Dispatch must run
the workflow definition from trusted `main`. Candidate mode builds the exact
main commit selected for dispatch and uploads immutable workflow artifacts for
90 days; it does not create tags/releases or public assets. Gate B records the
candidate workflow run ID, commit SHA, and artifact checksums. Publish mode
validates that run's successful unsigned macOS/Windows jobs and reuses those
exact artifacts instead of rebuilding them. The macOS distribution
choice is explicit and defaults to `unsigned`. That path requires no signing
environment/secrets and verifies the ad-hoc app signature, DMG integrity and
checksums without claiming notarization or Gatekeeper acceptance. Optional
`trusted` mode retains Developer ID and notarization support for candidate
validation, runs only when selected, and reads signing secrets only from the
protected `release` environment. The fixed `v0.1.0` publication path requires
unsigned mode. Windows remains unsigned NSIS.

Publish mode is a separate manual action. It requires the accepted candidate
SHA, candidate workflow run ID, and exact `v0.1.0` tag to resolve to that same
commit reachable from `main`. It confirms the candidate run was successful,
manual, on `main`, and contains the expected unsigned build artifacts; the
publish job verifies their original checksums and rechecks the tag immediately
before creating the public Pre-release. Pushing a tag alone cannot publish. Global permissions are
`contents: read`; only the publishing job receives `contents: write`. PR CI
does not trigger the release workflow or receive release credentials.

GitHub environments and tag rulesets are administrative configuration, not
repository files. Configure `release` only for trusted signing and
`release-publish` for final publication, with reviewer protection where
available at no cost. GitHub rulesets are available on Free for public
repositories. Protect `v0.1.0` against unauthorized creation, updates and
deletion; if a desired rule is unavailable, use the documented exact-SHA/tag
verification and release-owner confirmation. Before approving trusted signing,
reviewers inspect the source SHA printed in the validation job summary. See
[credential checklist](../releases/v0.1.0-release-credentials.md) for the
future Apple credential names and setup.

- Run the existing formatting, lint, typecheck, frontend test/build, Rust
  format/Clippy/test, dependency audits, RustSec, docs/link, and diff checks.
- Exercise Tauri bundle generation for release-profile `dmg` and `nsis` targets
  without credentials in PR CI. Verify expected paths/identity, ad-hoc macOS
  signature state, DMG integrity, unsigned Windows status and both checksums.
- Verify release workflow permissions are least-privilege, secrets are read
  only from a protected environment, PRs cannot access signing secrets, and
  release publication cannot occur from ordinary PR/develop validation.
- Inspect dependencies, bundled notices, package contents, signing output,
  checksums, and the release asset set.
- Review screenshots for local usernames, private paths, unrelated processes,
  or command data before committing.
- Complete read-only Sub-agent review of release security and README/license
  accuracy. Main Agent applies fixes and repeats checks/review as required.

## Gate B Manual Validation Plan

### macOS clean/isolated candidate validation

Use the exact unsigned release-profile DMG from the main-commit candidate
workflow. Record macOS version, Apple Silicon architecture/model category,
physical/VM/isolated environment, whether developer tools are installed, how
the DMG was transferred/downloaded, checksum, and whether quarantine/Gatekeeper
behavior was triggered. Install/copy, launch, record warnings and any normal
per-app System Settings approval offered; never disable security controls.
Exercise tray, listener discovery, process identity/icons, Search, Refresh,
Working Directory/Project Root, Copy/Open, safe process actions, Toast,
scroll/layout, quit/relaunch, remove, and reinstall.

Measure idle CPU/memory and representative scan duration per frozen NFR-006.
Run the packaged app for approximately eight continuous hours, periodically
using Refresh, Search/clear, listener appearance/disappearance, window/tray,
Copy and metadata inspection. Record start/end, duration, samples, refresh
observations, CPU/memory, errors/crashes, listener duplication/staleness, and
final condition. This is observational evidence, not a new threshold.

### Windows evidence

Automated Native Validation and release-profile unsigned NSIS packaging/checksum
are required. Interactive clean-machine validation and soak are **NOT RUN /
deferred**; selected testers may provide the first external installation
evidence. Do not represent CI compilation as interactive PASS.

Record the OS version, architecture, installer hash and actual results for all
performed checks. No Gatekeeper-success, notarization or signed status is
claimed for the unsigned candidate. README Dark Mode remains nonblocking and
deferred absent a known rendering defect.

## Current Implementation Evidence

- README, MIT metadata, ADR-007, project notices, draft release notes, and a
  tester guide are included in this R001 branch.
- `pnpm tauri build --bundles dmg` succeeded on the local Apple Silicon Mac
  and produced an unsigned local candidate at
  `src-tauri/target/release/bundle/dmg/Thaa_0.1.0_aarch64.dmg`. `hdiutil
verify` passed. The mounted app bundle reports version `0.1.0`, identifier
  `io.github.thg1rb.thaa`, and `LSMinimumSystemVersion` `15.0`.
- The locally built app has an ad-hoc signature with no Team ID. It is not
  Developer ID signed, notarized, or stapled. This is acceptable for the
  zero-budget candidate policy, but this local build has not passed Gate B and
  must not be distributed as the accepted release candidate.
- Local signing is unavailable: `security find-identity -v -p codesigning`
  reported zero valid identities. The explicit unsigned workflow path requires
  no secrets; the future trusted mode still fails closed unless its credentials
  are present.
- `README.md` accurately marks installers as not yet published and describes
  the selected package paths as planned, not available releases.
- No Git tag, GitHub Release, or tester distribution has been created. The
  unsigned local candidate remains only in the ignored Cargo target folder.
- The locally built DMG SHA-256 is
  `01812275b38a2d89afec6df25cb94789aa32695eefd75ff8885e871e5331fe6e`; this
  checksum is evidence for the local unsigned candidate only, not a release
  checksum.
- Local gates passed for this zero-budget update: Rust fmt, Clippy with
  `-D warnings`, Rust tests, frontend format/lint/typecheck, 48 frontend tests,
  production frontend build, pnpm audit, both RustSec target audits (with the
  two already documented allowed warnings), maintained documentation
  formatting, local Markdown links, Ruby YAML parsing, actionlint, and
  `git diff --check`.
- A prior release-workflow review found no blocking findings for the earlier
  signed-only design. The dedicated read-only review of this zero-budget
  update is now complete; it found and the implementation fixed the README
  checksum instructions and candidate artifact retention/provenance findings.
  Final-head CI is pending; earlier CI results do not validate the revised
  workflow.
- Earlier head `8a896a9edd159afe3d80f0487b700876327e238d` passed CI run
  `37287257327`, including Windows release-profile NSIS packaging, unsigned
  status, and checksum checks. This predates the zero-budget workflow changes
  and does not count as final-head CI. Fresh Shared Quality, macOS DMG, and
  Windows NSIS validation are pending.
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
- Gate B macOS clean/isolated install/reinstall, first-launch/quarantine
  observation, and NFR-006 idle/scan/8-hour soak measurements remain
  unperformed. Windows interactive install validation and soak are NOT RUN /
  deferred; Windows Working Directory/Project Root remain unsupported by the
  current provider. Developer ID, notarization, stapling, Gatekeeper trusted
  distribution and Authenticode are deferred release hardening.

## Read-only Review

The first read-only review found no defects in the MIT text/metadata, feature
claims, platform limitations, or README links. It identified outstanding R001
gates rather than implementation errors:

- The public GitHub Pre-release distribution for selected testers is now an
  explicit user decision; assets will be publicly downloadable.
- Rendered GitHub Dark Mode README review is still outstanding. Light Mode was
  inspected in the live PR; a sanitized real screenshot is included.
- Developer ID signing/notarization/stapling and Windows Authenticode are
  intentionally deferred; the user approved an unsigned preview for both
  platforms with explicit OS warning disclosures.
- The manual workflow and PR release-profile package validation were reviewed
  on earlier heads. This zero-budget update superseded their signed-only macOS
  assumptions. The latest read-only review checked this updated workflow,
  verified run/SHA/artifact binding and checksum handling, and found no
  remaining findings. Fresh final-head CI is still required. No candidate was
  run and no release workflow was dispatched.

The zero-budget read-only review also confirmed the README's per-artifact
checksum instructions, 90-day retention for both unsigned candidate artifacts,
cross-run download permissions, and the Gate B instruction to verify actual
`release-publish` environment protection before publication. No review
findings remain.

No code changes were required from review. The reviewer also confirmed that
project license statements and third-party notice boundaries are consistent.

The follow-up screenshot review found no personal path or unrelated listener
data. Its suggestion to remove the fixture PID and ambient-listener count was
applied to the screenshot; the stale screenshot-status text was reconciled.
The final README/license review found no remaining findings in that scope.
GitHub Light Mode rendering was visually checked; Dark Mode remains explicitly
deferred because the logged-out browser session cannot change GitHub's site
appearance.

R001 remains in progress. Gate A acceptance is pending. Gate B remains a later
post-promotion candidate-validation gate; paid signing credentials are not a
Gate A or zero-budget Gate B prerequisite.

## Final Acceptance

Pending. Gate A is ready for user acceptance only after the zero-budget
workflow, package CI, documentation, final review and final-head CI are green.
Gate B must then pass its macOS candidate, clean/isolated install and NFR-006
evidence before the exact candidate commit may be tagged and published. Paid
signing/notarization is deferred hardening, not a release gate.

## Stop Boundary

Keep the R001 PR open for user acceptance. Do not merge it, promote to `main`,
create the `v0.1.0` tag, publish a GitHub Release, distribute artifacts, or
begin W016 before the user explicitly accepts the completed R001 result.
