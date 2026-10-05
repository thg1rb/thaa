# Contributing to Thaa

Thaa is an open-source local runtime inspector for macOS and Windows. The
runtime inspector, search, process/project context, and supported process
actions are implemented; release packaging is still being prepared. Start with
the [README](README.md) for product capabilities and the current release
status, then review the [development prompt](docs/THAA-DEVELOPMENT-PROMPT.md),
[requirements](docs/requirements/), [architecture](docs/architecture/), and
[agent workflow](docs/agent/AGENT-WORKFLOW.md).

## Branches and pull requests

- `main` contains stable, release-ready work; `develop` is the integration branch.
- Start each task branch from the latest `develop`. Use `feature/*`, `fix/*`, `docs/*`, `refactor/*`, or `chore/*` according to the change.
- Open a pull request from the task branch to `develop`; do not merge task branches directly.
- Every task PR requires a dedicated review-only Sub-agent. The reviewer reports findings and must not edit, commit, push, merge, rebase, or rewrite history. The main Agent applies valid fixes and reruns applicable checks.
- Promote `develop` to `main` through a release PR after release validation.

## Change expectations

- Keep changes small and trace them to requirement/work IDs where available.
- Add or update tests for behavior changes and bug fixes where practical.
- Run the checks specified by the active work item. Report checks that could not run and why.
- Update relevant requirements, architecture, security, testing, and work documentation in the same change.
- Do not add telemetry, cloud dependencies, Linux release scope, or arbitrary execution of detected projects without an approved requirement change.

Follow [local setup](docs/development/LOCAL-SETUP.md), [development
commands](docs/development/DEVELOPMENT.md), and [coding
standards](docs/development/CODING-STANDARDS.md). Frontend source belongs
under `src/`; Rust and Tauri source belong under `src-tauri/`. Agents should
consult relevant [Project Skills](docs/agent/SKILLS.md) while following the
repository's governing requirements and review rules.
