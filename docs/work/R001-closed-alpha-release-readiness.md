# R001 — Closed Alpha Release Readiness

## Status

In progress — local macOS packaging and product documentation are prepared.
Apple signing/notarization credentials, Windows Authenticode credentials,
clean-machine installation evidence, performance measurements, rendered
GitHub review, tester-channel decision, final read-only review, and user
acceptance remain outstanding. README and license work are part of R001, not a
separate work item.

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
- Tauri bundling is enabled with application icon resources, but CI currently
  builds with `--no-bundle`.
- The repository has only validation CI. No release package, signing,
  notarization, artifact publishing, or GitHub Release workflow is configured.
- The GitHub repository is public. GitHub Release assets would be publicly
  downloadable; the exact selected-tester distribution visibility is pending
  product-owner clarification.
- No local Developer ID signing identity, GitHub `release` environment, or
  repository Actions secrets were found during initial inspection. Required
  signing credentials must be provisioned out of band; no secret values belong
  in this repository or this work record.
- The README previously had no screenshots or end-user installation guidance.
  CONTRIBUTING also incorrectly said product inspection features were not
  implemented.

## Decisions

- Target the first release at Apple Silicon (`aarch64`) on macOS 15 or later,
  packaged as a Developer ID signed, notarized, and stapled DMG.
- Target Windows 11 x64, packaged as an NSIS setup executable. Use Authenticode
  signing when a certificate is available; do not represent an unsigned
  installer as signed. If signing cannot be arranged, Windows tester
  distribution remains gated unless the product owner makes a separate
  explicit decision.
- Use the MIT License for Thaa with copyright year 2026 and the established
  public repository-owner handle `thg1rb`. Preserve third-party licenses and
  notices.
- The README tagline is “Know what's listening.” Its installation sections
  must say downloads are not available until the release is published.
- The intended version is `v0.1.0`; early-access status belongs in release
  metadata rather than a version suffix.
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

- [ ] Define and document whether selected testers receive publicly accessible
      GitHub pre-release assets or a restricted distribution.
- [ ] Provision protected Developer ID and notarization credentials and run a
      signed/notarized/stapled Apple Silicon DMG build.
- [ ] Produce the Windows 11 x64 NSIS installer; provision Authenticode
      credentials and sign it, or keep Windows distribution explicitly gated.
- [ ] Inspect signatures, notarization tickets, package contents, application
      identity/version/icon, and generated checksums.
- [ ] Define the `develop` → `main` release PR and post-promotion tag/release
      sequence. Do not execute that sequence during R001 pre-acceptance.
- [ ] Validate installation, first launch, permission/error handling, and
      uninstall/reinstall on clean macOS 15+ Apple Silicon and Windows 11 x64
      machines.
- [ ] Record NFR-006 measurements on both platforms and set only evidence-based
      budgets or claims.
- [ ] Prepare draft `v0.1.0` release notes and tester instructions.

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

- Run the existing formatting, lint, typecheck, frontend test/build, Rust
  format/Clippy/test, dependency audits, RustSec, docs/link, and diff checks.
- Exercise Tauri bundle generation for the selected `dmg` and `nsis` targets.
  The macOS DMG was generated locally; the Windows PR validation performs an
  unsigned debug NSIS package smoke build without uploading the installer.
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
  reported zero valid identities; no GitHub Actions release environment or
  repository Actions secrets are configured. Windows installer packaging and
  signing have not been run locally.
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
  validation, and NFR-006 performance measurements remain unperformed.

## Read-only Review

The first read-only review found no defects in the MIT text/metadata, feature
claims, platform limitations, or README links. It identified outstanding R001
gates rather than implementation errors:

- Selected-tester distribution visibility is unresolved; GitHub release assets
  in this public repository are publicly downloadable.
- Rendered GitHub Dark Mode README review is still outstanding. Light Mode was
  inspected in the live PR; a sanitized real screenshot is included.
- Developer ID signing/notarization/stapling and Windows Authenticode signing
  are not configured; clean-machine installation validation is outstanding.
- Windows bundle creation, platform performance measurements, and a release
  workflow are also outstanding.

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
