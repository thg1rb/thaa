# W006 — Shared Domain Models

## Status

Done — PR #13 reviewed with no findings and merged into `develop` as merge
commit `a6779779265d016e0bcec9659996c26dcd5f52c7`.

## Objective

Implement the minimum platform-neutral Rust domain values needed by later P0
listener and process providers, without inspecting the operating system or
implementing a user-facing capability.

## Scope

- Add shared models for the current TCP listener, process identity and metadata,
  per-field metadata availability, platform-wide capabilities, and normalized
  listener binding scope.
- Keep the domain independent of Tauri, frontend types, OS APIs, provider
  output, and shell behavior.
- Add deterministic unit tests for model behavior and pure IPv4/IPv6 binding
  classification.
- Align domain and testing documentation with the implementation.

## Out of Scope

Provider traits or implementations, platform adapters, live port/process
inspection, process actions, Tauri transport DTOs, frontend features, project
root/Git detection, and runtime detection.

## Requirements

Provides domain foundations for FR-002 and FR-003 and process metadata
foundations for FR-006. Identity evidence, capability, and availability models
support future FR-008, FR-009, and FR-011 work. No functional requirement is
implemented by W006; `THAA-REQ-0.1` remains frozen and unchanged.

## Dependencies

W001–W005.1 are Done and integrated into `develop`. W003 architecture and W004
security/testing baselines govern this work.

## Design Notes

Use standard-library `IpAddr`, `NonZeroU16`, `OsString`, `PathBuf`, and
`SystemTime` for normalized values. Keep absent listener ownership distinct
from query failure. Represent metadata availability per field, while keeping
process disappearance and whole-provider failures outside the domain field
state. Store command arguments as OS strings in argument order, never as a
shell-formatted command. A non-loopback binding means potentially reachable,
not Internet exposed. Structural process identity equality is not action
authorization.

## Acceptance Criteria

- [x] Shared Rust domain types support future P0 listener and process
      inspection.
- [x] Metadata availability distinguishes observed values and supported
      unavailable reasons without fabricating values.
- [x] Process paths and arguments retain OS-native representations.
- [x] IPv4/IPv6 binding classification is pure and conservative.
- [x] Domain has no Tauri, frontend, platform API, provider, or shell dependency.
- [x] Meaningful deterministic unit tests pass.
- [x] Architecture/testing docs accurately describe foundations without
      claiming product functionality is implemented.
- [x] PR review and integration into `develop` are complete.

## Validation

PASS: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
`cargo test` (8 tests), frontend Prettier check, ESLint, TypeScript typecheck,
Vitest (3 tests), frontend production build, macOS arm64 Tauri host build
(`pnpm tauri build --no-bundle`), `pnpm audit`, documentation link check (40
Markdown files), and `git diff --check`. Frozen requirement, ADR, and
authoritative prompt files are unchanged. `cargo audit` is NOT RUN because the
subcommand is not installed. Windows validation is NOT RUN on this macOS host.
Post-merge Rust format, Clippy, and tests passed; post-merge macOS arm64
Tauri host build passed. No application feature/provider, live OS inspection,
or transport changes were introduced.

## Security Considerations

Treat names and arguments as untrusted data. Preserve argument structure and
native paths; do not execute or format them as shell commands. Keep missing
ownership separate from provider failure. Identity value equality alone must
not authorize process control. Binding classification must not imply external
reachability.

## Platform Considerations

Shared types use Rust standard-library network and OS-string/path values so
macOS and Windows adapters can map supported data without platform-specific
types in the domain. Tests use deterministic fixtures only. Windows execution
evidence is unavailable on the current macOS host and must be reported as not
run.

## Documentation Impact

Update `docs/architecture/DOMAIN-MODEL.md`, the actual directory structure,
the relevant planned test-case notes, and this work record. Do not modify the
frozen requirements or add P1 project/Git fields to P0 models.

## Review Findings

PR #13 received a dedicated read-only review: no findings; no fixes or
re-review required.

## Known Limitations

No provider contracts or live OS evidence are included. The Tauri transport
representation for OS-native paths/arguments remains a later boundary decision.
Current domain models provide structural values only; safe process identity
revalidation belongs to application/provider work.

## Architecture Validation

- Domain independence: PASS — the domain imports only Rust standard-library
  types and its own domain modules.
- Cross-platform semantics: PASS by source inspection — shared Rust types use
  `IpAddr`, `OsString`, `PathBuf`, and `SystemTime`; Windows native execution is
  NOT RUN.
- Testability: PASS — pure classification and model semantics have unit tests.
- Missing metadata: PASS — field values and unavailable reasons are distinct;
  whole-query failures remain outside the field model.
- Path correctness: PASS — native `PathBuf` is retained without UTF-8
  conversion.
- Network classification: PASS — pure and conservative; does not infer
  Internet reachability.

## Platform Validation

macOS arm64 Rust checks and Tauri host build: PASS. Windows native execution:
NOT RUN because no Windows runner is available in this work session.
