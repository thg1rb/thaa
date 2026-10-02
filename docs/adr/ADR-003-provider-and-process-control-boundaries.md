# ADR-003: Separate Providers and Process Control

## Status

Accepted.

## Context

P0 requires listener discovery, process inspection, and explicit process actions. These operations have different effects, permissions, failure modes, and safety requirements.

## Decision

Define distinct `PortProvider`, `ProcessProvider`, and `ProcessController` contracts. Providers return normalized values and semantic availability/error outcomes. Process actions carry the previously observed process identity, revalidate available identity evidence immediately before acting, and preserve graceful and force stop as separate explicit actions.

## Alternatives Considered

- One system-manager service for discovery, metadata, and termination: rejected because read-only and destructive responsibilities would be coupled.
- Process-name targeting or PID-only assumptions: rejected because process names are not unique and PIDs may be reused.
- A universal native implementation: rejected because the operating systems use different APIs and permission models.

## Consequences

Fakes can test application orchestration independently. Native contract tests must cover discovery, metadata availability, and action outcomes on each supported OS. Identity evidence may differ by platform; mismatches or inadequate evidence must not silently target a process. Provider implementation choices remain deferred.
