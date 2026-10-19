# W025 — Windows Working Directory Feasibility

## Status

**Feasibility spike complete — Windows Working Directory remains unavailable.**

W025 is research and feasibility evidence only. It does not implement Working
Directory discovery and does not change the Windows provider. The recommended
disposition is **DEFER WINDOWS CWD DISCOVERY**: no documented, supported API
was identified that returns the current directory of an arbitrary existing
process with the least-privilege contract Thaa requires. Research did not prove
that no such mechanism could ever exist; the known internal-process approach is
not suitable for production without a separate approved investigation.

## Objective and Requirement Mapping

Determine whether Thaa can safely and maintainably obtain the actual Current
Working Directory of an arbitrary existing Windows process. This is distinct
from the executable path, command line, a process's launch directory, a Git
root, or an inferred project directory.

- **FR-006 (P0):** expose Working Directory when available and represent
  unavailable metadata explicitly.
- **FR-012 (P1):** detect Project Root from an observed Working Directory.
- **FR-013 (P1):** derive Git repository context from observed Working
  Directory.
- **FR-014 (P1):** offer project actions when a validated Project Root and
  platform capability are available.
- **NFR-005/NFR-006:** preserve identity and snapshot correctness, fail safely
  under changing state, and avoid unbounded or blocking inspection work.

The W025 investigation itself is **P1 Platform Capability Investigation**. The
Product Requirements place project-root and Git context in the P1 group and
FR-006 says their inference remains P1; the W024 work record labels FR-012 and
FR-013 P2. W025 records this existing priority inconsistency without changing
requirement wording or status.

### Dependency and platform matrix

| Capability               | Requires actual Working Directory?              | Additional dependency                                 | Current Windows status                                                     |
| ------------------------ | ----------------------------------------------- | ----------------------------------------------------- | -------------------------------------------------------------------------- |
| FR-006 Working Directory | It is the target metadata                       | Process identity, permission to observe               | `Unavailable(ProviderLimitation)`                                          |
| FR-012 Project Root      | Yes; discovery starts at the observed directory | Existing nearest-marker logic and filesystem access   | Unavailable because Working Directory is unavailable                       |
| FR-013 Git Context       | Yes; discovery starts at the observed directory | Git executable and bounded read-only repository query | Unavailable because Working Directory is unavailable                       |
| Reveal Project Folder    | Yes, through validated Project Root             | Native Windows file-manager integration and action UI | No usable Project Root; metadata alone would not implement the integration |
| Open Project in Terminal | Yes, through validated Project Root             | Windows terminal integration and action UI            | No usable Project Root; separate platform integration required             |
| Open Project in Editor   | Yes, through validated Project Root             | Windows editor integration and action UI              | No usable Project Root; separate platform integration required             |

Obtaining Working Directory would unlock inputs to existing cross-platform
Project Root and Git logic. It would not by itself complete the native Windows
integrations or their UI capability gates.

## Existing Windows Provider

The current `WindowsProcessProvider` opens a requested PID with
`PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE`, retains an owned
process handle through inspection, checks liveness, queries process start time,
image path, and resource samples, then checks liveness again. The image path is
queried with `QueryFullProcessImageNameW`. The handle anchors observations to a
process object across PID reuse during the inspection. Handles are closed by
RAII. Listener discovery separately uses the documented IP Helper
`GetExtendedTcpTable` API.

Working Directory and structured command arguments are explicitly
`Unavailable(ProviderLimitation)`. Project Root and Git remain absent when
their required input is unavailable. Optional metadata failure does not
invalidate listener/process inspection. No new dependency, API, permission,
or application behavior was introduced by W025.

The current provider identity and access contract is documented in
[W011.2](W011.2-windows-process-provider.md) and implemented in
`src-tauri/src/platform/windows/process_provider.rs` and
`src-tauri/src/platform/windows/process_native.rs`.

## API Inventory

| API / mechanism                                | Documentation status                        | Information available                                                                   | Arbitrary target CWD?                                               | Rights / limitations                                                                | W025 assessment                                                   |
| ---------------------------------------------- | ------------------------------------------- | --------------------------------------------------------------------------------------- | ------------------------------------------------------------------- | ----------------------------------------------------------------------------------- | ----------------------------------------------------------------- |
| `GetCurrentDirectoryW`                         | Documented Win32 API                        | Calling process's current directory                                                     | No                                                                  | No target-process handle parameter; reads caller state                              | Insufficient                                                      |
| `CreateProcessW.lpCurrentDirectory`            | Documented Win32 API                        | Supplies initial directory for a newly created process                                  | No; creation only                                                   | Applies when creating a process; target may later change its directory              | Not a retrieval mechanism                                         |
| `QueryFullProcessImageNameW`                   | Documented Win32 API                        | Target executable image path                                                            | No                                                                  | Current provider uses limited query rights; image path is not CWD                   | Insufficient; do not substitute executable path                   |
| `GetProcessImageFileNameW`                     | Documented Win32 API                        | Target executable image path, device-form path                                          | No                                                                  | Query-information or limited-query handle; image path is not CWD                    | Insufficient                                                      |
| WMI `Win32_Process`                            | Documented WMI class                        | PID, creation date, executable path, command line, parent PID, and other process fields | No documented `CurrentDirectory` property                           | WMI/COM query and provider overhead; some properties have privilege qualifiers      | Insufficient; command line is not CWD                             |
| `NtQueryInformationProcess`                    | Documented but explicitly internal/unstable | Selected process information classes; availability and structures vary                  | No documented CWD contract                                          | Microsoft warns it may change or be unavailable; classes/rights differ              | Unsuitable as a stable CWD contract                               |
| PEB / `RTL_USER_PROCESS_PARAMETERS` inspection | Internal implementation detail              | Process parameters in user-mode process memory                                          | No supported contract                                               | Layout/offset, target architecture, memory access, and concurrent mutation concerns | Not approved or suitable for production based on current evidence |
| `ReadProcessMemory`                            | Documented memory API                       | Reads a specified region in another process                                             | Only as an implementation building block, not a supported CWD query | Requires `PROCESS_VM_READ`; protected-process restrictions apply                    | Prohibited by W025 scope; not recommended                         |

The absence of a `CurrentDirectory` property in the documented `Win32_Process`
schema is an observation about that schema, not proof that every Windows
management interface lacks such a field. No authoritative supported API
contract for querying an arbitrary process's current directory was found in
this spike.

### Authoritative references

- Microsoft, [`GetCurrentDirectory`](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-getcurrentdirectory): retrieves the current directory for the calling process.
- Microsoft, [`CreateProcessW`](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessw): `lpCurrentDirectory` specifies the initial directory of a process being created.
- Microsoft, [`QueryFullProcessImageNameW`](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-queryfullprocessimagenamew): retrieves an executable image path, not a current directory.
- Microsoft, [`GetProcessImageFileNameW`](https://learn.microsoft.com/en-us/windows/win32/api/psapi/nf-psapi-getprocessimagefilenamew): retrieves the target executable file name/path.
- Microsoft, [`Win32_Process`](https://learn.microsoft.com/en-us/windows/win32/cimwin32prov/win32-process): documented process properties include `CommandLine`, `CreationDate`, and `ExecutablePath`; no `CurrentDirectory` property is listed.
- Microsoft, [`NtQueryInformationProcess`](https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntqueryinformationprocess): warns that the function and returned structures are internal, subject to change or removal, and advises using public alternatives.
- Microsoft, [`RTL_USER_PROCESS_PARAMETERS`](https://learn.microsoft.com/en-us/windows/win32/api/winternl/ns-winternl-rtl_user_process_parameters): warns the structure may change; the documented public layout exposes reserved fields, image path, and command line, not a CWD contract.
- Microsoft, [process security and access rights](https://learn.microsoft.com/en-us/windows/win32/procthread/process-security-and-access-rights): defines query, VM-read, and synchronization rights and protected-process restrictions.
- Microsoft, [`ReadProcessMemory`](https://learn.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-readprocessmemory): reads another process's address space and requires VM-read access.
- Microsoft, [`IsWow64Process2`](https://learn.microsoft.com/en-us/windows/win32/api/wow64apiset/nf-wow64apiset-iswow64process2): can identify target/host machine information; it does not provide CWD.

## Feasibility and Alternatives

### Public API conclusion

`GetCurrentDirectoryW` queries the caller. `CreateProcessW` can set the initial
directory only at creation. Process-query APIs in the current provider expose
process identity and executable metadata, not the target's current directory.
The documented WMI class provides useful process context but no current
directory field. The public API research therefore found **no supported,
least-privilege arbitrary-process CWD API**.

The potential internal route is to inspect process parameters through
implementation-specific process structures and remote memory. This is not an
approved prototype path. Microsoft explicitly cautions that native process
information APIs and structures can change or disappear. Remote memory reads
require `PROCESS_VM_READ`; Microsoft documents that right as unavailable for
protected processes. Broadening access or enabling debug privilege would
conflict with Thaa's least-privilege model. No such experiment was performed.

### Alternatives

| Alternative                        | Coverage                                                 | Benefits                                                        | Risks / limitations                                                                                                                                     | Recommendation                                                                                |
| ---------------------------------- | -------------------------------------------------------- | --------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| Keep Working Directory unavailable | Arbitrary existing processes remain unsupported          | Honest, accurate, no added access or data collection            | Windows users lack cwd-derived project context                                                                                                          | Keep as current behavior                                                                      |
| User-selected Project Association  | Only processes/listeners explicitly associated by a user | Uses a user-confirmed directory; can support existing processes | Requires mapping lifecycle, process-instance identity, stale/incorrect association handling, persistence/privacy decisions; does not satisfy FR-006 CWD | Possible separate product work after approval; label as association, never CWD                |
| Cooperative process metadata       | Participating processes or integrations only             | Explicit data and potentially low privilege                     | Requires agent/protocol participation, authentication, freshness and spoofing controls; misses arbitrary processes                                      | Not a general solution; separate product decision                                             |
| Thaa-managed process launch        | Processes launched by Thaa                               | Thaa can record the supplied initial CWD and launch identity    | Does not cover existing arbitrary processes or later `SetCurrentDirectory` changes; requires a launch feature                                           | Narrow future capability only; do not call it observed current CWD                            |
| Inferred project context           | Heuristic subset                                         | May provide hints without process-memory access                 | Can misattribute; executable path/arguments/Git root are not current CWD                                                                                | Do not show as factual CWD; any hint requires separate product approval and explicit labeling |

## Identity, Consistency, Security, and Performance

- **PID reuse:** a future query must remain associated with the same held
  process handle and observed start time used for the accepted process
  identity. PID alone must not key cached metadata.
- **Process exit and snapshot races:** query using an identity-bound handle,
  check liveness and identity before accepting metadata, and commit only to
  the corresponding scan generation. On failure or mismatch, mark the field
  unavailable. The current scan generation/identity model is a suitable
  boundary to preserve, but no future provider was implemented here.
- **Mutable CWD:** `SetCurrentDirectory` changes process-level state. Even a
  technically correct read is a point-in-time observation and can become
  stale before a later project action. Actions must continue resolving a
  current accepted entry and revalidating the filesystem target.
- **Permissions:** ordinary same-user process queries are bounded by process
  security descriptors. Cross-user, elevated, system, or protected processes
  may be inaccessible. Denial must produce unavailable metadata, without
  retrying with broader rights or elevating.
- **Architecture:** current product/release evidence targets Windows 11 x64;
  hosted native CI is Windows Server 2025 x64. ARM64, x86 builds, WOW64 target
  inspection, and cross-bitness access were not validated. Any future internal
  structure-based method would need explicit architecture handling and a
  native test matrix; `IsWow64Process2` reports architecture but does not
  solve metadata retrieval.
- **Privacy:** filesystem paths may contain personal information. A future
  provider should avoid logging, uploading, or persisting them beyond the
  in-memory runtime snapshot and the existing user-visible context purpose.
- **Performance:** the current scan already performs bounded process-level
  queries. A future method must query each process at most once per accepted
  snapshot, close handles deterministically, isolate per-process failures, and
  avoid blocking scan completion. No performance measurements for CWD
  discovery were possible because no candidate API was selected or run.
- **Resource safety:** no experimental process handle or memory resource was
  opened by this spike. A later public-API experiment must use a disposable
  child, least privilege, bounded execution, and RAII cleanup.

### Threat assessment

| Threat                                | Boundary / precondition                                         | Impact                                              | Mitigation                                                                                 | Residual risk                                      |
| ------------------------------------- | --------------------------------------------------------------- | --------------------------------------------------- | ------------------------------------------------------------------------------------------ | -------------------------------------------------- |
| PID reuse / wrong process association | PID changes between listener scan and metadata query            | Wrong project attribution                           | Hold the process handle; compare creation time; bind to snapshot generation                | Process may exit during query; report unavailable  |
| Mutable or stale CWD                  | Target changes CWD after observation                            | Later action targets stale path                     | Treat as timestamped/best-effort snapshot data; validate action target again               | CWD can change between scan and action             |
| Access denied                         | Target has stricter ACL, different owner/session, or protection | Metadata unavailable                                | Fail closed with explicit unavailable reason                                               | Some processes remain unsupported                  |
| Internal API drift                    | Use of NT APIs or private structure layout                      | Crash, incorrect path, compatibility failure        | Do not use for production; require separate approval and compatibility evidence            | No stable support promise for internals            |
| Cross-process memory disclosure       | Remote memory reading                                           | Exposure of unrelated process data; expanded rights | W025 prohibits it; do not request VM-read for optional metadata                            | Cannot use this route under current scope          |
| Privilege escalation                  | Attempt to inspect inaccessible target                          | Broad system access and user surprise               | Never elevate or enable debug privilege; no retries with wider rights                      | Metadata remains unavailable                       |
| Incorrect heuristic fallback          | Infer CWD from executable or arguments                          | Misleading Project Root/Git/actions                 | Keep CWD unavailable; label any future inference as a separate field                       | Product may still desire explicit association      |
| Sensitive path disclosure             | CWD appears in diagnostics or logs                              | User privacy loss                                   | No full-path logs by default; no external transmission/persistence                         | Path is intentionally visible in UI when available |
| Scan denial of service                | Slow/blocked provider call per PID                              | Refresh latency/resource use                        | Bounded, per-process optional query and scan-level limits before any future implementation | No candidate performance measurements exist        |

## Windows Compatibility and Validation Evidence

The repository's v0.1.1 tester guide and release record target **Windows 11
x64**. Existing native CI runs on the hosted `windows-2025` x64 runner. No
broader supported Windows client range or ARM64 target is established by the
current release documentation. The platform CI is useful native compilation
and test evidence, but it is not a Windows desktop session.

This spike ran on macOS 27.0.0 arm64. No Windows environment, desktop, or
interactive Windows session was available. No Windows API runtime experiment,
benchmark, harness, remote-memory read, privilege escalation, injection,
process modification, or provider change was performed. Windows CI results
from prior work are historical provider evidence, not W025 CWD evidence.

| Evidence class                    | Result  | Limitation                                                                                  |
| --------------------------------- | ------- | ------------------------------------------------------------------------------------------- |
| Repository/source inspection      | PASS    | Establishes current implementation and target scope only                                    |
| Official API documentation review | PASS    | No public arbitrary-process CWD API contract identified; not proof none exists              |
| Native Windows W025 experiment    | NOT RUN | No Windows host available; no experiment required to support the conservative defer outcome |
| Interactive Windows validation    | NOT RUN | No Windows desktop available                                                                |
| W025 performance measurement      | NOT RUN | No candidate implementation/API selected                                                    |

## Feasibility Decision

**Classification: D — DEFER.**

**Go/no-go: DEFER WINDOWS CWD DISCOVERY.**

The reviewed documented APIs do not provide the target process's current
directory. The apparent user-mode process-parameter route depends on internals
and remote-memory access that W025 prohibits and that are not suitable as a
stable production contract under the current security and compatibility
requirements. No native Windows experiment was run, so the result is a
documentation-backed go/no-go recommendation rather than a claim of runtime
impossibility.

If Windows project context is prioritized later, first obtain a separate
product decision on a **user-selected Project Association** or another
cooperative workflow. Such a value must be represented as an association or
provided context, not as a verified process Working Directory. Any new
investigation of undocumented memory access requires separate explicit
approval and is not implied by this report.

## Acceptance Criteria

| Criterion                                           | Status | Evidence                                                                                                       | Limitations                                               |
| --------------------------------------------------- | ------ | -------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------- |
| AC-01 — Requirements verified                       | PASS   | FR-006/012/013/014 and provider dependencies documented above                                                  | Requirement completion is unchanged                       |
| AC-02 — API inventory                               | PASS   | Official references and API comparison table                                                                   | Research does not establish that no API exists anywhere   |
| AC-03 — Supported/internal mechanisms distinguished | PASS   | Public APIs, WMI, native NT API, PEB/process parameters, and memory reads classified separately                | Internal mechanism not experimentally evaluated           |
| AC-04 — Least privilege                             | PASS   | Current rights documented; `PROCESS_VM_READ` and protected-process limitations cited; no elevation recommended | No target-specific denial tests in this environment       |
| AC-05 — Identity and races                          | PASS   | PID reuse, held-handle identity, process exit, mutable CWD, snapshot generation addressed                      | No CWD provider exists to exercise these races            |
| AC-06 — Compatibility                               | PASS   | Windows 11 x64 target and Windows Server 2025 x64 CI identified; ARM64/cross-bitness marked unverified         | No broad compatibility claim made                         |
| AC-07 — Performance                                 | PASS   | Costs and required future measurement strategy assessed; no invented benchmark                                 | No measurement possible without an implementation         |
| AC-08 — Security                                    | PASS   | No code or privilege changes; internal memory route rejected for this spike                                    | Future alternative requires its own threat review         |
| AC-09 — Native evidence accuracy                    | PASS   | Windows runtime and interactive results explicitly marked NOT RUN                                              | No native W025 evidence                                   |
| AC-10 — Go/no-go recommendation                     | PASS   | Defer recommendation and alternative product path provided                                                     | Advisory only; production work requires separate approval |
| AC-11 — Independent review                          | PASS   | Independent read-only security and architecture reviews found no material or minor findings                    | Reviewers made no repository changes                      |
| AC-12 — Scope compliance                            | PASS   | Production provider and application code unchanged                                                             | Documentation-only changes remain                         |

## Review and Documentation PR Gates

- Security reviewer (`/root/w024_readonly_review`): PASS; no findings.
- Architecture reviewer (`/root/w025_feasibility_plan_review`): PASS; no
  material findings. Priority sources and the W024 discrepancy were rechecked.
- Documentation formatting: PASS with Prettier 3.9.9 on the changed Markdown.
- Local Markdown links: PASS (`pnpm docs:check`, 88 Markdown files).
- `git diff --check`: PASS.
- PR #73 (`816e08e7f9e7515e3c05ae22549f6b3425827a14`): Shared Quality, macOS
  Native Validation, and Windows Native Validation all passed in run
  [39217548815](https://github.com/thg1rb/thaa/actions/runs/39217548815).
- PR #73 merged to `develop` as
  `77f5e43049ed1f62a11fa8abc5fe2a3f020fa49c`. Post-merge CI passed on that
  merge commit in run
  [39218294122](https://github.com/thg1rb/thaa/actions/runs/39218294122).
- This documentation closeout records research and feasibility only; it is
  not Windows Working Directory feature completion or production approval.

## Scope Record

```text
Research completed: YES
Windows Working Directory feature implemented: NO
Production Windows provider modified: NO
Remote-memory prototype implemented: NO
Privilege escalation performed: NO
W026 started: NO
Release or main branch changed: NO
```

## Sources

Repository evidence: W011.2, `docs/requirements/FUNCTIONAL-REQUIREMENTS.md`,
`docs/requirements/NON-FUNCTIONAL-REQUIREMENTS.md`,
`docs/work/W008.1-cross-platform-ci-baseline.md`,
`docs/releases/v0.1.1-tester-guide.md`,
`src-tauri/src/platform/windows/process_provider.rs`, and
`src-tauri/src/platform/windows/process_native.rs`.
