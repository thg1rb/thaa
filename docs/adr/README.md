# Architecture Decision Records

ADRs record material decisions that shape implementation boundaries. They supplement, and do not supersede, the frozen requirements or `THAA-DEVELOPMENT-PROMPT.md`.

| ADR                                                           | Decision                                                         | Status                           |
| ------------------------------------------------------------- | ---------------------------------------------------------------- | -------------------------------- |
| [ADR-001](ADR-001-desktop-stack.md)                           | Tauri 2, Rust, React, and TypeScript desktop stack               | Accepted (fixed by prompt)       |
| [ADR-002](ADR-002-layered-platform-adapter-architecture.md)   | Shared inward-facing core with isolated platform adapters        | Accepted                         |
| [ADR-003](ADR-003-provider-and-process-control-boundaries.md) | Separate port discovery, process inspection, and process control | Accepted                         |
| [ADR-004](ADR-004-backend-refresh-coordination.md)            | Single backend refresh coordinator                               | Accepted                         |
| [ADR-005](ADR-005-local-first-no-telemetry.md)                | Local-first operation and no telemetry by default                | Accepted (fixed by prompt)       |
| [ADR-006](ADR-006-project-root-detection-policy.md)           | Nearest-marker project-root detection policy                     | Accepted for W015 implementation |
| [ADR-007](ADR-007-project-license.md)                         | MIT license for Thaa-owned material                              | Accepted                         |
| [ADR-008](ADR-008-git-context-cli.md)                         | Read-only Git context through the installed Git CLI              | Accepted for W016 implementation |

Use the repository ADR format: status, context, decision, alternatives, consequences. Revisit a decision through a new or superseding ADR and a reviewed requirement change when product scope is affected.
