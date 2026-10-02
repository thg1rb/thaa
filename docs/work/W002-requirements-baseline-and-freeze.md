# W002 — Requirements Baseline and Freeze

## Status

Reviewed; applicable validation passes. PR #3 merge pending.

## Objective

Turn the authoritative development prompt into a concise, identified, testable P0/P1/P2/P3 requirement baseline before application implementation.

## Scope

- Define product outcomes, target users, principles, supported platforms, MVP boundary, and explicit exclusions.
- Assign stable functional, non-functional, and product-rule identifiers with acceptance evidence and source references.
- Trace P0 requirements to planned work and test-case identifiers.
- Record the product-owner-confirmed P0/P1 boundary and requirement freeze/change process.

## Out of Scope

Architecture design/ADRs (W003), security/test strategy documents (W004), Tauri bootstrap, implementation, detailed UX layouts, and P1/P2/P3 feature designs.

## Requirements

Implements prompt sections 13–19, 34–36, 51–54, and 69–74. Baseline identifier: `THAA-REQ-0.1`.

## Dependencies

W001 is Done and integrated into `develop` by PRs #1 and #2.

## Design Notes

Distill rather than reproduce the prompt. Requirement identifiers remain stable; test IDs link intent to future executable coverage. Use `docs/requirements/` as the baseline home.

## Acceptance Criteria

- Product, functional, non-functional, and product-rule documents share one baseline version/status/date.
- P0 acceptance is explicit and traceable to W007–W014 and TC-001–TC-011.
- Project root and Git context remain P1; process working directory remains P0 where available.
- P2/P3 and explicit exclusions are recorded without adding scope.
- Unavailable metadata, platform differences, safe process actions, privacy, and no arbitrary project execution are requirements.
- Baseline change process is explicit; no material unresolved product question blocks the P0 freeze.
- Baseline is marked Frozen, not implemented.

## Validation

- `git diff --check`
- Local Markdown relative-link/path check
- Requirement source coverage and identifier uniqueness review
- P0 acceptance-to-work/test traceability review
- Read-only Sub-agent review of W002 PR

## Security Considerations

Retain least privilege, safe process identity, no shell injection, no telemetry, privacy-aware logging, safe URL handling, and no arbitrary project execution. W004 will define the full threat model and controls before process actions.

## Platform Considerations

Every P0 capability is framed for native macOS and Windows behavior. Represent capability differences explicitly rather than claiming false parity. Do not introduce Linux release requirements.

## Documentation Impact

Creates the four `docs/requirements/` baseline documents, updates the docs index, and records this work item. Architecture/security/testing remain in W003/W004.

## Review Findings

The read-only reviewer identified a Low-severity wording mismatch: the docs index called the requirements baseline “Approved” while this work item was still under review. The index heading is now “Initial requirements baseline”; focused re-review reported no remaining findings.

## Known Limitations

No requirements are implemented by this work item. Numeric performance budgets and provider feasibility details are assigned to measurement and architecture/provider work rather than invented here.
