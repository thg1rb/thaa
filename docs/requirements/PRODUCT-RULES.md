# Product Rules

**Baseline:** THAA-REQ-0.1 · **Status:** Frozen for initial implementation · **Date:** 2026-10-02

These rules are product-level invariants derived from the [development prompt](../THAA-DEVELOPMENT-PROMPT.md). A material change requires a reviewed requirement change; implementation must not silently weaken them.

| ID | Rule | Consequence / evidence |
|---|---|---|
| PR-001 | Thaa is local-first and privacy-first by default. No account, telemetry, cloud sync, AI API, or hidden upload is introduced without an explicit requirement change. | Dependency/config review; process and filesystem information remains local. |
| PR-002 | macOS and Windows are first-class targets. | Shared behavior has native provider contracts/tests on both platforms; unsupported capabilities are disclosed. |
| PR-003 | OS-specific behavior stays in platform/infrastructure adapters. | No broad OS branching in shared domain/application logic. |
| PR-004 | Missing/denied process metadata is unavailable, not fabricated. | Fields have explicit absence/capability/error representation and clear UI copy. |
| PR-005 | Process control targets a revalidated process identity, never a name-only group. | Verify current PID and identity evidence where feasible; report permission/race failures. |
| PR-006 | Force stop is explicit and distinct from graceful stop. | No silent escalation; confirmation and clear result before/after force stop. |
| PR-007 | Process metadata and paths are untrusted and can be sensitive. | Do not execute them, interpolate them into shell commands, or log them by default. |
| PR-008 | URL opening is limited to validated local listener URLs. | Validate scheme, host, port, and IPv6 formatting; never treat command-line strings as URLs. |
| PR-009 | A wildcard or non-loopback bind means potentially reachable from other interfaces, not proven internet exposure. | Use factual language and expose unknown state when classification is unavailable. |
| PR-010 | Project/runtime/framework classifications require observable evidence. | Never infer a runtime solely from a port number or execute project code for detection. |
| PR-011 | Project root and Git branch context are P1, not initial P0 acceptance. | Include in the initial release only by a deliberate scope change after P0 parity/stability evidence. |
| PR-012 | Linux is not a v1 release target. | Shared code may avoid unnecessary barriers, but no Linux product/release commitment is implied. |
| PR-013 | Thaa remains a focused runtime utility. | No generic admin dashboard, full system monitor, IDE, terminal, Git client, or full container manager. |

## Requirement change process

A requirement change PR records affected IDs, user/problem rationale, new acceptance criteria, P0–P3/out-of-scope impact, platform consequences, architecture/security implications, tests/traceability, and work dependencies. The product owner reviews scope before implementation. Update this baseline deliberately; never mark a requirement implemented without verification evidence.
