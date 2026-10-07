# Engineering Security Baseline

**Applies to:** `THAA-REQ-0.1` and the W003 architecture baseline · **Status:** Initial baseline

This document sets security rules for implementation and review. It complements the public vulnerability reporting instructions in the repository-root `SECURITY.md`; it does not replace the threat inventory in [THREAT-MODEL.md](THREAT-MODEL.md).

## Security objectives

- Protect users from actions against the wrong process and from accidental system damage.
- Keep process, filesystem, and project context local and minimize unnecessary exposure.
- Use least privilege; normal inspection must not silently request or obtain administrator/root access.
- Treat operating-system metadata, subprocess output, frontend IPC, stored configuration, and dependency/release inputs as untrusted.
- Report observable capability and action outcomes accurately; never fabricate metadata or imply stronger guarantees than the OS provides.

## Trust boundaries and data handling

W003 defines the layer model: frontend → Tauri boundary → application → domain ports → platform adapters → OS. Every crossing is a validation/normalization boundary. Tauri IPC is untrusted even when the packaged frontend is the normal caller. See the trust-boundary diagram and threats in [THREAT-MODEL.md](THREAT-MODEL.md).

Process names, command lines, paths, listener data, native return values, subprocess output, and configuration must be treated as untrusted input. Normalize and bound data before it reaches shared models or presentation. Do not execute project files or infer trust from a local path.

## Process-action rules

- Never target by process name or a name-matching group.
- Carry the observed `ProcessIdentity` into the action request. `ProcessController` performs authoritative revalidation inside the platform adapter immediately before the native action, comparing PID and all available stable identity evidence such as executable and start time.
- Reject mismatches, disappearance, insufficient target confidence, and permission failures. Do not silently retry against a new identity.
- Keep graceful stop and force stop as separate explicit requests. Never silently escalate a failed graceful request.
- Test destructive actions only against controlled child processes created by the test.
- Avoid known protected/system targets where they can be identified safely. If the OS denies access or action, report the denial; do not elevate automatically.

Revalidation narrows but cannot necessarily eliminate the time-of-check/time-of-use race. Platform implementations must document any residual race and report actual action outcomes.

## Process, shell, path, and URL boundaries

- Prefer native APIs. If a subprocess is necessary, use a fixed executable and argument array, bounded output/time, and explicit error handling. Never construct a shell command from PID, process name, arguments, address, port, or path; do not invoke a shell merely to pass untrusted metadata.
- Escape/control untrusted strings for display; prevent markup interpretation and terminal escape/control-sequence effects. Bound extremely long values and preserve an explicit truncation indication.
- Build local listener URLs only from validated protocol, host/address, and port fields. Use a safe platform URL-opening API. Never open arbitrary command-line or process metadata as a URL.
- Future filesystem/terminal/editor actions require an explicit user action, validated target, structured OS API/argument passing, and clear handling of missing paths and symlinks. Do not resolve, execute, or open paths merely because process metadata names them.

## Native APIs and unsafe Rust

Prefer safe Rust APIs and documented native interfaces. Keep unavoidable `unsafe` blocks minimal and localized in adapter wrappers. Each block must state the safety invariant it relies on; validate native inputs, result lengths, status codes, pointer/null assumptions, handle ownership, and buffer lifetimes. Release OS handles/resources on every success and error path. Map failures to W003's typed provider/action errors and never pass native error strings directly to the UI.

Native/FFI changes require security review before merge. W004 does not choose native crates or APIs.

W009's Windows listener adapter uses only the documented `GetExtendedTcpTable`
API and Microsoft-maintained `windows-sys` bindings. Keep the dependency under
`cfg(windows)` and enable only required features. Native buffers must remain
bounded and correctly aligned; check all size arithmetic and validate the
reported entry count against physically available row bytes before parsing.
Preserve these invariants in every unsafe block comment and test malformed
table layouts through safe synthetic byte fixtures. Listener inspection must
not request elevation or query process metadata beyond the owner PID already
present in the TCP table.

W011.2's Windows process adapter uses the same target-specific Microsoft
binding with only `Win32_System_Threading` added. It requests limited query
and synchronization rights only; it must never request VM access, debug
privilege, or termination rights. One RAII-owned handle anchors inspection to
one process object and is closed before return. Keep unsafe calls local and
document valid-handle, buffer, output-initialization, and lifetime invariants.
Bound UTF-16 image-path allocation, convert native paths without lossy UTF-8,
and keep raw Win32 codes out of provider Display output. Command arguments and
working directories remain unavailable rather than being reconstructed or
inferred; do not read a remote PEB or process memory.

## Privacy and diagnostics

Do not log by default:

- complete command lines or arguments, which may contain credentials
- authentication tokens, environment variables, credentials, or private file contents
- full user-specific paths or repository details unless a specific diagnostic need is justified

Prefer stable error categories and bounded operational context. Redact or omit sensitive values before logging; do not assume a local log is harmless. UI and screenshot guidance must remind contributors that usernames, project names, paths, command arguments, and local service information may be visible. Do not claim screenshots can be made fully safe automatically.

## Configuration and dependencies

No configuration subsystem exists yet. When introduced, parse configuration as untrusted input, validate types/ranges/known keys, use safe defaults, and never allow configuration to disable process identity checks, permission handling, or privacy rules silently.

For every dependency, record need, source/maintainer trust, maintenance/security history, license compatibility, and alternatives where material. Avoid duplicate or unnecessary dependencies, especially native/FFI dependencies. Commit lockfiles where applicable; run ecosystem advisory/audit and license checks when tooling is selected in bootstrap/CI work. Pin GitHub Actions to immutable references and use minimum workflow permissions. Review packaged/update artifacts and signing in release work; W004 does not configure release tooling.

## Security severity and merge policy

Use the existing finding levels: Critical, High, Medium, Low, Suggestion. Unresolved Critical or High security issues block merge. Medium findings block when they leave a required safety control, P0 acceptance criterion, or test obligation unsatisfied; otherwise document owner, follow-up work, and rationale. Record Low findings and Suggestions without inflating severity. The main Agent resolves valid findings; a reviewer does not patch.

## Mandatory security review triggers

Require a documented security review for work involving process termination, shell/subprocess execution, native FFI, URL opening, filesystem/external-application opening, new sensitive Tauri commands, persistence/configuration, network behavior, startup/autostart, installer privileges, or dependencies with native/system access. Trivial copy/layout changes do not require a threat-model rewrite.

## W004 scope and traceability

This baseline supports NFR-001–NFR-005 and NFR-010, PR-001–PR-008, and FR-007–FR-009. It establishes future implementation obligations; it does not claim any functional requirement or mitigation is implemented. Detailed threat-to-test mappings are in [TEST-CASES.md](../testing/TEST-CASES.md).

W013.1 icon enrichment is untrusted presentation input. It accepts only a
backend-observed `ProcessInfo` path on Windows; no frontend path is passed to
icon extraction. Native icon output is size- and dimension-bounded and PNG
validated before IPC. macOS uses public AppKit application lookup. Icon bytes,
references, and bundle/executable icon metadata do not enter process identity,
action targets, or controller authorization. Native Windows handles and GDI
resources are released locally on every path. Icon failures remain fallback
presentation and are not logged with process metadata.

W017 resource collection is read-only and uses the process inspection rights
already needed by each provider. CPU, resident-memory, and uptime values are
optional snapshot metadata only; they do not enter identity/action targets.
Windows queries reuse and close the provider's RAII handle. macOS confines
libproc FFI to the existing C bridge. The shared CPU baseline is bounded to
the current accepted process snapshot and keyed by PID plus start time, so a
reused PID cannot inherit old sampling data. Do not log metric values or add
elevation to make protected-process metrics available.
