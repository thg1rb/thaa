# Continuous Integration

GitHub Actions validates pull requests targeting `develop`, commits pushed to
`develop`, and manual workflow runs. CI supplements the required read-only
Sub-agent review; it does not replace it. The workflow is validation-only and
requires no project secrets.

## Jobs and runners

| Job                       | Runner             | Checks                                                                                                                             |
| ------------------------- | ------------------ | ---------------------------------------------------------------------------------------------------------------------------------- |
| Shared quality            | `ubuntu-24.04` x64 | Frontend format, lint, typecheck, tests/build, local Markdown links, pnpm audit, RustSec audit for supported macOS/Windows targets |
| macOS native validation   | `macos-15` arm64   | Rust format/Clippy/tests including W008 controlled `lsof` listener tests, then a Tauri app build                                   |
| Windows native validation | `windows-2025` x64 | Rust format/Clippy/tests on Windows, then a Tauri app build                                                                        |

The hosted macOS image is macOS 15 arm64; it is not equivalent to the local
macOS 27 arm64 development host. Windows CI uses a GitHub-hosted Windows Server
2025 x64 image, not a self-hosted physical machine. Windows provider tests do
not exist until W009; the Windows job currently proves the existing shared
code and application build run on Windows.

The jobs use `.node-version`, `packageManager` in `package.json`, and
`rust-toolchain.toml` with the committed lockfiles. pnpm's store may be cached
by its lockfile-derived key. Build outputs are not shared between runners.
Each job prints toolchain versions so the run log identifies the environment.

## Workflow security

The workflow grants only `contents: read`. It does not use project secrets,
`pull_request_target`, shell commands built from PR values, release signing, or
write-capable actions. External Actions are pinned to full commit SHAs and
their upstream source, revision, purpose, and trust rationale are recorded
below. The repository does not currently enforce SHA pinning in GitHub
settings, so these pins are maintained in the workflow and reviewed in PRs.

| Action               | Upstream revision                                     | Purpose and trust rationale                                                              |
| -------------------- | ----------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| `actions/checkout`   | `3d3c42e5aac5ba805825da76410c181273ba90b1` (`v7.0.1`) | Official GitHub action to fetch the repository.                                          |
| `actions/setup-node` | `820762786026740c76f36085b0efc47a31fe5020` (`v7.0.0`) | Official GitHub action to install the Node version declared by the repository.           |
| `pnpm/action-setup`  | `ea17c68df8912ef543352723c149a84f56e3d413` (`v6.1.0`) | Maintained by the pnpm organization; reads the pinned pnpm version and caches its store. |

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

GitHub Actions is enabled and workflows use read-only token permissions.
Branch protection is not currently configured for `develop` or `main`, so
passing checks are not yet enforced by GitHub as merge requirements. This work
does not change branch rules; establish required checks after the baseline is
stable. Until then, the PR merge gate requires the main Agent to inspect the
current PR-head workflow runs and verify every required job passed.

No release, signing, artifact publishing, Linux product build, self-hosted
runner, or automated dependency update workflow is configured.
