# Product Rules

**Baseline:** THAA-REQ-0.1 · **Status:** Frozen for initial implementation · **Date:** 2026-10-02

These rules are product-level invariants derived from the [development prompt](../THAA-DEVELOPMENT-PROMPT.md). A material change requires a reviewed requirement change; implementation must not silently weaken them.

| ID | Rule | Prompt source | Consequence / evidence |
|---|---|---|---|
| PR-001 | Thaa is local-first and privacy-first by default. No account, telemetry, cloud sync, AI API, or hidden upload is introduced without an explicit requirement change. | §4, §19, §36, §44 | Dependency/config review; process and filesystem information remains local. |
| PR-002 | macOS and Windows are first-class targets. | §5, §8, §39, §53 | Shared behavior has native provider contracts/tests on both platforms; unsupported capabilities are disclosed. |
| PR-003 | OS-specific behavior stays in platform/infrastructure adapters. | §7–8 | No broad OS branching in shared domain/application logic. |
| PR-004 | Missing/denied process metadata is unavailable, not fabricated. | §10–11, §43 | Fields have explicit absence/capability/error representation and clear UI copy. |
| PR-005 | Process control requires one identity-bound target and fresh validation of platform-required evidence; missing or changed evidence refuses the action. Never target by name, group, wildcard, or port. | §4.5, §38, §76 | Require positive PID and required start-time evidence; compare executable identity where part of the platform policy; document residual races and report permission/disappearance outcomes. |
| PR-006 | Graceful Stop and Force Stop are distinct, capability-gated actions. Graceful Stop requests normal shutdown; Force Stop requests forced termination and is never substituted automatically. | §4.5, §11, §13, §38 | Expose only actions the platform reports as generally supported; require explicit force confirmation; no automatic escalation or elevation; distinguish request acceptance from observed exit. |
| PR-007 | Process metadata and paths are untrusted and can be sensitive. | §37, §44, §67 | Do not execute them, interpolate them into shell commands, or log them by default. |
| PR-008 | URL opening is limited to validated local listener URLs. | §75 | Validate scheme, host, port, and IPv6 formatting; never treat command-line strings as URLs. |
| PR-009 | A wildcard or non-loopback bind means potentially reachable from other interfaces, not proven internet exposure. | §16 | Use factual language and expose unknown state when classification is unavailable. |
| PR-010 | Project/runtime/framework classifications require observable evidence. | §15, §74 | Never infer a runtime solely from a port number or execute project code for detection. |
| PR-011 | Project root and Git branch context are P1, not initial P0 acceptance. | §14, §52 | Include in the initial release only by a deliberate scope change after P0 parity/stability evidence. |
| PR-012 | Linux is not a v1 release target. | §5, §19 | Shared code may avoid unnecessary barriers, but no Linux product/release commitment is implied. |
| PR-013 | Thaa remains a focused runtime utility. | §2, §19 | No generic admin dashboard, full system monitor, IDE, terminal, Git client, or full container manager. |

## Requirement change process

A requirement change PR records affected IDs, user/problem rationale, new acceptance criteria, P0–P3/out-of-scope impact, platform consequences, architecture/security implications, tests/traceability, and work dependencies. The product owner reviews scope before implementation. Update this baseline deliberately; never mark a requirement implemented without verification evidence.

## Approved clarification record

W012.0.1 clarifies FR-008/FR-009 and PR-005/PR-006 without changing the
`THAA-REQ-0.1` identifier or its stable requirement IDs. The former product
summary described “graceful stop with explicit force-stop fallback,” and
FR-009 called force stop a “fallback.” That wording did not express the
verified capability difference: macOS supports both requests, while Windows
does not have a safe generic graceful mechanism for arbitrary discovered
processes. The revised requirements make each action capability-gated and
prohibit substituting Force Stop when Graceful Stop is unsupported.

The user-approved change is based on W012.0 evidence. macOS uses identity
revalidation followed by SIGTERM or SIGKILL, with the documented residual
check-to-signal PID race. Windows generic graceful stop is unsupported; future
force stop must revalidate and call `TerminateProcess` through the same
process HANDLE. Both require identity-bound targets, fail closed on missing or
changed required evidence, avoid automatic elevation/escalation, and treat
accepted requests separately from confirmed exit. Test and work traceability
is updated in W012.0.1; neither native controller is implemented by this
change. The pre-clarification wording remains recoverable in Git history and
is quoted in the W012.0.1 work record.
