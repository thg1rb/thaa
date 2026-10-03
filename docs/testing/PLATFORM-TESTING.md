# Platform Testing and Parity

**Targets:** macOS and Windows · **Status:** W008.1 establishes GitHub-hosted macOS/Windows build and test runners; Windows provider remains unimplemented

## Execution requirements

W007 adds deterministic tests for shared `PortProvider` result/error semantics. W008 adds controlled native macOS listener evidence. W008.1 runs existing tests and application builds on GitHub-hosted macOS 15 arm64 and Windows Server 2025 x64 runners. This establishes execution/build environments, not Windows provider behavior. W009 must add native Windows provider integration tests and verify those tests execute on the Windows runner. Mocks/fakes verify shared application contracts but cannot establish native behavior. Compilation on one platform is not validation of the other. Record platform version/environment, test class, result, and limitation; `NOT RUN` is not `PASS`.

Tests create their own sockets/processes, use ephemeral ports and controlled children, signal readiness explicitly, bound waits, and clean up in all paths. They must not inspect/terminate arbitrary user processes or need administrator/root privileges in normal cases.

## Capability parity matrix

"Required" is an expected P0 behavior on each native platform; "Best Effort" allows explicit unavailable/permission outcomes where the OS prevents observation; "Capability-based" requires correct disclosure and behavior only when the platform supports the action; "Unsupported" means the platform cannot provide the capability and must report that accurately. Implementation status is recorded per platform in the final column; expected capability is not a claim of cross-platform completion.

| Capability                    | macOS expectation                     | Windows expectation                   | Shared evidence                                                            | Implementation status                                                            |
| ----------------------------- | ------------------------------------- | ------------------------------------- | -------------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| Listening TCP discovery       | Required                              | Required                              | Controlled random-port listener appears/disappears                         | macOS implemented (W008); Windows Not Yet Implemented                            |
| Protocol, local address, port | Required for discovered listeners     | Required for discovered listeners     | Normalize; cover IPv4 and IPv6 where host supports them                    | macOS IPv4/IPv6 loopback and wildcard tested (W008); Windows Not Yet Implemented |
| PID ownership                 | Best Effort                           | Best Effort                           | Resolve where native provider reports owner; otherwise explicit unresolved | macOS controlled owner tested (W008); Windows Not Yet Implemented                |
| Process PID/name              | Required when inspectable             | Required when inspectable             | Controlled child contract; denied/protected state explicit                 | Not Yet Implemented                                                              |
| Executable path/command line  | Best Effort                           | Best Effort                           | Field-level available/unavailable; do not fabricate                        | Not Yet Implemented                                                              |
| Working directory             | Best Effort, P0 where available       | Best Effort, P0 where available       | No project-root/Git inference                                              | Not Yet Implemented                                                              |
| Parent PID                    | Best Effort, later P1 context         | Best Effort, later P1 context         | No P1 feature implementation in W004                                       | Not Yet Implemented                                                              |
| Loopback/broader binding      | Required when address is determinable | Required when address is determinable | Same pure classifier and conservative labels                               | Not Yet Implemented                                                              |
| Graceful stop                 | Capability-based                      | Capability-based                      | Explicit action/result, never silently force                               | Not Yet Implemented                                                              |
| Force stop                    | Capability-based, explicit fallback   | Capability-based, explicit fallback   | Explicit confirmation and revalidated identity                             | Not Yet Implemented                                                              |
| Tray access                   | macOS menu bar                        | Windows system tray                   | Platform-specific UX with shared snapshot                                  | Not Yet Implemented                                                              |
| Permission/error states       | Required                              | Required                              | Denied/unavailable/disappeared/provider failure do not crash               | Not Yet Implemented                                                              |

Do not force identical native behavior where capability differs. The frontend and API must expose the documented capability/outcome; user-facing claims remain limited to evidence.

## CI runner evidence

The GitHub Actions workflow runs the full Rust suite on each native platform.
The macOS job also checks `/usr/sbin/lsof` and runs W008 controlled-listener
integration tests. The Windows job runs shared Rust/domain/contract tests and
builds Thaa with the native Windows toolchain; no Windows provider exists yet,
so there are no Windows-native listener tests to report. Shared frontend and
documentation checks run on Ubuntu 24.04 x64. These are GitHub-hosted runners,
not self-hosted machines; runner/toolchain versions and the actual results
must be recorded from each workflow run. macOS 15 arm64 CI evidence is distinct
from the local macOS 27 arm64 evidence below.

## macOS native scenarios

W008 validates controlled IPv4 loopback discovery and absence after close, PID
mapping to the test process, IPv4 wildcard, IPv6 loopback, IPv6 wildcard, and
NUL field parser behavior on macOS 27.0 arm64 with `/usr/sbin/lsof` 4.91.
The provider uses fixed direct arguments and tests its bounded runner. Other
macOS versions, real permission-denial behavior, and executable-unavailable
behavior remain future environment-specific checks. Later work validates
controlled process metadata and graceful/force stop; check process/descriptor
cleanup. Compilation alone is insufficient.

## Windows native scenarios

On a Windows runner/machine, W009 must validate native listener discovery and
PID mapping, IPv4/IPv6 where available, address classification, process
metadata, supported action behavior, protected/denied outcomes where safely
reproducible, and native handle cleanup. W008 does not validate Windows.
PowerShell is not the permanent provider contract. Compilation alone is
insufficient.

## Shared race and long-running scenarios

Native runs cover listener closure and controlled-process exit/change where deterministic. Exact PID reuse is simulated with identity fakes; tests must never wait for real PID reuse. Combine these with shared refresh barriers to verify no overlap, stale-result rejection, and no duplicate scans across window/tray consumers. Repeat refresh and listener/process churn to measure resource growth, open handles, subprocess frequency, idle CPU/memory, scan duration, and UI responsiveness before performance claims.

## Environment and limitation recording

Document OS/tool versions, permissions, IPv4/IPv6 availability, listener/process counts, test class, pass/fail/blocked/not-run status, and any unsupported capability. A disabled environment fixture or unavailable permission is not silently counted as a pass. Numeric performance limits and exact runner images remain deferred until measurement and CI selection.
