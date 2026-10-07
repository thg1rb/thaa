# Continuous Integration

GitHub Actions validates pull requests targeting `develop` and `main`, commits
pushed to `develop` and `main`, and manual workflow runs. CI supplements the required read-only
Sub-agent review; it does not replace it. Ordinary CI is validation-only and
requires no project secrets. The separate R001 release workflow is described
below; it is not triggered by pull requests or ordinary `develop` pushes.

## Jobs and runners

| Job                       | Runner             | Checks                                                                                                                                                              |
| ------------------------- | ------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Shared quality            | `ubuntu-24.04` x64 | Frontend format, W013 docs format, lint, typecheck, tests/build, local Markdown links, pnpm audit, RustSec audit for supported macOS/Windows targets                |
| macOS native validation   | `macos-15` arm64   | Rust format/Clippy/tests including W008 controlled listeners and W013 native snapshot integration, then a Tauri app build                                           |
| Windows native validation | `windows-2025` x64 | Rust format/Clippy/tests including W013 native snapshot integration on Windows; PR builds release-profile unsigned NSIS, develop/manual builds app without bundling |

The hosted macOS image is macOS 15 arm64; it is not equivalent to the local
macOS 27 arm64 development host. Windows CI uses a GitHub-hosted Windows Server
2025 x64 image, not a self-hosted physical machine. Windows CI is the
authoritative native environment for Windows-only provider tests and the
application build; a cross-target compile from another OS is not a substitute.

The jobs use `.node-version`, `packageManager` in `package.json`, and
`rust-toolchain.toml` with the committed lockfiles. pnpm's store may be cached
by its lockfile-derived key. Rust dependencies and build artifacts are cached
with `Swatinem/rust-cache` separately on each operating system. The action
targets `src-tauri/target`; its key includes the runner OS/architecture, Rust
toolchain, Cargo manifests and lockfile, and explicit cache key where used.
Pull requests can restore a compatible base-branch cache and same-repository
PR runs may save only to their GitHub `refs/pull/<number>/merge` cache scope.
GitHub does not make these PR-scoped entries available to `develop` or other
PRs. Fork PRs do not save caches. Only trusted pushes to `develop` save the
develop-scoped cache. Manual runs are cache readers. This follows [GitHub's
cache scope and security model](https://docs.github.com/en/actions/reference/workflows-and-actions/dependency-caching).
A cache miss does not skip any check and must build successfully from the
lockfile.

macOS pull requests run the debug no-bundle app build and the release-profile
`scripts/package-macos-release.sh` path. Tauri signs the completed app bundle
with the explicit ad-hoc identity `-`. CI runs strict `codesign --verify
--deep --strict` before DMG creation and again against the exact app mounted
from the final DMG. It checks the sealed `CodeResources`, `Info.plist`
signature coverage, arm64 executable, version, icon, key bundle-file hashes,
DMG integrity, and checksum. A failure at either signature gate fails CI.
`syspolicy_check` is diagnostic: expected ad-hoc and missing-notarization
findings do not claim Apple trust or notarization. This automated package check
does not claim interactive installation or Gatekeeper acceptance.
Trusted `develop` pushes/manual runs
use `pnpm tauri build --no-bundle`. The Windows PR job builds
`pnpm tauri build --bundles nsis` in release profile and checks the unsigned
`Thaa_0.1.1_x64-setup.exe` output and SHA-256 manifest; this is
packaging/compilation evidence, not interactive Windows installation or
SmartScreen visual validation. Windows develop/manual runs use
`pnpm tauri build --no-bundle`.
W017 adds controlled-process resource assertions to the existing macOS and
Windows provider test suites and shared runtime/unit tests. Windows resource
API behavior is validated on the Windows native runner; local cross-target
checking may require the Windows resource compiler and is not treated as
native validation.
Native formatting, Clippy, Rust tests, and provider integration tests remain
required on pull requests. Frontend lint, typecheck, unit tests, and
documentation checks run once in Shared Quality; native jobs still install
frontend dependencies because Tauri invokes the frontend production build.

The Shared Quality job also caches the pinned `cargo-audit` 0.22.2 binary and
its dependencies. It verifies the binary version on each run and installs that
exact version if the cache is cold. Both RustSec target audits still execute;
the cache affects setup time only. Each job prints runner and toolchain
versions so the run log identifies the environment. Build outputs are never
shared between operating systems.

### Performance baseline and measurement

The W008.1 baseline comes from [CI run 37093143837](https://github.com/thg1rb/thaa/actions/runs/37093143837),
which validated PR #20 at `963a01bfb51718c6f116022240489ca09b758b70`. The
Windows job took 9m52s total (Clippy 2m19s, tests 1m39s, Tauri release build
4m52s); the macOS job took 5m10s (Clippy 1m02s, tests 51s, Tauri release
build 2m48s). The Shared Quality job took 3m32s, including 2m49s to compile
`cargo-audit`; pnpm installation took about 2s in the native jobs and was not
the bottleneck. W008.2's [work item](../work/W008.2-ci-performance-hardening.md)
records cold PR and integrated develop cache-seed runs, plus three sequential
warm release-profile runs. Warm median job times were 2m55s on macOS, 3m59s
on Windows, and 39s for Shared Quality. Warm median release Tauri build steps
were 95s on macOS and 84s on Windows. These are observations from hosted
runners, not guaranteed timings; the work item links every source run and
reports cache restoration and runner variance.

## Workflow security

The workflow grants only `contents: read` and does not persist the checkout
credential in local Git configuration. It does not use project secrets,
`pull_request_target`, shell commands built from PR values, release signing, or
actions with repository write permission. The Rust cache action only writes
the isolated cache scopes allowed by its event condition. External Actions are pinned to full commit SHAs and
their upstream source, revision, purpose, and trust rationale are recorded
below. The repository does not currently enforce SHA pinning in GitHub
settings, so these pins are maintained in the workflow and reviewed in PRs.

| Action                      | Upstream revision                                     | Purpose and trust rationale                                                                                                                                                     |
| --------------------------- | ----------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `actions/checkout`          | `3d3c42e5aac5ba805825da76410c181273ba90b1` (`v7.0.1`) | Official GitHub action to fetch the repository.                                                                                                                                 |
| `actions/setup-node`        | `820762786026740c76f36085b0efc47a31fe5020` (`v7.0.0`) | Official GitHub action to install the Node version declared by the repository.                                                                                                  |
| `pnpm/action-setup`         | `ea17c68df8912ef543352723c149a84f56e3d413` (`v6.1.0`) | Maintained by the pnpm organization; reads the pinned pnpm version and caches its store.                                                                                        |
| `Swatinem/rust-cache`       | `6323deb102c322ba6fcbdcafc7e3dddab59af2b6` (`v2.9.2`) | Rust cache action maintained by Swatinem; follows its release commit, keeps OS/toolchain/cache inputs isolated, and stores no secrets. Official Tauri CI also uses this action. |
| `actions/upload-artifact`   | `043fb46d1a93c77aae656e7c1c64a875d1fc6a0a` (`v7.0.1`) | Official GitHub action used to retain short-lived release-candidate artifacts and checksums.                                                                                    |
| `actions/download-artifact` | `3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c` (`v8.0.1`) | Official GitHub action used by the gated publisher to assemble artifacts from the same workflow run.                                                                            |

Rust dependency advisories use RustSec `cargo-audit` version `0.22.2`,
installed with Cargo's exact-version and lockfile options. The audit runs for
macOS arm64 and Windows x64 because Linux is not a supported product target.
Advisories are not silently ignored. Cargo-audit's default policy fails for
vulnerabilities; allowed informational warnings remain visible in the logs
and must be reviewed. The current lockfile emits warnings for unmaintained
`proc-macro-error` and the `glib::VariantStrIter` soundness advisory; `cargo
tree` shows neither crate in the supported macOS arm64 or Windows x64 target
graphs. These transitive lockfile warnings are recorded rather than
suppressed. JavaScript auditing uses `pnpm audit --audit-level high`; findings
at High or Critical fail CI, while lower severity remains subject to W004
triage and review.

## Local equivalents

Run the local quality commands in [Development](DEVELOPMENT.md). In addition,
`pnpm docs:check` verifies that relative Markdown link targets exist. The link
checker does not validate external URLs or heading fragments. Rust remains
formatted with `cargo fmt`; Prettier does not format Rust.

## Required status checks and limitations

GitHub Actions is enabled and ordinary CI uses read-only token permissions.
The `main` and `develop` branch rulesets require pull requests and the `Shared
quality`, `macOS native validation`, and `Windows native validation` checks;
they block force-pushes and deletion and have no bypass actors. The dedicated
read-only Sub-agent review remains a separate manual merge gate. GitHub
approval count is not required because the repository has one maintainer.

The separate `.github/workflows/release.yml` is manually dispatched from
trusted `main`; pushes, merges, and tag creation do not publish. Candidate mode
builds the exact dispatched main commit without publishing; macOS defaults to
an explicit ad-hoc bundle-signing path, while an opt-in trusted candidate path
retains Developer ID/notarization support. The current publish mode is fixed
to v0.1.1, verifies the accepted source and tag, reuses the candidate
artifacts, and grants `contents: write` only to the publishing job. Future
versions require deliberate workflow and release-note updates through the
reviewed promotion process. See the authoritative [Release Workflow](../releases/RELEASE-WORKFLOW.md)
for candidate invalidation, tag/artifact identity, post-release
reconciliation, and the release checklist. Linux product builds, self-hosted
runners, and automated dependency-update workflows remain unconfigured.
