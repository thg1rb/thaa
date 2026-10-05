# ADR-001: Desktop Application Stack

## Status

Accepted — required by `docs/THAA-DEVELOPMENT-PROMPT.md` §6.

## Context

Thaa needs a macOS and Windows desktop shell, a system-facing core suitable for native integrations, and a focused webview presentation. The authoritative prompt fixes Tauri 2, Rust, React, and TypeScript as the technology direction.

## Decision

Use Tauri 2 for the desktop shell and command boundary, Rust for application/domain/system integration, and React with TypeScript for frontend presentation. Linux is not a v1 product target.

## Alternatives Considered

- Native separate UI stacks per operating system: rejected because they duplicate presentation and product behavior.
- A browser-only app or a shell-language system utility: rejected because the product needs a local desktop boundary and testable native system integration.
- Reopening the specified stack in W003: rejected because the prompt is authoritative and the decision is already made.

## Consequences

The backend/frontend boundary must use explicit serialized DTOs. Rust owns privileged system inspection; frontend code cannot directly inspect processes or ports. Native packaging and signing remain later release work. This ADR records existing direction; it does not authorize scaffolding in W003.
