# Product Requirements

**Baseline:** THAA-REQ-0.1 · **Status:** Frozen for initial implementation · **Date:** 2026-10-02
**Authority:** [Development prompt](../THAA-DEVELOPMENT-PROMPT.md), with the P0/P1 boundary clarified by the product owner during planning.

## Product

Thaa is a local-first desktop utility for developers who need to understand local ports, the processes that own them, and the development runtimes around them. It should answer: **what is running, which port is it using, where did it come from, and what project/runtime does it belong to?**

The initial product targets macOS and Windows and uses Tauri 2, Rust, TypeScript, and React. Thaa is a focused runtime inspector, not a general system administration dashboard or a graphical wrapper around one command-line tool.

## Users and outcomes

Primary users are developers and students running local services or encountering port conflicts. Early releases should let a user reliably answer:

1. What TCP listener is present?
2. Which process owns it, when the OS allows ownership to be resolved?
3. What process metadata and working directory are available?
4. Is the listener bound only to loopback or to a broader local address, when this can be determined?
5. Can the user inspect, open, copy, or safely stop the process?

## Product principles

- **Local-first and privacy-first:** no account, cloud backend, telemetry, or remote service is required. Process and filesystem information stays local by default.
- **Cross-platform by architecture:** macOS and Windows use isolated adapters and shared behavioral contracts.
- **Progressive disclosure:** essential port/process information is easy to scan; sensitive or advanced command/path information is shown only when useful.
- **Safe process control:** expose only actions supported by the active platform capability; keep graceful and force stop distinct, and require fresh identity validation. Cross-platform parity means equivalent safe intent where native capabilities exist, not identical action availability.
- **Explainable and conservative:** show observable evidence; represent missing data honestly; do not claim internet exposure from a wildcard bind alone.
- **Focused utility:** no general-purpose system monitor, IDE, terminal, Git client, container manager, or automatic project execution.

## Release boundary

### P0 — initial core

TCP listener discovery; address, port, and ownership mapping when available; process name/PID and available executable, command, and working-directory metadata; manual and sensible automatic refresh; search/filter; local URL and copy actions; capability-supported graceful and force-stop actions with separate semantics; macOS menu-bar and Windows system-tray access; and clear loading, empty, permission, unavailable-metadata, process-ended, and error states. Graceful Stop requests normal shutdown and is unavailable when the platform does not support it. Force Stop is a separate explicit request; it is never substituted automatically. See the traceable acceptance criteria in [Functional Requirements](FUNCTIONAL-REQUIREMENTS.md).

### P1 — after P0 stability

Project root and marker detection, `.git` repository/branch context, resource information, process parent/tree, and project-oriented terminal/editor/folder actions. Project root and Git context are **not** P0 acceptance criteria. They may be included in an initial release only after P0 has passed both native platform acceptance suites and the added work does not significantly destabilize parity.

### P2 — later intelligence

Evidence-based runtime/framework/package-manager detection, best-effort runtime version, port-conflict assistance, and fuller network-exposure explanations. Never execute arbitrary project code to identify runtimes or claim exposure beyond observed binding facts.

### P3 — deferred

Opt-in/configurable watch/favorite/notification behavior and limited Docker/container context, after separate requirements and safety design.

## Explicit exclusions

Unless a later requirement change is approved: cloud accounts/sync, telemetry, AI APIs, remote monitoring, packet sniffing, Linux release, general activity-monitor replacement, full Docker/Kubernetes management, IDE/terminal/Git-client replacement, arbitrary project execution, and automatic port-conflict interception.

## Success and freeze

P0 is successful only when users can find a listener, identify its process when the OS allows it, inspect available metadata, understand local binding conservatively, and safely use the actions supported on macOS and Windows. Platform parity means equivalent core intent where native safe capabilities exist; it does not require every platform to expose every action. The baseline identifier remains THAA-REQ-0.1; its W012.0.1 action clarification is recorded in [Product Rules](PRODUCT-RULES.md) and [Functional Requirements](FUNCTIONAL-REQUIREMENTS.md). Other changes require a rationale, affected requirement IDs, scope/platform/security/test impact, and review before implementation. See [traceability](FUNCTIONAL-REQUIREMENTS.md#traceability).
