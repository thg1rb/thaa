# ADR-005: Local-First Operation and No Telemetry

## Status

Accepted — required by `THAA-DEVELOPMENT-PROMPT.md` §4 and frozen as NFR-001 / PR-001.

## Context

Process identifiers, command lines, filesystem paths, and local project context can be sensitive. Normal inspection does not require a cloud service.

## Decision

Normal operation remains local. Do not add accounts, telemetry, cloud sync, remote APIs, or external AI services without an explicit reviewed requirement change. Treat observed process/path data as sensitive and avoid logging command arguments by default.

## Alternatives Considered

- Default product analytics or remote diagnostics: rejected because they conflict with the privacy-first requirement and are unnecessary for local inspection.
- Cloud-backed inspection: rejected because the operating-system data is local and no remote feature is required.
- Opt-in telemetry in the initial architecture: deferred; it would still require a separate user need, consent design, data minimization, and requirement/security review.

## Consequences

Dependencies and build configuration must not create hidden reporting paths. Diagnostics and bug reports need privacy-aware review. This ADR does not replace W004's threat model or dependency-supply-chain controls.
