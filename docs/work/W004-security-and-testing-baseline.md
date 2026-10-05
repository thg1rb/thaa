# W004 — Security and Testing Baseline

## Status

Done — W004 was reviewed and integrated into `develop` by PR #7. This post-merge status update follows the repository's established work-item closeout convention.

## Objective

Define the initial security assurance model, threat register, testing layers, platform contracts, and evidence rules for future Thaa implementation.

## Scope

- Establish a public vulnerability-reporting route and internal engineering security baseline.
- Model assets, trust boundaries, realistic threats, mitigations, residual risk, review triggers, and verification links.
- Define unit, provider contract, integration, process-action, Tauri, frontend, concurrency, security, platform, regression, flake, and performance testing expectations.
- Preserve traceability to frozen requirements and W003 architecture; document future quality-gate categories without choosing tools or implementing CI.

## Out of Scope

Application/provider/process-action/UI code, application dependencies, CI workflows, security/audit tooling installation, exact testing framework selection, numeric performance budgets, signing/release tooling, and P1/P2/P3 feature implementation.

## Requirements

Supports `FR-001`–`FR-011`, `NFR-001`–`NFR-010`, and `PR-001`–`PR-013`, with direct focus on NFR-001–NFR-006/NFR-008–NFR-010 and PR-001/002/004–PR-009. It defines future verification; it does not implement or change any requirement.

## Dependencies

W001, W002, and W003 are Done and integrated into `develop`; `THAA-REQ-0.1` is frozen. W003 architecture and ADRs govern the baseline.

## Design Notes

Threat entries use `THR-###`. Preserve requirement case IDs `TC-001`–`TC-011`; add grouped contract/security IDs only for meaningful verification. Test tool choices remain deferred. Root `SECURITY.md` is public disclosure policy; `docs/security/SECURITY.md` is engineering guidance. GitHub Private Vulnerability Reporting was enabled and verified before publishing the public policy. The installed Project Skill inventory contains only `find-skills`; no security/testing skill or application dependency is warranted for this documentation-only baseline.

## Acceptance Criteria

- [x] Engineering security baseline and threat model cover required assets, trust boundaries, threat areas, mitigations, residual risks, severity/blocking rules, and review triggers.
- [x] Material threats link to verification cases and relevant requirements/architecture; no mitigation is claimed implemented.
- [x] Testing strategy defines unit, provider-contract, integration, process-action, Tauri, frontend, race/concurrency, reliability/performance, regression, flake, and evidence rules.
- [x] Native macOS and Windows expectations and capability-parity statuses are explicit; no false parity or test claims.
- [x] Test isolation, test-data privacy, and controlled-process cleanup prevent harm to unrelated user/system processes.
- [x] Future quality gate categories and CI relationship are defined while tool/CI implementation remains deferred.
- [x] Requirements and W003 architecture remain unchanged and consistent; no product implementation, dependencies, or CI are introduced.

## Validation

- `git diff --check` — pass.
- Markdown links/anchors and documentation navigation — pass.
- Threat/test ID uniqueness and FR/NFR/PR references — pass.
- W003/ADR consistency and frozen requirement comparison — pass.
- Public hygiene and scope check (no code, dependencies, or CI) — pass.
- Dedicated read-only Sub-agent PR review of the initial PR head — pass; no findings.
- Final-head read-only re-review — pass; no findings.
- Application builds/tests — not applicable; no executable application code was introduced.

## Security Considerations

Security-sensitive reporting route uses GitHub Private Vulnerability Reporting. The threat model preserves least privilege, no hidden telemetry, untrusted metadata, safe native boundaries, process identity revalidation, privacy-aware diagnostics, and explicit destructive actions.

## Platform Considerations

macOS and Windows require separate real native integration validation for provider/action behavior. Shared contract expectations are consistent; differences are explicit as Required, Best Effort, Unsupported, Capability-based, Platform Difference, or Not Yet Implemented.

## Documentation Impact

Adds root vulnerability-reporting policy, engineering security baseline, threat model, test strategy, initial test catalog, platform-testing guidance, and this work item; updates `docs/README.md`. Does not change requirements or W003 architecture.

## Review Findings

The dedicated read-only Sub-agent reviewed PR #7 and its final head; both reviews reported no findings.

## Known Limitations

No product code exists, so mitigations and test cases are planned rather than implemented or executed. Test frameworks, security/audit tools, CI, runner images, performance thresholds, and release signing remain future decisions.

## Integration

PR [#7](https://github.com/thg1rb/thaa/pull/7) was merged into `develop` on 2026-10-02 as merge commit `ae1a57f29c85f4bcd40a45533f719f43cd8e840c`. The security and testing documents are now part of the integration baseline; this work remains documentation-only.
