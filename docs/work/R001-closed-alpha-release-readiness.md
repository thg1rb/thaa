# R001 — Closed Alpha Release Readiness

## Status

**Current phase: TESTER-DRIVEN VALIDATION.** Gate A was accepted and integrated
into `develop`; the accepted release source is on `main`. `v0.1.0` was
published as a public GitHub Pre-release on 2026-10-05 from the accepted source
SHA `2edc9cc934517b935f14c85f606296a3539481b5`. Tester-driven Gate B has
started and is **not complete**. No manual tester evidence is recorded in this
work item yet. Historical planning and pre-publication status statements
below describe the state at that time; the [publication record](#v010-publication-and-tester-driven-validation-2026-10-05)
is authoritative for the current release state.

Branch: `feature/r001-closed-alpha-readiness`

Base: `develop` at `e03249aa1543aa08af72e96169ffed70205e53bf`

Target version: `v0.1.0` (release metadata should mark it as a pre-release;
do not add an alpha suffix to the version).

## Objective

Prepare the accepted macOS and Windows feature set for a first selected-tester
Early Preview. Candidate identity, packages, publication workflow, security
controls, and user-facing disclosures were validated before publication.
Current remaining work is to collect and triage interactive installation,
runtime, and stability evidence from selected testers without representing
unperformed checks as passed.

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
  an opt-in future path. Candidate run `37313991657` built from the exact
  `main` SHA recorded below; trusted signing and publish jobs were skipped.
- No local Developer ID identity, signing Actions secret, or signing variable
  is configured. No secret values belong in this repo. The separate
  `release-publish` environment was configured later; its current protection
  settings are recorded in the final publication section below.
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
  were prepared for the v0.1.0 publication; the public release-specific
  download and security instructions are in the release notes and tester guide.
- The intended release is public, marked Pre-release, and versioned/tagged
  `0.1.0` / `v0.1.0`; no alpha/beta suffix.
- The selected notarization mechanism is an App Store Connect API key.
- **Promotion sequence:** after R001 acceptance and integration into `develop`,
  prepare a separate reviewed PR from `develop` to `main`, require release
  validation, merge it, then create `v0.1.0` at the accepted `main` commit.
  Finally dispatch this workflow from `main` in `publish` mode. The
  `release-publish` environment gates publication. The separate `release`
  environment for future trusted signing is not configured or protected yet;
  configure its approval rules before adding signing credentials or selecting
  trusted mode. This sequence is documented only; no release action occurs in
  this task.
- User-approved NFR-006 evidence method is an approximately 8-hour continuous
  macOS soak. This is test methodology, not a new requirement threshold.
  Windows interactive measurements and soak are NOT RUN / deferred.
- The app bundle must advertise macOS 15.0 to match the selected Apple Silicon
  tester target. Package inspection found Tauri's default 10.13 minimum and
  `tauri.conf.json` now sets 15.0 and explicitly enables Hardened Runtime.
- Candidate-source release notes and a selected-tester guide live under
  `docs/releases/`. The candidate-source notes disclose the unsigned builds
  and planned official-release download path. The updated release-note draft
  below reflects the accepted candidate artifact names/hashes; it is on
  `develop` and is not the body automatically read by the frozen candidate
  source's publish workflow.

## Scope

- Complete Gate A's credential-free unsigned macOS DMG and Windows NSIS
  packaging paths, checksums, static release-workflow controls, documentation,
  read-only review, and final PR CI without publishing artifacts.
- Gate B is tester-driven after publication for this selected-tester Early
  Preview. The publication gate is the pre-publication boundary; interactive
  macOS and Windows validation, environment reports, NFR-006 observations, and
  stability evidence are collected from testers without being represented as
  pre-publication PASS.
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
license changes to third-party materials, retagging, replacing published
artifacts, or representing tester-driven evidence as complete. Release
publication was separately approved and completed as recorded below.

## Release Readiness Gates

### Gate A — PR #37 release infrastructure acceptance

- [x] Define public GitHub Pre-release distribution for selected testers.
- [x] Select Apple Silicon macOS 15+ unsigned/ad-hoc DMG for zero-budget v0.1.0.
- [x] Select Windows 11 x64 unsigned NSIS-only packaging; no MSI.
- [x] Implement explicit unsigned-default candidate/publish workflow while
      preserving trusted signing as an opt-in path.
- [x] Build/verify macOS DMG and Windows NSIS installer in credential-free PR
      CI; generate and verify SHA-256 checksums.
- [x] Define the reviewed `develop` → `main` release PR and post-promotion
      candidate, accepted-SHA tag, and manual-publication sequence. Do not
      execute it during Gate A.
- [x] Update README, release notes, tester guidance, test catalog, CI and work
      records with accurate unsigned warnings and deferred hardening.
- [x] Prepare the MIT license, release notes, tester guide and announcement.
- [x] Run local quality/security/docs/workflow static checks.
- [x] Re-review the corrected mounted-DMG verification path and updated release
      workflow.
- [x] Confirm all required CI on the final PR head after the latest
      documentation status update.
- [x] User explicitly accepted Gate A on 2026-10-05 at feature head
      `afa11eec0be72d517b4a1048b000ad64f3974cac`; scope is release
      infrastructure readiness, not Gate B or final `v0.1.0` acceptance.
- [x] Merge PR #37 into `develop`; merge commit and resulting head are
      `5955ae02d283728a0a611389789b468edcdc058f`.
- [x] Post-merge CI run `37304041326` passed Shared Quality, macOS Native
      Validation, and Windows Native Validation on that resulting `develop`
      head. The tree matches the PR-tested source tree; PR CI run `37303335329`
      passed macOS DMG and Windows NSIS packaging verification on the same
      tree.

Gate A does **not** require Apple Developer Program membership, Developer ID,
notarization, stapling, Authenticode, or actual release publication.

### Gate B — candidate evidence (historical candidate-generation stage)

- [x] Promote the accepted `develop` state through a separate reviewed
      `develop` → `main` PR; merge does not publish a release.
- [x] Run the manual non-publishing candidate workflow from the exact intended
      `main` commit; record its SHA and both artifact/checksum results.
- [x] Validate automated macOS 15+ Apple Silicon DMG integrity, mountability,
      arm64 executable, unsigned/ad-hoc state, version/icon inspection, and
      checksum for the retained candidate artifact.
- [x] Confirm Windows release-profile unsigned NSIS output, checksum, and
      automated CI; interactive clean-machine install and soak remain
      NOT RUN / deferred and are nonblocking for this selected-tester release.
- [x] Reclassify the previously planned manual checks as tester-driven
      post-publication evidence for this `v0.1.0` Early Preview. They remain
      pending; none is claimed as passed.

The candidate-generation checks are complete. Candidate generation is not
publication approval. Gate B manual/runtime evidence is now tester-driven
after publication. Paid Developer ID, notarization, stapling, and Authenticode
remain deferred release hardening.

#### Candidate run evidence — 2026-10-05

- Workflow run [37313991657](https://github.com/thg1rb/thaa/actions/runs/37313991657)
  completed successfully in manual `candidate` mode from `main`, with explicit
  macOS distribution mode `unsigned` and application version `0.1.0`.
- Exact candidate source SHA: `2edc9cc934517b935f14c85f606296a3539481b5`
  (PR #39 merge commit). Source validation and platform build jobs checked out
  that exact SHA.
- Retained workflow artifacts expire 2027-01-03. Candidate package identity:

  | Platform            | File                       | SHA-256                                                            | Candidate evidence                                                                                                                                                                        |
  | ------------------- | -------------------------- | ------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
  | macOS Apple Silicon | `Thaa_0.1.0_aarch64.dmg`   | `4365f2a0d91ef5865f3aaad953fdce004c80ed8425272af9ad2c5af8fb438297` | Workflow: DMG integrity/mount, app bundle, arm64 executable, ad-hoc signature with no Team ID, checksum PASS. Separate local inspection: bundle version `0.1.0` and `icon.icns` presence. |
  | Windows x64         | `Thaa_0.1.0_x64-setup.exe` | `f2503274b4a6d432b27c3203029f37126791ccf851f7900d6f092c45a9944073` | Release-profile NSIS build, unsigned status, and checksum PASS; interactive installation NOT RUN / deferred.                                                                              |

- Downloaded copies were checked against each artifact's `SHA256SUMS.txt`.
  Candidate artifacts are internal Actions workflow artifacts only, not public
  release assets. The trusted Developer ID/notarization job and Publish job
  were skipped; unsigned jobs did not access signing secrets. No tag, GitHub
  Release, or public asset was created. No Gatekeeper or clean-machine result
  is inferred from automated package checks.
- The workflow package-validation step does not itself assert the app bundle
  version or icon resource; those two observations came from the separate local
  inspection of the downloaded candidate, not from CI.
- At this historical evidence point, macOS first-launch/quarantine observation,
  clean/isolated installation, packaged regression, uninstall/reinstall, and
  NFR-006 measurements were not run. The later user decision reclassifies
  these as tester-driven post-publication evidence; Windows interactive
  validation and soak remain **NOT RUN / deferred / tester-driven**.

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
validation, runs only when selected, and reads signing secrets from the
`release` environment. That signing environment currently has no protection
rules; configure and verify those rules before setting secrets or using
trusted mode. The fixed `v0.1.0` publication path requires unsigned mode.
Windows remains unsigned NSIS.

Publish mode is a separate manual action. It requires the accepted candidate
SHA, candidate workflow run ID, and exact `v0.1.0` tag to resolve to that same
commit reachable from `main`. It confirms the candidate run was successful,
manual, on `main`, and contains the expected unsigned build artifacts; the
publish job verifies their original checksums and rechecks the tag immediately
before creating the public Pre-release. Pushing a tag alone cannot publish. Global permissions are
`contents: read`; only the publishing job receives `contents: write`. PR CI
does not trigger the release workflow or receive release credentials.

GitHub environments and tag rulesets are administrative configuration, not
repository files. `release-publish` has reviewer protection and is restricted
to `main`; the future signing environment `release` is not yet configured.
Before adding credentials or running trusted mode, configure `release` with
reviewer protection. GitHub rulesets are available on Free for public
repositories. Protect `v0.1.0` against unauthorized updates and deletion; if a
desired rule is unavailable, use exact-SHA/tag verification and release-owner
confirmation. Before approving trusted signing, reviewers inspect the source
SHA printed in the validation job summary. See
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

## Gate B Manual Validation Checklist (tester-driven after publication)

The following checklist is retained for testers. It is not a pre-publication
gate and remains unperformed until actual tester evidence is recorded.

### macOS clean/isolated candidate validation

Use the exact unsigned release-profile DMG from run `37313991657`; do not
substitute a local rebuild. Obtain the `macos-aarch64` workflow artifact from
the [candidate run](https://github.com/thg1rb/thaa/actions/runs/37313991657),
extract it, and verify `Thaa_0.1.0_aarch64.dmg` against its adjacent
`SHA256SUMS.txt` with `shasum -a 256 --check SHA256SUMS.txt`. Confirm the
digest matches the candidate record above before opening it. Record macOS
version, Apple Silicon architecture/model category, physical/VM/isolated
environment, whether developer tools are installed, how the artifact was
obtained, and whether quarantine/Gatekeeper behavior was triggered. Open the
DMG, copy Thaa.app to Applications, attempt first launch, and record the actual
OS message. If macOS offers normal per-application approval through System
Settings / Privacy & Security, use only that supported flow. Never disable
Gatekeeper, SIP, or other security controls globally; stop if managed policy
prevents launch. Exercise tray, listener discovery, process identity/icons,
Search, Refresh, Working Directory/Project Root, Copy/Open, safe Stop/Force
validation using a disposable process, Toast, scroll/layout, quit/relaunch,
remove, and reinstall.

Gate B macOS packaged regression checklist (leave unchecked until exercised
with this exact artifact):

- [ ] Candidate artifact and SHA-256 verified.
- [ ] DMG opened; Thaa.app installed/copied.
- [ ] First launch attempted; actual OS warning/block and normal approval path
      recorded.
- [ ] Tray and Runtime Inspector open/close behavior.
- [ ] Listener discovery and process identity.
- [ ] Native/fallback process icons.
- [ ] Search by Process Name and exact Port; clear Search.
- [ ] Refresh while Search is active.
- [ ] Working Directory and Project Root.
- [ ] Copy URL, Copy Port, Copy PID, and Open URL.
- [ ] Stop and Force Stop against a safe disposable test process; confirmation
      target checked.
- [ ] Toast, scrollbar/layout, vertical scrolling, and no horizontal overflow.
- [ ] Quit and relaunch.
- [ ] Uninstall/remove and reinstall.
- [ ] Record issues and environment; do not infer PASS from CI packaging.

Optional tester evidence: measure idle CPU/memory and representative scan
duration per frozen NFR-006. Power testers may run the packaged app for
approximately eight continuous hours, periodically using Refresh,
Search/clear, listener appearance/disappearance, window/tray, Copy and
metadata inspection. Record start/end, duration, samples, refresh observations,
CPU/memory, errors/crashes, listener duplication/staleness, and final
condition. This is observational evidence, not a new threshold, and is not
required from every tester or before publication.

### Windows evidence

Automated Native Validation and release-profile unsigned NSIS packaging/checksum
are required. Interactive clean-machine validation and soak are **NOT RUN /
deferred**; selected testers may provide the first external installation
evidence. Do not represent CI compilation as interactive PASS.

Record the OS version, architecture, installer hash and actual results for all
performed checks. No Gatekeeper-success, notarization or signed status is
claimed for the unsigned candidate. README Dark Mode remains nonblocking and
deferred absent a known rendering defect.

### NFR-006 measurement and soak evidence template

Re-read and apply the frozen NFR-006 wording above. This template records
observations without adding numeric product thresholds. Use the packaged
candidate identified in the run evidence above.

```text
Candidate source SHA: 2edc9cc934517b935f14c85f606296a3539481b5
Artifact: Thaa_0.1.0_aarch64.dmg
Artifact SHA-256: 4365f2a0d91ef5865f3aaad953fdce004c80ed8425272af9ad2c5af8fb438297

Environment:
macOS version:
Architecture/model category:
Physical / VM / isolated environment:
Developer tools installed (yes/no):
Artifact transfer/download method:

Idle observation duration:
Runtime Inspector state:
Listener/process count (approximate):
Measurement tool/method:
CPU observations:
Memory observations:

Scan/Refresh measurement boundaries:
Refresh samples and durations:

Soak start (date/time/time zone):
Soak end (date/time/time zone):
Total continuous duration:
Initial CPU:
Initial memory:

Intermediate observations:
- Time:
  CPU:
  Memory:
  Interaction:
  Refresh/scan observation:
  Result:

Final CPU:
Final memory:
Crashes:
UI hangs:
Duplicate/stale listeners:
Refresh failures:
Tray failures:
Other defects:
Final condition:
Result: PASS / FAIL (against frozen NFR-006; do not invent thresholds)
```

## Pre-publication Implementation Evidence (historical)

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
- The ad-hoc linker signature is on the arm64 executable; the unsigned app
  bundle has no sealed resource directory, so `codesign --verify` reports that
  bundle resources are absent. CI validates the DMG container/checksum, mounts
  it read-only, and checks executable architecture plus ad-hoc/no-Team-ID
  signature metadata. This does not claim Developer ID trust or notarization.
- Local signing is unavailable: `security find-identity -v -p codesigning`
  reported zero valid identities. The explicit unsigned workflow path requires
  no secrets; the future trusted mode still fails closed unless its credentials
  are present.
- `README.md` accurately marks installers as not yet published and describes
  the selected package paths as planned, not available releases.
- The Gate B candidate workflow has produced retained Actions artifacts; these
  are not public release assets and have not been distributed to testers. No
  Git tag or GitHub Release has been created.
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
  At that point in the validation sequence, final-head CI was still pending;
  the subsequent successful final-head runs are recorded below.
- CI run `37295364882` passed Shared Quality and Windows Native Validation,
  including release-profile unsigned NSIS packaging and checksum verification.
  The macOS DMG built successfully, but its verification step initially failed
  because Tauri removes its staging app bundle after packaging. The mounted-DMG
  correction then exposed an Xcode `lipo` argument-order mismatch in run
  `37297128392`; the DMG integrity and read-only mount passed before that
  architecture check failed. Local reproduction showed the unsigned
  linker-signed executable has no sealed app resources, so bundle-level
  `codesign --verify` is not applicable; the package check confirms the arm64
  executable, ad-hoc signature metadata, DMG integrity, and SHA-256. The Xcode
  argument order is corrected and passed read-only follow-up review. CI run
  `37298088535` passed Shared Quality, macOS Native Validation (including the
  mounted-DMG architecture/signature/checksum step), and Windows Native
  Validation including release-profile NSIS packaging, on head
  `fa995a0606f07fb6e5fb22a3ed6c530097cbb0d9`. Subsequent docs-only heads
  `fead75cfd90a50ee9341ee6519657d83beb478e9` and
  `4cbc71a30b51f8d526560b08e2957f23d3dfe746` also passed Shared Quality,
  macOS Native Validation, and Windows Native Validation in runs
  `37298810275` and `37300608805`, respectively. The latter includes the final
  checklist synchronization. At that point no candidate had yet been run; the
  later candidate run is recorded in the Gate B section above.
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
- macOS clean/isolated install/reinstall, first-launch/quarantine observation,
  and NFR-006 idle/scan/8-hour soak measurements remain unperformed and are
  tester-driven after publication. Windows interactive install validation and
  soak are NOT RUN / deferred / tester-driven; Windows Working Directory and
  Project Root remain unsupported by the current provider. Developer ID,
  notarization, stapling, Gatekeeper trusted distribution, and Authenticode
  are deferred release hardening.

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
- The zero-budget review checked the README checksum guidance and candidate
  run/SHA/artifact binding, and the retention mismatch it found was fixed. A
  follow-up review checked the 90-day retention. The subsequent CI run exposed
  a staging-app-path assumption in DMG verification; the first correction
  review identified best-effort cleanup handling, and local reproduction
  showed bundle-level `codesign --verify` is not valid for this intentionally
  unsealed unsigned app. CI then identified an Xcode `lipo` argument-order
  mismatch after successfully mounting and inspecting the DMG. The corrected
  argument order passed read-only follow-up review, and CI run `37298088535`
  passed all three required checks on head `fa995a0606f07fb6e5fb22a3ed6c530097cbb0d9`.
  The final checklist synchronization is recorded in run `37300608805` on head
  `4cbc71a30b51f8d526560b08e2957f23d3dfe746`; all three required CI jobs
  passed. Those Gate A review/CI records predate the candidate workflow run
  recorded above.

The read-only review confirmed the README's per-artifact checksum
instructions, 90-day candidate artifact retention, cross-run download
permissions, and the Gate B instruction to verify actual `release-publish`
environment protection before publication. Its follow-up review found no
blocking issues in mounted-DMG inspection, executable signature metadata,
architecture validation, or cleanup behavior.

No code changes were required from review. The reviewer also confirmed that
project license statements and third-party notice boundaries are consistent.

The focused read-only review of candidate run `37313991657` found no blockers.
It confirmed exact source/run binding, unsigned artifacts and checksum matches,
skipped trusted/publish jobs, and no tag or release. It noted that workflow
validation does not assert bundle version/icon; the candidate record now
distinguishes those observations from CI by attributing them to the separate
local inspection of the downloaded macOS artifact. Manual Gate B checks remain
pending.

The follow-up screenshot review found no personal path or unrelated listener
data. Its suggestion to remove the fixture PID and ambient-listener count was
applied to the screenshot; the stale screenshot-status text was reconciled.
The final README/license review found no remaining findings in that scope.
GitHub Light Mode rendering was visually checked; Dark Mode remains explicitly
deferred because the logged-out browser session cannot change GitHub's site
appearance.

At Gate A completion, R001 remained in progress. Gate A was accepted by the
user on 2026-10-05 at feature head
`afa11eec0be72d517b4a1048b000ad64f3974cac`, then merged by PR #37 as commit
`5955ae02d283728a0a611389789b468edcdc058f`. Post-merge CI passed on that
`develop` head; its tree is identical to the source tree that passed PR
packaging CI. PR #39 promoted that accepted release source to `main` at merge
commit `2edc9cc934517b935f14c85f606296a3539481b5`. Candidate run `37313991657`
succeeded from that exact SHA. Publication and current tester-driven Gate B
state are recorded below. Paid signing credentials are not a Gate A or
zero-budget release prerequisite.

## Gate A Acceptance Record

Accepted. The user accepted Gate A at feature head
`afa11eec0be72d517b4a1048b000ad64f3974cac` after the zero-budget workflow,
macOS unsigned/ad-hoc DMG and Windows unsigned NSIS packaging, SHA-256
verification, source pinning, workflow security, release/tester documentation,
deferred paid-signing strategy, read-only review, and final-head CI passed.
At that point, the acceptance covered Gate A only and did not authorize
publication. The later user decision made manual installation, packaged
regression, NFR-006 runtime observations, and the approximately eight-hour soak
tester-driven post-publication evidence for the selected-tester Early Preview.
Those items remain unperformed until actual reports are recorded. Paid
signing/notarization is deferred hardening, not a release gate.

## Gate A Integration Record (at Gate A completion)

- PR: #37, `feature/r001-closed-alpha-readiness` → `develop`, merged using a
  merge commit.
- Merge commit and resulting `develop` head:
  `5955ae02d283728a0a611389789b468edcdc058f`.
- Post-merge run `37304041326` passed Shared Quality, macOS Native Validation,
  and Windows Native Validation. The merged tree is identical to PR head
  `8e0fa40eb92b3d4d30420ab38465090e3f3e0e51`; PR run `37303335329` passed
  unsigned macOS DMG and Windows NSIS packaging checks on that tree.
- PR #39 (`develop` → `main`) merged as `2edc9cc934517b935f14c85f606296a3539481b5`.
- Candidate run `37313991657` built the unsigned DMG and NSIS installer from
  that exact `main` commit; automated package/checksum validation passed.
- At that candidate-generation point, the `v0.1.0` tag and GitHub Release had
  not yet been created. Their final state is recorded below.

## Tester-driven Gate B policy and pre-publication plan (2026-10-05)

This section records the approved validation model and the publication gate
before execution. Its candidate and publication statuses are historical; the
actual release outcome is recorded in the post-publication section below.

The user explicitly changed the `v0.1.0` validation model: publication as a
GitHub Pre-release for selected/early testers may precede interactive manual
Gate B evidence. Tester use becomes the source of environment-specific
installation, first-launch, regression, and stability evidence. This policy is
specific to this Early Preview and does not weaken future stable-release
criteria. Any item below marked tester-driven or deferred remains unperformed
until evidence is received.

### Frozen candidate identity

| Field                  | Accepted value                                                                                                         |
| ---------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| Version / planned tag  | `0.1.0` / `v0.1.0`                                                                                                     |
| Candidate source SHA   | `2edc9cc934517b935f14c85f606296a3539481b5`                                                                             |
| Candidate workflow run | [37313991657](https://github.com/thg1rb/thaa/actions/runs/37313991657), candidate mode, explicit `unsigned` macOS mode |
| macOS artifact         | `Thaa_0.1.0_aarch64.dmg` — SHA-256 `4365f2a0d91ef5865f3aaad953fdce004c80ed8425272af9ad2c5af8fb438297`                  |
| Windows artifact       | `Thaa_0.1.0_x64-setup.exe` — SHA-256 `f2503274b4a6d432b27c3203029f37126791ccf851f7900d6f092c45a9944073`                |

These exact candidate artifacts are to be reused. Do not rebuild them in
publish mode. A source or binary change requires a new version/candidate cycle;
never move or replace a published `v0.1.0` artifact. Candidate Actions
artifacts expire **2027-01-03**; confirm their retention/availability before
publication. The public release should attach the workflow-generated checksum
manifest. The release-notes draft on the candidate source already contains the
Early Preview and unsigned-platform disclosures. PR #41 on `develop` changes
only internal evidence documentation; it is not required for runtime release
safety and must not be promoted before this candidate is published. The
expanded release-notes draft edited in this task is on `develop`; the current
publish workflow checks out the tagged candidate SHA and uses the notes file
from that source instead. That frozen-source notes body already contains the
necessary Early Preview/platform-warning language. Use the actual body from
the workflow when approving publication; do not assume the expanded draft is
automatically attached.

### `v0.1.0` Publication Gate

Every item below is a mandatory pre-publication check. A user decision to
publish is still required after this gate passes.

| Classification                     | Check                             | Evidence / status                                                                                                                                                                                                                                                                                                   |
| ---------------------------------- | --------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| PASS — REQUIRED BEFORE PUBLICATION | Candidate source identity         | SHA `2edc9cc934517b935f14c85f606296a3539481b5` is the accepted `main` source; verify it remains unchanged and has no release-blocking defect immediately before publication.                                                                                                                                        |
| PASS — REQUIRED BEFORE PUBLICATION | Candidate workflow                | Run `37313991657` succeeded in candidate mode, pinned exact SHA, unsigned mode; trusted-signing and publish jobs skipped.                                                                                                                                                                                           |
| PASS — REQUIRED BEFORE PUBLICATION | Candidate artifacts and checksums | Both listed artifact digests verified; macOS DMG package checks passed and Windows release-profile x64 NSIS packaging/checksum passed. Reverify exact files and manifest before attaching.                                                                                                                          |
| PASS — REQUIRED BEFORE PUBLICATION | Applicable CI and security        | Candidate-era Shared Quality, macOS/Windows Native Validation, workflow/static checks, package checks, dependency/security checks and reviews have no unresolved release blocker; recheck current run/source status before publication. Two previously accepted RustSec advisories remain disclosed in this record. |
| PASS — REQUIRED BEFORE PUBLICATION | User-facing disclosure            | Draft release notes state Early Preview, unsigned/not-notarized macOS and unsigned Windows, OS warnings/blocks, supported platforms, limitations, checksums, process-action caution, and issue reporting. Verify exact attached release body at publish time.                                                       |
| PASS — REQUIRED BEFORE PUBLICATION | Tester instructions and triage    | Three-level tester plan, issue feedback template, safety guidance, known limitations, and post-publication evidence tracking are documented below and in `docs/releases/`.                                                                                                                                          |
| PASS — REQUIRED BEFORE PUBLICATION | No known release-blocking defect  | Recheck open findings and candidate evidence. No known defect currently indicates unusable/corrupt packages, wrong source/artifact, unsafe process termination, or serious security risk.                                                                                                                           |
| PASS — REQUIRED BEFORE PUBLICATION | Publication workflow safety       | Confirm publish mode reuses the accepted candidate artifacts, verifies candidate run/source/tag/checksums, does not rebuild platform packages, creates a GitHub Pre-release, and uses least-privilege publication permission. Candidate mode cannot publish.                                                        |
| PASS — REQUIRED BEFORE PUBLICATION | Tag safeguards                    | Before publication, ensure tag does not exist, target exact SHA, and use configured tag protection if available; otherwise apply exact-SHA verification and procedural no-force-update/no-reuse rule. See Tag Protection below.                                                                                     |
| PASS — REQUIRED BEFORE PUBLICATION | Final release authorization       | Explicit user approval was given; the exact candidate was tagged and published as recorded in the post-publication record below.                                                                                                                                                                                    |

Do not classify missing interactive validation as PASS. Classification of
post-publication work is:

- **TESTER-DRIVEN AFTER PUBLICATION:** macOS and Windows install/first launch,
  interactive platform behavior, functional regression, environment-specific
  compatibility and feedback collection.
- **DEFERRED RELEASE HARDENING:** Developer ID, Apple notarization/stapling,
  trusted Gatekeeper distribution, and Windows Authenticode.
- **BLOCKING:** any newly found genuine package/source/security defect, failed
  mandatory publication check, or missing explicit user publication approval.

### Tester-driven Gate B after publication

For macOS, request reports on install, first-launch/security-warning behavior,
tray, Runtime Inspector, listener/process identity and icons, Search, Refresh,
Working Directory, Project Root, Copy/Open, safe process actions, Toast,
scroll/layout, quit/relaunch, uninstall/reinstall, NFR-006 observations, and
optional long-run stability. These are **not pre-publication PASS claims**.
The approximately **8-hour continuous soak** remains optional extended tester
methodology, not a mandatory publication gate or a new NFR threshold.

For Windows, interactive install/regression and soak remain **NOT RUN /
tester-driven / deferred**. Automated release-profile NSIS and Native
Validation passed. Windows Working Directory and consequently Project Root
are unavailable in the current provider; generic graceful Stop is unsupported.
SmartScreen may warn, Smart App Control or managed policy may block
installation. Testers must not disable system security controls.

### Tester evidence tracking

Record each useful tester report with these fields; testers may omit checks
they did not perform:

```text
Platform:
Environment (OS/version/architecture; no identifying details):
Install:
First launch:
Security warning/block:
Core regression:
Extended regression:
Long-run stability:
Issue references:
Outcome:
```

Triage outcomes:

- **Critical / release safety:** security issue, destructive process-action
  behavior, corrupt package, or severe system instability. Consider hiding the
  release while investigating.
- **Significant:** major capability unusable, frequent crash, or broad install
  failure. Fix through a new version (normally `0.1.1`); never retag or replace
  `v0.1.0` binaries.
- **Normal Early Preview defect:** record and prioritize through the normal
  development backlog.

### Tag protection and publish review

A no-cost repository ruleset is configured for `refs/tags/v0.1.0` with update
and deletion restrictions, no bypass actors, and `current_user_can_bypass:
never`. It does not prevent initial tag creation. The `release-publish`
environment is configured with required review by `thg1rb`, no administrator
bypass, and deployments restricted to `main`. GitHub reports
`prevent_self_review: false`; the only repository collaborator is the owner,
so the owner can approve the protected deployment. These settings are
administrative controls and do not create the tag or release. If protection
ever becomes unavailable, preserve procedural immutability: confirm the exact
SHA, do not force-update or reuse the tag, and use a new version for changed
binaries. Before publication, the ruleset and candidate were verified; the
created tag resolves to `2edc9cc934517b935f14c85f606296a3539481b5`. Publish
mode reused the exact candidate artifacts and verified their checksums without
rebuilding them. The Pre-release is not presented as “Latest stable.”

### Final publication stop boundary

This section records the stop boundary immediately before publication. The
actual publication outcome and current R001 state are recorded below.

## v0.1.0 publication and tester-driven validation (2026-10-05)

The user explicitly approved publication of the exact accepted candidate.
Thaa `v0.1.0` was published as a public GitHub **Pre-release** for selected / early
testers. The release is an Early Preview, not a stable or production-ready
release. This record supersedes the pre-publication status statements above;
those statements describe the state before approval and publication.

### Release identity and artifact verification

| Field                             | Published value                                                                                                  |
| --------------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| Tag                               | [`v0.1.0`](https://github.com/thg1rb/thaa/releases/tag/v0.1.0)                                                   |
| Tag target / candidate source SHA | `2edc9cc934517b935f14c85f606296a3539481b5`                                                                       |
| Candidate workflow                | [37313991657](https://github.com/thg1rb/thaa/actions/runs/37313991657), candidate mode, explicit `unsigned` mode |
| Publish workflow                  | [37331918775](https://github.com/thg1rb/thaa/actions/runs/37331918775), successful                               |
| Release classification            | Public GitHub Pre-release; not draft                                                                             |
| macOS asset                       | `Thaa_0.1.0_aarch64.dmg` — SHA-256 `4365f2a0d91ef5865f3aaad953fdce004c80ed8425272af9ad2c5af8fb438297`            |
| Windows asset                     | `Thaa_0.1.0_x64-setup.exe` — SHA-256 `f2503274b4a6d432b27c3203029f37126791ccf851f7900d6f092c45a9944073`          |
| Checksum manifest                 | `SHA256SUMS.txt`; both published assets verify against it                                                        |

The publish workflow retrieved the accepted candidate artifacts from run
`37313991657`, verified their digests, and assembled the release assets. It did
not run platform build jobs or trusted signing. The downloaded public release
assets match the accepted candidate hashes exactly. The annotated tag peels to
the accepted source SHA; the active ruleset blocks tag update and deletion.
`main` remains at the candidate source SHA. No artifact rebuild or replacement
was performed.

The read-only post-publication integrity review passed: tag/source binding,
candidate reuse, hashes, release classification, and expected assets were
verified. It found stale publication wording in the linked tester guide and
announcement. Those documents are corrected on a documentation follow-up
branch; a read-only re-review of those corrections is required before that
documentation PR is considered complete. No source or binary change is
required, and no tester message has been sent.

### Tester-driven Gate B state

Tester-driven Gate B **STARTED** upon publication and is **NOT COMPLETE**.
No tester reports or manual installation evidence are recorded yet. In
particular, macOS first-launch/Gatekeeper behavior, packaged interactive
regression, NFR-006 idle and scan observations, and an optional approximately
8-hour stability soak remain unperformed and are requested as tester evidence.
The eight-hour duration is a test method, not a product threshold. Windows
interactive installation/regression and soak remain `NOT RUN / deferred / tester-driven`.
Do not report any of these as PASS until actual evidence is received.

Developer ID, Apple notarization, stapling, trusted Gatekeeper distribution,
and Windows Authenticode remain deferred release hardening. The known Windows
Working Directory and Project Root limitations and unsupported generic
Graceful Stop remain disclosed. Two previously accepted RustSec advisories
remain documented; no new release-blocking security finding was reported by
the publication review.

R001 transitions to **TESTER-DRIVEN VALIDATION** and is not fully closed. The
tester guide, feedback template, and prepared announcement are available under
`docs/releases/`. The announcement is prepared but has not been sent
externally. Tester evidence should be recorded by platform, environment,
installation/first-launch results, core or extended regression, stability,
issue references, and outcome. Critical release-safety findings should be
triaged promptly; ordinary defects must use a new version rather than moving
the immutable `v0.1.0` tag or replacing its binaries.

The post-publication documentation correction is limited to `develop`; it
does not change `main`, the tag, or published binaries. W016 remains unstarted.
