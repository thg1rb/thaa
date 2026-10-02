# W001 — Repository and Agent Bootstrap

## Status

Review complete; applicable validation passes. PR #1 merge pending.

## Objective

Make the new repository understandable and establish a reproducible, project-scoped agent workflow before requirements and application implementation.

## Scope

- Preserve and link the authoritative development prompt.
- Add concise repository/contributor guidance, docs index, root agent instructions, agent workflow, review-only guidelines, skill record, and Git workflow.
- Install only the required trusted `find-skills` capability in project scope.

## Out of Scope

Requirements freeze, architecture decisions, application code, dependency/toolchain setup, license selection, remote publication, and production signing.

## Requirements

Supports prompt sections 25–31, 50, 64, and 67–69. Product requirement IDs will be assigned in W002.

## Dependencies

W000 one-time Git seed and `develop` branch foundation.

## Design Notes

Keep the root concise. Keep process, privacy, and review rules in maintained docs rather than copying the full prompt. Skills are Project Skills and must be source-reviewed before installation.

## Acceptance Criteria

- The authoritative prompt remains intact and reachable from README/docs index.
- Contributor and agent docs specify task branches, PRs to `develop`, read-only Sub-agent reviews, main-Agent fixes, validation, and Definition of Done.
- `find-skills` is installed under `.agents/skills/` with a lock entry; no other project skill is installed in W001.
- No empty directories, credentials, or local environment details are committed.

## Validation

- `git diff --check`
- Project-scope skill inventory and source/lock review
- Verify local Markdown links and Git branch/worktree state

## Security Considerations

Review skill source and permissions. Do not publish local paths, secrets, private endpoint names, or process data.

## Platform Considerations

Document macOS and Windows as targets; no platform implementation is introduced.

## Documentation Impact

Creates the initial navigable documentation and agent/contributor process. Requirements, architecture, security, and testing baselines remain in later work items.

## Review Findings

The initial local review identified an instruction conflict: upstream `find-skills` included a global-install recommendation. Repository instructions now explicitly override global-scope installation and skipped source review; local re-review reported no remaining findings. The final PR review identified a Low-severity stale remote-state note in W000. The main Agent corrected it, and the reviewer re-reviewed the update with no remaining findings.

## Known Limitations

There is no CI workflow yet; CI baseline is W006. GitHub branch protection has not been configured and is not represented as active. PR #1 has completed the documented human/agent review and applicable W001 checks; application builds/tests are not applicable before scaffold work.
