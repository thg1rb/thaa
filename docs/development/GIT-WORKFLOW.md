# Git Workflow

## Branches

- `main`: stable, release-ready state and tagged releases.
- `develop`: integration branch for reviewed work.
- Task branches start from `develop`: `feature/*`, `fix/*`, `docs/*`, `refactor/*`, or `chore/*`.

Normal work follows task branch → validation → PR to `develop` → read-only Sub-agent review → main Agent fixes and revalidation → merge. Do not merge task branches directly. Release promotion uses a PR from `develop` to `main` after macOS and Windows validation, security/dependency checks, current docs, and release notes.

The first repository commit was a one-time genesis exception: it preserved the supplied authoritative prompt on `main`, after which `develop` was created at the same commit. No product or documentation work is performed directly on `main`.

Every release must reach `main` through a reviewed `develop → main` Release PR. Candidates are generated only from the accepted `main` SHA; release-affecting changes invalidate a candidate and require a new promotion/candidate cycle. See the authoritative [Release Workflow](../releases/RELEASE-WORKFLOW.md) for the Release Freeze, tag/artifact identity, reconciliation, checklist, and protections.

## Review-only Sub-agent

The reviewer inspects the diff, tests, documentation, architecture, security, platform effects, regressions, and acceptance criteria. It must not edit, commit, push, merge, rebase, or rewrite history. The main Agent applies valid findings. Re-review after material fixes.

## Branch protection

The active `Protect main and develop` GitHub ruleset requires pull requests
and Shared Quality, macOS Native Validation, and Windows Native Validation
checks; it blocks force-pushes and deletion and has no bypass actors. The
read-only Sub-agent review remains a separate manual merge gate.
