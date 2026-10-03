# Platform Testing and Parity

**Targets:** macOS and Windows · **Status:** W008 and W009 establish native listener-provider evidence on GitHub-hosted runners; user-facing monitoring remains unimplemented

## Execution requirements

W007 adds deterministic tests for shared `PortProvider` result/error semantics. W008 adds controlled native macOS listener evidence. W008.1 runs existing tests and application builds on GitHub-hosted macOS 15 arm64 and Windows Server 2025 x64 runners. W009 adds native Windows provider integration tests to that Windows job. These are GitHub-hosted runners, not self-hosted physical machines. Mocks/fakes verify shared application contracts but cannot establish native behavior. Compilation on one platform is not validation of the other. Record platform version/environment, test class, result, and limitation; `NOT RUN` is not `PASS`.

Tests create their own sockets/processes, use ephemeral ports and controlled children, signal readiness explicitly, bound waits, and clean up in all paths. They must not inspect/terminate arbitrary user processes or need administrator/root privileges in normal cases.

## Capability parity matrix

"Required" is an expected P0 behavior on each native platform; "Best Effort" allows explicit unavailable/permission outcomes where the OS prevents observation; "Capability-based" requires correct disclosure and behavior only when the platform supports the action; "Unsupported" means the platform cannot provide the capability and must report that accurately. Implementation status is recorded per platform in the final column; expected capability is not a claim of cross-platform completion.

| Capability                    | macOS expectation                     | Windows expectation                   | Shared evidence                                                            | Implementation status                                                    |
| ----------------------------- | ------------------------------------- | ------------------------------------- | -------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
| Listening TCP discovery       | Required                              | Required                              | Controlled random-port listener appears/disappears                         | macOS native tests (W008); Windows native tests (W009)                   |
| Protocol, local address, port | Required for discovered listeners     | Required for discovered listeners     | Normalize; cover IPv4 and IPv6 where host supports them                    | macOS IPv4/IPv6 loopback and wildcard (W008); Windows (W009)             |
| PID ownership                 | Best Effort                           | Best Effort                           | Resolve where native provider reports owner; otherwise explicit unresolved | Controlled test process PID checked on both native providers (W008/W009) |
| Process PID/name              | Best effort when inspectable          | Best effort when inspectable          | Controlled child contract; denied/protected state explicit                 | macOS implemented/tested (W011.1); Windows W011.2 native CI pending      |
| Executable path               | Best Effort                           | Best Effort                           | Field-level available/unavailable; do not fabricate                        | macOS explicitly unavailable (W011.1); Windows W011.2 native CI pending  |
| Start time                    | Best Effort                           | Best Effort                           | Normalize process creation evidence                                        | macOS implemented/tested (W011.1); Windows W011.2 native CI pending      |
| Structured command arguments  | Best Effort                           | Best Effort                           | Preserve real argument boundaries or report unavailable                    | Unavailable on both selected providers (W011.1/W011.2)                   |
| Working directory             | Best Effort, P0 where available       | Best Effort, P0 where available       | No project-root/Git inference                                              | macOS implemented/tested (W011.1); Windows W011.2 limitation documented  |
| Parent PID                    | Best Effort, later P1 context         | Best Effort, later P1 context         | No P1 feature implementation in W004                                       | Not Yet Implemented                                                      |
| Loopback/broader binding      | Required when address is determinable | Required when address is determinable | Same pure classifier and conservative labels                               | Not Yet Implemented                                                      |
| Graceful stop                 | Capability-based                      | Capability-based                      | Explicit action/result, never silently force                               | Not Yet Implemented                                                      |
| Force stop                    | Capability-based, explicit fallback   | Capability-based, explicit fallback   | Explicit confirmation and revalidated identity                             | Not Yet Implemented                                                      |
| Tray access                   | macOS menu bar                        | Windows system tray                   | Platform-specific UX with shared snapshot                                  | Not Yet Implemented                                                      |
| Permission/error states       | Required                              | Required                              | Denied/unavailable/disappeared/provider failure do not crash               | Not Yet Implemented                                                      |

Do not force identical native behavior where capability differs. The frontend and API must expose the documented capability/outcome; user-facing claims remain limited to evidence.

## CI runner evidence

The GitHub Actions workflow runs the full Rust suite on each native platform.
The macOS job also checks `/usr/sbin/lsof` and runs W008 controlled-listener
and W011.1 controlled-process integration tests. The Windows job runs shared Rust/domain/contract tests,
W009 controlled native listener tests, and builds Thaa with the native Windows
toolchain. Shared frontend and
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
behavior remain future environment-specific checks. W011.1 additionally
validates a test-owned non-GUI `/bin/sleep` child on macOS 27.0 arm64: PID,
name, start time, working directory, explicitly unavailable executable/argv,
W010 contract behavior, W011 orchestration, and post-exit disappearance.
Process inspection uses bounded `/bin/ps` and `/usr/sbin/lsof` calls without
elevation. Other macOS versions and sandbox/distribution compatibility remain
unverified. W012 action behavior is separate future work. Compilation alone
is insufficient.

## Windows native scenarios

W009 validates native listener discovery and PID mapping on the GitHub-hosted
Windows 2025 x64 runner, with controlled IPv4/IPv6 loopback and wildcard
listeners plus post-close absence. Listener discovery does not validate
process actions. W011.2 adds process inspection evidence on `windows-2025`
x64 using a test-owned child running the integration-test executable. It
validates PID, executable path against the known test binary, its derived
file-stem name, creation time, explicit `ProviderLimitation` for arguments and
working directory, shared W010 assertions, W011 orchestration, and
`ProcessDisappeared` after the child exits. It requests no elevation and does
not inspect unrelated runner processes. These fields remain unsupported by
the public-API implementation. PowerShell is not used by the provider.
Compilation alone is insufficient.

## Shared race and long-running scenarios

Native runs cover listener closure and controlled-process exit/change where deterministic. Exact PID reuse is simulated with identity fakes; tests must never wait for real PID reuse. Combine these with shared refresh barriers to verify no overlap, stale-result rejection, and no duplicate scans across window/tray consumers. Repeat refresh and listener/process churn to measure resource growth, open handles, subprocess frequency, idle CPU/memory, scan duration, and UI responsiveness before performance claims.

## Environment and limitation recording

Document OS/tool versions, permissions, IPv4/IPv6 availability, listener/process counts, test class, pass/fail/blocked/not-run status, and any unsupported capability. A disabled environment fixture or unavailable permission is not silently counted as a pass. Numeric performance limits and exact runner images remain deferred until measurement and CI selection.
