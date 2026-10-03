# W009 — Windows Listening Port Provider

## Status

In Progress — implementation is on `feat/windows-listening-port-provider`.
Use `Done on merge` only after the reviewed PR merges and required post-merge
CI passes.

## Objective

Implement the W007 `PortProvider` contract on Windows using the documented
IP Helper API and prove discovery of controlled native listeners on the
Windows CI runner.

## Scope

- Add the Windows-only `WindowsPortProvider` under `platform/windows`.
- Query IPv4 and IPv6 owner-PID listener tables using `GetExtendedTcpTable`.
- Normalize addresses, nonzero ports, and reported PIDs into W006 values.
- Add checked bounded buffer acquisition, safe row validation/conversion,
  deterministic unit coverage, and Windows-native controlled-listener tests.
- Update architecture, platform testing, security, and this work record.

## Out of Scope

PowerShell, `netstat`, shell commands, UDP, process metadata/control, Tauri
commands, UI, refresh coordination, provider polling, elevated privileges,
and changes to frozen requirements or the W007 contract.

## Requirements

Provides Windows provider-side implementation evidence for FR-001/FR-002 and
normalized binding inputs for FR-003. It does not complete the user-facing
P0 Port Monitoring feature or FR-001–FR-003 end-to-end; refresh, presentation,
and other application work remains. The frozen baseline remains
`THAA-REQ-0.1`.

## Dependencies

W007 `PortProvider`/`PortScanResult`, W006 `NetworkListener`/`ProcessId`, W008
macOS provider semantics, W008.1 Windows hosted CI, and W008.2 native Windows
PR validation govern this implementation. The installed `rust-patterns` and
`security-and-hardening` Project Skills apply.

## Design Notes

Microsoft's [`GetExtendedTcpTable` documentation](https://learn.microsoft.com/en-us/windows/win32/api/iphlpapi/nf-iphlpapi-getextendedtcptable)
specifies that `TCP_TABLE_OWNER_PID_LISTENER` returns
`MIB_TCPTABLE_OWNER_PID` for `AF_INET` and `MIB_TCP6TABLE_OWNER_PID` for
`AF_INET6`. A null table pointer is the sizing query;
`ERROR_INSUFFICIENT_BUFFER` reports the required size. `bOrder` is false.
Microsoft documents the [IPv4 owner-PID row](https://learn.microsoft.com/en-us/windows/win32/api/tcpmib/ns-tcpmib-mib_tcprow_owner_pid)
and [IPv6 owner-PID row](https://learn.microsoft.com/en-us/windows/win32/api/tcpmib/ns-tcpmib-mib_tcp6row_owner_pid)
as network-representation address/port fields, with IPv6's address as a
16-byte array and its scope ID separately supplied.

Use Microsoft's [`windows-sys`](https://github.com/microsoft/windows-rs/blob/master/docs/crates/windows-sys.md)
0.61.2 binding as a Windows-only dependency with only Foundation, IpHelper,
and WinSock features. The crate is published from Microsoft's `windows-rs`
project under `MIT OR Apache-2.0`. Reuse the already locked version; no custom
Win32 declarations or new runtime abstraction are needed.

The native buffer is a zero-initialized `Vec<u64>`, providing at least 8-byte
alignment for the DWORD-aligned MIB rows. Bound each table to 16 MiB and make
at most three retrieval attempts after the sizing request. Use fallible
allocation, checked arithmetic, and validate the returned byte length,
flexible-array offset, row size, and entry count before parsing. The API calls
and bounded slice view are the only unsafe operations; row conversion uses
safe checked byte slices.

Map `ERROR_ACCESS_DENIED` to `PermissionDenied`, `ERROR_NOT_SUPPORTED` to
`Unsupported`, other API errors and exhausted buffer retries to
`OperatingSystemFailure`, and inconsistent native table data to
`ParseFailure`. A pathological size or allocation failure maps to
`ProviderFailure`. Preserve usable rows if one address-family query fails and
mark the result partial; if both fail, return the IPv4 error deterministically.
If both queries succeed with no rows, return a complete empty result. A
nonzero IPv6 scope ID is retained as an unknown address and marks the result
partial/Unsupported, matching W008's W006-model limitation. Preserve the
reported PID including zero; W006 defines no zero sentinel.

## Acceptance Criteria

- [ ] Windows adapter implements W007 without exposing Win32 types above the
      platform boundary.
- [ ] IPv4/IPv6 listener tables normalize addresses, ports, and PIDs correctly.
- [ ] Native buffer is aligned, initialized, bounded, retried finitely, and
      table rows are bounds-checked before access.
- [ ] Win32 failure, empty, partial, and malformed-table semantics are
      explicit and tested where deterministic.
- [ ] Unit tests cover address/port conversion, scope degradation, bounds,
      malformed rows, multiple rows, empty results, and error mapping.
- [ ] Windows native tests discover test-owned IPv4/IPv6 loopback and
      wildcard listeners, verify ephemeral ports and current-process PID, and
      observe disappearance after close.
- [ ] `windows-2025` PR CI executes the Windows integration target; macOS W008
      native tests and all existing gates remain passing.
- [ ] Dedicated read-only security/architecture review is complete, valid
      findings are fixed, and post-merge CI passes.
- [ ] No Windows discovery requirement is marked fully implemented as a
      user-facing feature.

## Validation

Local host: macOS 27.0 arm64, Rust 1.99.0. Host `cargo test`, Clippy, and
controlled macOS provider tests pass; the host release-profile Tauri
`--no-bundle` build passes. The Windows adapter, its unit tests, and the
Windows integration-test source pass `cargo check --tests` and Clippy in a
temporary Windows-target harness using the project-pinned compiler and
Microsoft binding. A full-crate Windows-target check is blocked locally by
the absence of `llvm-rc` required by Tauri's Windows resource build script;
neither check is native Windows evidence. `pnpm audit`, frontend gates, and
Markdown link checks pass. Local `cargo audit` is unavailable; the existing
CI installs pinned cargo-audit and audits both supported targets. Windows
native execution, Windows Tauri build, and final PR checks remain pending
GitHub Actions. Do not close W009 until the Windows job runs the controlled
listener tests successfully and post-merge CI passes.

## Security Considerations

The Windows table is untrusted native input. Allocation size and row count
are bounded and checked before parsing; malformed rows fail rather than
being silently discarded. Unsafe code is limited to direct documented API
calls and a byte-slice view whose validity is bounded by the initialized
aligned allocation. No process handles, process-enrichment APIs, shell,
PowerShell, automatic elevation, or raw error text are used.

## Platform Considerations

The implementation and dependency compile only on Windows. Native behavior
must execute on GitHub-hosted `windows-2025` x64. W008's macOS provider remains
unchanged. IPv6 scoped endpoints cannot be represented precisely by W006's
`Option<IpAddr>` and therefore remain partial with an unknown address. No
administrator privileges are requested.

## Documentation Impact

Update `PLATFORM-ADAPTERS.md`, `DATA-FLOW.md`, `DIRECTORY-STRUCTURE.md`,
`TEST-CASES.md`, `PLATFORM-TESTING.md`, `SECURITY.md`, and this work record.
Do not alter frozen requirements or accepted ADRs.

## Review Findings

Pending mandatory read-only review after the PR exists.

## Known Limitations

- W006 cannot retain IPv6 scope IDs; a scoped listener is represented with
  unknown local address and explicit partial status.
- This work provides listener discovery infrastructure only; no Tauri command
  or frontend port view consumes the provider yet.
- Native Windows evidence is not established by macOS host tests or
  cross-compilation; Windows hosted CI is the merge gate.
- No coordinator timeout/cancellation is added; provider calls remain
  synchronous and the refresh coordinator owns invocation policy later.
