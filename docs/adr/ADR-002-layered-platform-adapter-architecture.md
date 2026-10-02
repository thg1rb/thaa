# ADR-002: Layered Core with Platform Adapters

## Status

Accepted.

## Context

macOS and Windows must provide equivalent product behavior through different operating-system APIs and permissions. Scattered platform conditionals would undermine testability and parity.

## Decision

Use domain, application, infrastructure/platform-adapter, thin Tauri boundary, and frontend presentation responsibilities. Shared application/domain logic depends on stable domain contracts. macOS and Windows implementations live in separate platform modules selected at the composition root. Use the pattern only for current needs; do not build speculative extension frameworks.

## Alternatives Considered

- OS branching throughout shared use cases: rejected because it couples business rules to platform APIs and makes contract parity harder to test.
- Separate application/domain implementations per OS: rejected because it duplicates shared behavior.
- A large plugin or dependency-injection framework: rejected because ordinary Rust composition is sufficient for the known providers.

## Consequences

Provider behavior can be replaced by fakes in shared tests and tested against equivalent contracts on native macOS and Windows runners. Adapter boundaries normalize native results and errors. Conditional compilation stays at platform composition/infrastructure boundaries. Exact API wrappers remain deferred to provider work.
