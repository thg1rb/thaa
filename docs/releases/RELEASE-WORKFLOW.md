# Thaa Release Workflow

This is the authoritative branch and release process for Thaa. The v0.1.1
release is historical and immutable; this policy governs future releases and
post-release repository work.

## Branch roles

- `main` is the accepted release line. A release candidate must be generated
  from an exact commit on `main`.
- `develop` integrates reviewed work and is the source of release promotion.
- `feature/*`, `fix/*`, `docs/*`, and other task branches start from
  `develop` and enter it through a pull request.
- Do not develop directly on `main` or `develop`. Do not introduce a
  `release/*` branch or direct-to-main hotfix path without a separate
  decision.

Normal work follows task branch → pull request to `develop` → required CI →
dedicated read-only Sub-agent review → merge. Before major release or
reconciliation PRs, use the same read-only review gate. The reviewer reports
findings only; the primary Agent applies fixes.

## Release promotion and freeze

Every release is promoted from `develop` to `main` through a reviewed Release
PR. The Release PR review checks scope, version, source consistency, packaging,
signing policy, CI, security, release documentation, and unrelated work.
Merging that PR does not itself publish a release.

When the release scope is frozen for `develop → main` promotion and a candidate
is generated, the Release Freeze applies. The accepted candidate source SHA
must equal the intended `main` source SHA. Release-affecting changes include
application or packaging code, signing configuration, release workflows,
release safety CI, security or dependency changes, version metadata, bundled
resources, runtime configuration, and required public release documentation.
Impact, not filename alone, determines whether a change affects the release.

Any required release-affecting change after candidate generation invalidates
that candidate. Route it through a task branch and reviewed PR to `develop`,
then through a new reviewed `develop → main` PR. Generate and validate a new
candidate from the resulting exact `main` SHA. Never modify candidate artifact
bytes in place or reuse their checksum after mutation.

## Candidate, tag, and publication invariants

1. Generate candidates only by manually dispatching the release workflow from
   trusted `main`; the workflow validates and checks out the exact source SHA.
2. Complete the required automated and manual validation before tagging. A
   failed candidate is corrected through the normal task-branch workflow and
   replaced by a new candidate cycle.
3. The version tag points to the exact accepted candidate source SHA. Tags are
   immutable; later `main` commits never move a historical version tag.
4. Publish mode reuses the accepted candidate artifacts, verifies their
   checksums, and does not rebuild platform binaries. Published artifact hashes
   must match the validated candidate hashes.
5. A release is created only through the explicit publish workflow after
   approval. Merging to `main`, pushing a tag, or completing CI must not
   publish or distribute a release.

The full release identity is:

```text
accepted Release PR state → main candidate SHA → version tag SHA
validated candidate artifact hashes → published artifact hashes
```

For example, `v0.1.1` remains bound to
`b2784bf97b115e7e8ad10dff4f4ef5c29c3171fb` even as `main` advances.

## Post-release work and reconciliation

After publication, the release freeze ends. Normal work and closeout evidence
continue through task branches and PRs to `develop`. When appropriate, bring
accepted post-release work into `main` with a reviewed `develop → main`
reconciliation PR. This advances the repository's release line without
changing any prior tag, release, or artifact. `main` and `develop` need not
remain equal between promotions.

The v0.1.1 freeze ends with the post-release reconciliation merge following
this policy. Parent R001 remains open for tester-driven validation; this
housekeeping does not close it.

## GitHub protections and required validation

Protect `main` and `develop` with active branch rulesets that require pull
requests and the current CI checks, block force-pushes and deletion, and have
no bypass actors. The required CI check names are `Shared quality`, `macOS
native validation`, and `Windows native validation`. The repository's
dedicated read-only review remains a separate manual merge gate; no GitHub
approval count is required because the repository has one maintainer.

Release and reconciliation PRs must pass applicable formatting, lint,
typecheck, frontend tests/build, Rust format/Clippy/tests, dependency audits,
RustSec, documentation and workflow validation, and macOS/Windows native
validation. Preserve the final macOS bundle and mounted-DMG strict codesign
gates, Windows NSIS validation, source pinning, and candidate/publish
separation.

For macOS, strict `codesign --verify --deep --strict` is always a hard gate.
The known `Internal Xprotect Error` is warning-only solely on the two tested
hosted runner image/version pairs recorded in the R001.1 incident report, where
it also affected a valid minimal control app. Unknown environments, unknown
security findings, and unstructured output fail closed pending review; do not
broaden the exception.

The release workflow is `workflow_dispatch` only and must be run from `main`.
Ordinary PRs, pushes, merges, and tag creation do not publish. Release inputs
and notes are version-specific and must be deliberately updated through the
normal reviewed promotion path for a future version.

## Release checklist

- [ ] Freeze release scope and confirm `develop` CI is green.
- [ ] Open the reviewed `develop → main` Release PR; inspect the full diff.
- [ ] Confirm no unrelated or unfinished feature work is included.
- [ ] Merge the Release PR; record the exact accepted `main` SHA.
- [ ] Generate a candidate from that exact `main` SHA.
- [ ] Validate package identity, platform gates, checksums, and required manual tests.
- [ ] Confirm no release-affecting source change has occurred since candidate generation.
- [ ] After explicit approval, create the immutable tag at the candidate SHA.
- [ ] Publish by reusing the accepted candidate artifacts without rebuilding.
- [ ] Verify published hashes, release classification, notes, and assets.
- [ ] Complete a read-only post-publication review and end the Release Freeze.

If a candidate fails or a release-critical defect is found, do not patch `main`
directly. Use `fix/*` → PR to `develop` → review and CI → Release PR to `main`
→ a new candidate cycle. Do not move a published tag or silently replace a
released binary; use a new version for changed artifacts.
