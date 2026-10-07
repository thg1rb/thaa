# ADR-008: Read-only Git Context Through the Git CLI

## Status

Accepted for W016 implementation; final validation is pending.

## Context

FR-013 requires the runtime inspector to show a repository root and current
branch from process working-directory metadata. The result is presentation
context only. The application must not become a full Git client, execute
project code, or make process actions depend on repository data.

## Decision

Use the installed Git CLI behind a shared `GitContextProvider` contract. Run
fixed built-in read-only commands (`rev-parse --show-toplevel` and
`branch --show-current`) through direct process execution with structured
arguments. Do not invoke a shell or add a Git library dependency. Clear
inherited `GIT_*` environment variables so the process working directory is
the only repository-discovery input. Treat missing Git, non-worktrees,
unavailable directories, and query failures as absent optional context.
Represent detached HEAD explicitly.

The application scan deduplicates lookups by exact working-directory path for
one snapshot. No persistent cache is introduced.

## Alternatives

- **Git library:** Not selected because the limited read-only requirement is
  fully served by Git's installed CLI and an additional parser/dependency
  would increase maintenance and supply-chain scope.
- **Direct .git parsing:** Rejected because .git can be a file in a
  worktree and Git owns nested repository and branch semantics.
- **Shell command construction:** Rejected because process paths are untrusted
  metadata and must never be interpolated.
- **Global repository search or executable-path inference:** Rejected because
  FR-013 starts from the process working directory and does not authorize
  filesystem-wide discovery.

## Consequences

Git must be installed and available to the app process. Git lookups are local,
read-only, and best-effort; their failure does not affect listener rows.
Repository paths and branch names are potentially sensitive user-visible
metadata and are not logged or transmitted. The current Windows process
provider has no working-directory value, so Git context remains unavailable
there until a separately accepted process-provider capability supplies it.
Git metadata is excluded from process identity and action authorization.
