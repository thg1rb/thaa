# ADR-006: Nearest-Marker Project Root Detection

## Status

Accepted; implemented in W015 and integrated by PR #35

## Context

FR-012 requires the nearest plausible project root from a process working
directory using documented marker precedence without scanning the whole
filesystem. Nested projects and workspaces can contain multiple plausible
roots. The existing Windows process provider does not expose arbitrary process
working directories.

## Decision

Traverse the canonicalized working directory and its parents from nearest to
farthest. The first directory containing any supported marker is the root;
directory proximity takes precedence over marker category. Supported markers
are `.git` (file or directory), `package.json`, `pnpm-workspace.yaml`,
`Cargo.toml`, `go.mod`, `pyproject.toml`, `pom.xml`, `build.gradle`,
`build.gradle.kts`, `settings.gradle`, `settings.gradle.kts`, `composer.json`,
and direct-child `*.sln` or `*.csproj`. Multiple markers in one directory
yield the same root; marker category is not used to select between paths, so
same-directory evidence is treated as an unordered set.
Marker contents are not read or executed. Only one project-root path is
exposed; workspace and repository roots are not separate W015 fields.

If the working directory is unavailable, cannot be canonicalized, or a
candidate directory cannot be safely inspected, detection returns no root.
Traversal stops at the platform filesystem/volume root or the first different
mounted filesystem and never recursively scans. Unix device identity and
Windows volume mount paths are checked by the platform adapter before each
ancestor inspection. The starting directory is canonicalized once; a marker
symlink counts only when metadata resolves to an expected file/directory type.
Windows detection uses only existing working-directory metadata and therefore
remains unavailable with the current Windows provider limitation.

## Alternatives Considered

- Prefer the outermost Git/workspace root: rejected because it hides nearer
  project context in nested repositories and monorepos.
- Treat lockfiles or generic files such as `README.md`/`Makefile` as roots:
  rejected because they create ambiguous false positives.
- Acquire Windows cwd through undocumented process structures, shell tools,
  or elevation: rejected for safety and platform-boundary reasons.
- Expose project, workspace, and repository roots separately: deferred until
  a requirement calls for those distinct concepts.

## Consequences

The result is deterministic, bounded by directory depth, and testable with
filesystem fixtures. A nearer package root wins over a distant `.git` or
workspace root. Root availability depends on the existing OS working-directory
provider; Windows currently returns no root. Detection is contextual metadata
and must never participate in process identity or authorization.
