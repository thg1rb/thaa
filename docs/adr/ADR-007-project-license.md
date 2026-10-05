# ADR-007 — Project License

## Status

Accepted — 2026-10-05

## Context

The project had no root license file or project SPDX metadata. A clear license
is required for public open-source use and before the planned `v0.1.0` release.
Third-party source, dependencies, and copied Project Skills have separate
license terms and notices.

## Decision

License Thaa project-owned code and documentation under the MIT License. Use
the canonical MIT text in the root `LICENSE`, with copyright year 2026 and the
public repository owner handle `thg1rb`. Declare SPDX identifier `MIT` in
Cargo and npm package metadata and link to the license from the README.

This decision applies to Thaa project-owned material. It does not change the
license of third-party dependencies, assets, or Project Skills; retain their
own required notices and license files.

## Alternatives Considered

- No license: rejected because users and contributors would not have clear
  permission to use, modify, or redistribute the project.
- A copyleft or more restrictive license: not selected; the owner selected a
  concise permissive license for this developer utility.

## Rationale

MIT is a widely recognized permissive license suitable for a small open-source
developer utility. It permits use, modification, redistribution, and
commercial use while requiring preservation of the copyright and permission
notice. The repository owner handle is used as the public copyright identity;
no personal or legal name is inferred.

## Consequences

- The root README, Cargo manifest, and package metadata must state `MIT`
  consistently.
- New project-owned files follow the project license unless a file-specific
  notice says otherwise.
- Third-party notices remain separate and are not relicensed by this ADR.
- The final release review verifies license text, metadata, and notices.
