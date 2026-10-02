# W003 — Architecture Baseline and ADRs

## Status

In Progress — implementation is documentation-only; review and integration are pending.

## Objective

Establish the architecture that turns frozen baseline `THAA-REQ-0.1` into maintainable, testable implementation work for macOS and Windows.

## Scope

- Define layer responsibilities, dependency direction, domain concepts, provider contracts, platform composition, and frontend/Tauri boundaries.
- Document safe process action, metadata availability, refresh coordination, error, and network binding classification principles.
- Record material initial decisions in accepted ADRs and trace architecture to frozen requirements.

## Out of Scope

Application scaffold or source code, native providers, process termination, React components, detailed visual design, W004 security/testing baselines, and P1/P2/P3 implementation.

## Requirements

Supports `FR-001`–`FR-011`, `NFR-001`–`NFR-010`, and `PR-001`–`PR-013` as referenced by the architecture. It does not implement or change requirements. Project-root and Git context remain P1; working directory remains P0 where available.

## Dependencies

W001 and W002 are Done and integrated into `develop`. The baseline `THAA-REQ-0.1` is frozen.

## Design Notes

Use a Clean/Hexagonal-inspired structure without speculative abstractions. Keep shared domain/application code independent of Tauri, frontend frameworks, shell output, and platform APIs. Select concrete adapters in the application composition root. Rust transport DTOs are authoritative; TypeScript mirrors them manually with contract checks until evidence justifies generation.

## Acceptance Criteria

- [ ] Architecture layers, inward dependency direction, composition, and proposed source layout are clear.
- [ ] Domain concepts represent listeners, process identity and metadata, metadata availability, and binding scope without platform types.
- [ ] Port discovery, process inspection, and process control have distinct substitutable contracts.
- [ ] macOS and Windows implementations remain isolated and can run equivalent behavioral contracts.
- [ ] Tauri commands are thin; frontend code does not inspect the system directly.
- [ ] Process actions can revalidate observed identity immediately before acting; graceful and force stop remain separate.
- [ ] Capability differences, error mapping, privacy constraints, and missing metadata are explicit.
- [ ] One backend coordinator owns refresh state, prevents overlapping scans, and rejects stale snapshots.
- [ ] Architecture supports P0 traceability without promoting P1 project-root/Git features.
- [ ] Accepted ADRs record material decisions and alternatives; no application scaffold or provider implementation is added.

## Validation

- `git diff --check`
- Check local Markdown links, documentation navigation, ADR numbering/references, and requirement identifiers.
- Confirm frozen requirement files are unchanged and no requirement is marked implemented.
- Review terminology, dependency direction, security constraints, platform isolation, and architecture against the authoritative prompt and `THAA-REQ-0.1`.
- Application builds and runtime tests are not applicable because this work item adds documentation only.
- Complete a read-only Sub-agent review of the PR; resolve valid findings and re-review material fixes.

## Security Considerations

Treat process names, arguments, and paths as untrusted and potentially sensitive. Never use shell interpolation or execute observed metadata. Pass structured validated inputs across boundaries, avoid logging command arguments, revalidate process identity before actions, and never silently elevate privileges.

## Platform Considerations

macOS and Windows implement shared contracts with native adapters. Differences are expressed as capabilities and unavailable metadata, not shared-layer OS branches or fabricated values. Compile-time platform selection is confined to the composition/infrastructure boundary.

## Documentation Impact

Adds the architecture overview, domain model, platform adapter contracts, data-flow diagrams, ADR index and records, and this work item. Updates the documentation index. W004 owns the detailed threat model and testing strategy.

## Review Findings

Pending PR review.

## Known Limitations

The architecture does not select concrete native API crates, provider method signatures, refresh cadence, frontend state library, logging backend, or generated type bindings. These remain implementation decisions for the relevant work items and must not weaken the documented contracts.
