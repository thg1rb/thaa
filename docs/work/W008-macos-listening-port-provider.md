# W008 — macOS Listening Port Provider

## Status

Ready for review — implementation and native macOS validation are complete;
PR review and integration remain outstanding.

## Objective

Implement the W007 `PortProvider` contract on macOS and demonstrate live
discovery of controlled TCP listeners without exposing macOS or `lsof`
semantics beyond the platform adapter.

## Scope

- Add `MacOSPortProvider` under `src-tauri/src/platform/macos/`.
- Invoke the system `lsof` directly with fixed arguments and bounded execution
  and captured output.
- Parse NUL-delimited machine-readable fields and normalize records to W006
  listener values.
- Add deterministic parser/provider tests and controlled native macOS
  integration tests.
- Update architecture, testing, and W008 work documentation with actual host
  evidence and limitations.

## Out of Scope

Windows provider, process inspection/control, Tauri port command, frontend port
UI, refresh coordinator, polling, `sudo`, shell execution, native FFI, and
changes to W006/W007 contracts or frozen requirements.

## Requirements

Provides macOS implementation evidence for the listener-discovery foundations
of FR-001/FR-002 and normalized address semantics in FR-003. This does not
complete cross-platform listener requirements: Windows remains unimplemented
and unvalidated. The frozen baseline remains `THAA-REQ-0.1`; no requirement is
changed or claimed fully implemented cross-platform.

## Dependencies

W007 `PortProvider` and `PortScanResult`, W006 `NetworkListener` and
`ProcessId`, and W003 platform isolation are integrated. W004 security and
testing baselines govern command execution, output handling, and native test
isolation.

## Design Notes

The host was macOS 27.0 arm64 with `/usr/sbin/lsof` 4.91. The installed binary
supports `-F0` NUL-delimited fields, `-nP`, and `-iTCP -sTCP:LISTEN`; `-a` is
required to AND the selectors. The selected fields are `p`, `f`, `t`, `P`, and
`n`. An actual unused-port search returned exit 1 with empty stdout/stderr;
W008 maps this observed no-match case to a complete empty result. `-Q` is not
supported by this installed version and is not used.

Invocation is a single direct `Command` call to `/usr/sbin/lsof`; the provider
clears inherited environment variables, sets `LC_ALL=C`, and passes fixed
arguments. It never invokes a shell or escalates privileges. Runtime is bounded
to 30 seconds and combined stdout/stderr capture to 16 MiB. The reader/parser
is byte-oriented; required textual fields are decoded narrowly. Unknown fields
are ignored, but malformed process/file association or required listener
fields fail the scan rather than silently dropping records.

IPv4 and bracketed IPv6 numeric endpoints normalize to `IpAddr`; `*` is
interpreted using the reported address family, retaining separate IPv4 and
IPv6 wildcard records. Scoped IPv6 cannot be represented in the current W006
domain model; such records are retained with unknown address and the scan is
marked partial/Unsupported. A nonzero exit with valid rows yields partial
results; permission-denial diagnostics map to the stable PermissionDenied
category. Raw stdout/stderr and native error details are not exposed.

## Acceptance Criteria

- [x] `MacOSPortProvider` implements the W007 contract and is isolated under
      the macOS platform module.
- [x] Only TCP listeners are selected; numeric IPv4/IPv6, ports, and reported
      PID ownership are normalized.
- [x] Parsing uses selected NUL-delimited fields, not human-readable columns.
- [x] Execution is direct, static-argument, non-elevated, single-scan, and
      bounded by time/output limits.
- [x] Empty, partial, and failed query semantics remain distinct.
- [x] Deterministic parser, error-mapping, timeout, and output-bound tests pass.
- [x] Controlled IPv4 loopback discovery, PID mapping, post-close absence,
      IPv4 wildcard, IPv6 loopback, and IPv6 wildcard integration tests pass on
      the host macOS environment.
- [x] Architecture and platform-testing documentation distinguish macOS
      evidence from Windows implementation/support.
- [ ] Dedicated read-only PR review completed and findings resolved.
- [ ] PR merged into `develop` and post-merge checks completed.

## Validation

Host environment: macOS 27.0 arm64, `/usr/sbin/lsof` 4.91. Native tests use
test-owned ephemeral listeners and assert only their own endpoint; they do not
stop or alter discovered processes. Detailed command results will be recorded
in the PR/final report. Windows validation is NOT RUN. `cargo audit` is NOT RUN
if unavailable. No application UI or product IPC is changed.

## Security Considerations

System-derived output is untrusted. Fixed argument-vector invocation prevents
shell interpolation; executable resolution uses the observed system path, not
a writable search location. Output and duration are bounded; no `sudo`, raw
diagnostic logging, URL handling, Tauri permission, or native unsafe code is
introduced. Existing user permissions determine which listeners `lsof` can
observe. Scoped-address degradation is explicit and does not imply external
reachability.

## Platform Considerations

The provider is compiled only under the macOS platform boundary. Native
integration evidence covers the current host's IPv4/IPv6 loopback and wildcard
listener forms. `/usr/sbin/lsof` 4.91 lacks `-Q`; empty-search exit semantics
are handled based on observed empty status-1 output. Windows remains
Not Implemented / NOT RUN and must receive its own W009 implementation and
native validation.

## Documentation Impact

Update `docs/architecture/PLATFORM-ADAPTERS.md`,
`docs/architecture/DIRECTORY-STRUCTURE.md`, `docs/testing/TEST-CASES.md`,
`docs/testing/PLATFORM-TESTING.md`, and this work record. W006/W007 domain and
contract docs, accepted ADRs, and `THAA-REQ-0.1` remain unchanged.

## Review Findings

The dedicated read-only review reported one Low finding: the post-close
assertion required a globally Complete scan, so an unrelated scoped IPv6 row
could make the controlled IPv4 test flaky. The main Agent changed the targeted
absence assertion to accept `Complete` or only the known
`Partial(Unsupported)` scoped-address limitation while still failing on other
partial causes. Re-review is pending.

## Known Limitations

- The fixed `/usr/sbin/lsof` system path and field behavior were validated on
  the current macOS host; other supported macOS versions still need runner
  evidence.
- `lsof` results are limited by user permissions and may not expose every
  protected process/socket. No automatic privilege escalation is attempted.
- Scoped IPv6 addresses lose the scope identifier at the current domain
  boundary and therefore produce an explicit partial result.
- The synchronous provider bounds its subprocess call, but coordinator-level
  cancellation remains future application work.
- Windows provider and cross-platform parity are not implemented.
