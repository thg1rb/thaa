# W007 — PortProvider Contract

## Status

Ready for re-review — the initial read-only review's Low finding was fixed by
moving reusable contract assertions into `src-tauri/tests/common/`, and the
affected validation has been rerun. PR re-review and integration remain.

## Objective

Define the smallest shared contract future platform adapters use to return
normalized listening TCP endpoints, with explicit partial and failed query
semantics.

## Scope

- Add the provider port, normalized scan outcome, and stable provider error
  categories under the domain layer specified by W003.
- Add deterministic contract tests using a test-only provider double and
  document how W008/W009 extend validation on native platforms.
- Update architecture, directory, data-flow, testing, and this work-item
  documentation where the contract changes their meaning.

## Out of Scope

macOS/Windows provider implementations, `lsof`, Windows APIs, shell execution,
live listener discovery, process inspection/control, refresh coordinator
implementation, Tauri commands, and frontend changes.

## Requirements

Establishes the shared provider-contract foundation required to implement
FR-001 and FR-002, and supports the normalized address/binding data in FR-003.
It supports NFR-003/005/008/009 and PR-002/003/004/007/009. No listening-port
discovery or other functional requirement is implemented by W007; the frozen
baseline remains `THAA-REQ-0.1`.

## Dependencies

W006 — Shared Domain Models is Done and integrated. W003 architecture and
ADR-003/ADR-004, W004 security/testing baselines, and W006 models govern this
work.

## Design Notes

The contract belongs in `domain` because W003 explicitly assigns stable
provider ports there. `PortProvider::listeners(&self)` is synchronous,
object-safe, and `Send + Sync`: the future coordinator owns a shared provider
and dispatches blocking scans away from the UI thread. Cancellation and
non-overlap belong to orchestration; an in-flight call must settle before a
replacement scan begins.

Successful queries return `PortScanResult` with complete or partial status. A
complete empty result means no listeners were found. Partial results carry a
stable failure category; query failures use `PortProviderError`. Provider
ordering is unspecified. Preserve records that differ by address or
ownership; only fully identical normalized duplicate rows may be coalesced.
Errors expose stable categories, not platform codes or raw output.

Reusable controlled-listener discovery and absence assertions live in
`src-tauri/tests/common/port_provider_contract.rs`. They are available to
external Cargo integration-test targets such as future W008/W009 provider
tests. Discovery returns completeness alongside the match; absence requires a
complete scan so a partial result cannot falsely prove that a listener is gone.

## Acceptance Criteria

- [x] `PortProvider` is defined at the W003 consumer-facing provider boundary
      and returns W006 `NetworkListener` values.
- [x] Complete, partial, and query-failure outcomes are distinct; complete
      empty results do not conceal errors.
- [x] Stable categories cover permission denial, unsupported operation,
      unavailable mechanism, parse failure, OS failure, and provider failure.
- [x] Contract preserves IPv4/IPv6 addresses, unresolved ownership, and
      distinct same-port endpoints without imposing order.
- [x] Trait is substitutable through a trait object and uses no runtime or
      platform-specific dependency.
- [x] Deterministic tests cover successful, empty, partial, and error outcomes.
- [x] Contract assertions are available to external integration-test crates.
- [x] Architecture and testing docs accurately distinguish contract evidence
      from native discovery implementation.
- [ ] PR review and integration into `develop` are complete before status is
      changed to Done.

## Validation

After the review fix: PASS — `cargo fmt --check`,
`cargo clippy --all-targets -- -D warnings`, `cargo test` (8 unit and 4
integration tests), frontend Prettier/lint/typecheck/Vitest (3 tests)/production
build, host macOS arm64 `pnpm tauri build --no-bundle`, `pnpm audit`, Markdown
formatting/link checks (19 architecture/testing/security/work documents), and
`git diff --check`. `cargo audit` is NOT RUN because the subcommand is not
installed. Windows execution is NOT RUN because no Windows runner is available.

## Security Considerations

The contract returns normalized values only. Provider implementations must
validate/normalize system-derived data, must not send raw output or native
error details across the boundary, and must not execute returned values. A
provider failure cannot be represented as a successful empty scan. No shell,
Tauri IPC, URL, or destructive behavior is added.

## Platform Considerations

The shared contract contains no macOS/Windows types or conditional behavior.
Native providers must implement the same semantics and run the shared behavior
expectations on their own OS. W007's host checks do not establish either
provider's platform support.

## Documentation Impact

Update `docs/architecture/PLATFORM-ADAPTERS.md`,
`docs/architecture/DATA-FLOW.md`,
`docs/architecture/DIRECTORY-STRUCTURE.md`, and
`docs/testing/TEST-CASES.md`. Do not change W006 domain semantics, ADRs, or the
frozen requirements.

## Review Findings

The initial dedicated read-only review reported one Low finding: a helper
compiled inside the library's `cfg(test)` module was inaccessible to external
Cargo integration-test crates, despite being documented for W008/W009 reuse.
The main Agent moved the assertions and contract tests to `src-tauri/tests/`;
the tests now compile against the public library API. Re-review is pending.

## Known Limitations

Test-only provider doubles establish shared result/error semantics and
trait-object substitution, not native discovery. W007 supplies no
operating-system evidence. Provider deadlines and exact blocking execution
mechanisms are deferred to platform implementation; refresh coordination must
still prevent overlap and keep blocking work off the UI thread.
