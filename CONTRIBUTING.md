# Contributing to Thaa

Thaa is in repository bootstrap. Read the [authoritative development prompt](docs/THAA-DEVELOPMENT-PROMPT.md) and [agent workflow](docs/agent/AGENT-WORKFLOW.md) before making changes. The current work item and requirements are recorded under `docs/work/` and `docs/requirements/` as those baselines are established.

## Branches and pull requests

- `main` contains stable, release-ready work.
- `develop` is the integration branch.
- Start each task branch from `develop`. Use `feature/*`, `fix/*`, `docs/*`, `refactor/*`, or `chore/*` according to the change.
- Open a pull request from the task branch to `develop`. Do not merge task branches directly.
- Every task PR requires a dedicated review-only Sub-agent. The reviewer reports findings and must not edit, commit, push, merge, rebase, or rewrite history. The main Agent applies valid fixes and reruns applicable checks.
- Promote `develop` to `main` through a release PR after release validation.

The initial repository seed commit on `main` is a one-time genesis exception because no commit or remote branch existed. It contains only the authoritative project prompt; all subsequent work follows the branch workflow above.

## Change expectations

- Keep changes small and trace them to requirement/work IDs where available.
- Add or update tests for behavior changes and bug fixes where practical.
- Run the checks specified by the active work item. Report checks that could not run and why.
- Update relevant requirements, architecture, security, testing, and work documentation in the same change.
- Do not add telemetry, cloud dependencies, Linux release scope, or arbitrary execution of detected projects without an approved requirement change.

The local setup and quality commands will be recorded in [development documentation](docs/README.md) when the application scaffold is added.
