# Contributing to Thaa

Thaa is under active development. Start with the [authoritative development prompt](docs/THAA-DEVELOPMENT-PROMPT.md), [frozen requirements](docs/requirements/), [architecture](docs/architecture/), and [agent workflow](docs/agent/AGENT-WORKFLOW.md). The application shell exists; product inspection features are not implemented yet.

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

Follow [local setup](docs/development/LOCAL-SETUP.md) and [development commands](docs/development/DEVELOPMENT.md).
