# Agent Workflow

## Before a task

Read the authoritative development prompt and active work item. Confirm its requirement links, dependencies, scope, acceptance criteria, validation, security, platform, and documentation impacts. Resolve repository facts through inspection; record product assumptions rather than silently expanding scope.

Use only the relevant entries in [Project Skills](SKILLS.md): Rust work may
use `rust-patterns`; React work may use the React guidance and accessibility
skill; UI design may use `frontend-design`; security-sensitive work should add
`security-and-hardening` to the applicable engineering guidance. Skills are
advisory and do not override project requirements, accepted ADRs, W003/W004,
or the scope of the active work item. For Tauri specifics, use official Tauri
documentation; no Tauri skill is approved currently.

## During a task

Work on a task branch from `develop`. Implement the smallest coherent slice, add/update tests, run applicable quality gates, update affected documentation, and inspect the final diff. Do not claim unrun checks passed. Do not add project skills globally.

Follow [Coding Standards](../development/CODING-STANDARDS.md). Use Prettier
for the maintained frontend and documentation paths selected by the package
scripts, and `rustfmt` for Rust. Inspect the formatter's diff and do not run it
over frozen requirement documents.

## Pull request and review

Open a PR to `develop`. A dedicated review-only Sub-agent checks the diff, code, tests, documentation, architecture, security, platform effects, regressions, and acceptance criteria. The reviewer reports findings only and must not edit, patch, commit, push, merge, rebase, or rewrite history. The main Agent evaluates findings, applies valid changes, reruns checks, and requests re-review when changes are material. Merge only after blocking findings are resolved.

## Definition of Done

A work item is complete only when applicable acceptance criteria, tests, platform validation, formatting/lint/type/static checks, security review, documentation updates, read-only review findings, and integration into `develop` are complete. A successful local build alone is insufficient.
