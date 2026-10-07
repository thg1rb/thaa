# W017 — CPU / Memory / Uptime

## Status

Status: COMPLETE. Implementation merged to `develop` by PR #57. The final
reviewed source was `5d2de903864f1f19b4c676f696d59514fa88a6b3`; merge commit
and resulting `develop` were `9f5705d239ddf21d527e1d0a1fe924f27e644e5c`.

Requirement: FR-014 metrics subset (CPU, memory, uptime/start time). Parent
process/tree and project-oriented actions remain deferred.

## Objective and scope

Add current CPU, resident-memory, and process-uptime metadata to the Runtime
Inspector on macOS and Windows. These values are optional observations from a
runtime snapshot. They do not change process identity, action targets, Stop or
Force Stop authorization, process enumeration, or refresh cadence.

FR-014 is frozen at the requirement level. This work assigns only its metrics
subset; parent process/tree and project-oriented open/reveal actions remain
future P1 work. W018 remains process-tree work and is not part of W017.

## Metric semantics

- **CPU:** percent of total logical CPU capacity, represented internally in
  hundredths of a percent and displayed to one decimal place at most. The
  provider reports cumulative user plus kernel CPU time. The application
  derives utilization from two per-process observations using monotonic time
  and the host's available logical CPU count. The value is normalized to
  `0–100%` and capped at `100%`. The first observation, unavailable identity or
  counter, invalid interval, or unavailable CPU count is shown as `—`, never
  as a fabricated `0%`.
- **Memory:** resident bytes. macOS uses `ri_resident_size` from the public
  `proc_pid_rusage` API; Windows uses `WorkingSetSize` from
  `K32GetProcessMemoryInfo`. Both are physical-residency measures but retain
  their operating-system reporting differences. The domain and transport use
  bytes; the UI formats binary units (KiB, MiB, GiB, and larger). Zero is a
  valid value.
- **Uptime:** duration from the already observed process start time to the
  runtime snapshot observation time. Missing or future start times are
  unavailable. The transport uses milliseconds and the UI displays compact
  elapsed time.

## Architecture and lifecycle

Platform process providers collect the cumulative CPU counter and resident
bytes through read-only native APIs and attach a monotonic sample time to the
resource sample. The shared runtime scanner derives CPU utilization and uptime
into optional snapshot metrics. A bounded map keyed by PID plus the existing
process start time retains only processes in the latest accepted snapshot.
Sampling state is copied for a scan and committed only when that scan's
generation is accepted; superseded scans cannot perturb later results. Missing
metrics never fail a process or listener scan.

The transport adds nullable `resourceMetrics` values (`cpuPercent`,
`memoryBytes`, `uptimeMs`) to each runtime entry. They are not included in
`ProcessInfoDto` identity fields and are not inputs to process actions. The UI
shows CPU, Memory, and Uptime as secondary metadata and uses an em dash for
unavailable values.

## Platform and security behavior

- macOS uses `proc_pid_rusage(RUSAGE_INFO_V4)` through the existing isolated C
  bridge and links the system `libproc`. The existing `sysctl` start-time
  identity checks still bracket process inspection; exit or PID replacement
  remains a process-level disappearance rather than mixed metadata.
- Windows uses the provider's existing least-privilege process HANDLE for
  `GetProcessTimes` and `K32GetProcessMemoryInfo`; the handle is still RAII
  closed and no VM-read, debug, termination, or elevated rights are added.
- Access denial, unsupported values, and process races degrade only the
  affected resource fields. No subprocess, privilege escalation, persistent
  history, telemetry, or resource-value logging is introduced.
- CPU sample keys include PID and start time. Stale processes are removed on
  each accepted scan; a reused PID cannot inherit its prior CPU baseline.

## Acceptance criteria

| ID    | Criterion                                                                                                                                                        | Verification                                                  |
| ----- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------- |
| AC-01 | macOS and Windows providers return cumulative CPU time and resident bytes for an inspectable controlled process when the OS permits it.                          | Native controlled-process provider tests on both CI platforms |
| AC-02 | CPU is calculated from counter and monotonic-time deltas, normalized to total logical CPU capacity, capped at 100%, and unavailable on the first/invalid sample. | Deterministic calculation and sampler tests                   |
| AC-03 | CPU samples are associated with PID plus process start time and stale sample state is bounded to the current accepted snapshot.                                  | PID-reuse, disappearance, and superseded-scan tests           |
| AC-04 | Memory remains raw bytes in domain/IPC; zero, large, unavailable, and binary-unit display cases are handled.                                                     | DTO and frontend formatting tests                             |
| AC-05 | Uptime derives from observed start time, rejects future/missing times, and formats zero through multi-day durations.                                             | Deterministic domain and frontend tests                       |
| AC-06 | Resource metrics remain optional snapshot presentation data and do not alter listener visibility, process identity, Git Context, or process actions.             | Runtime composition, DTO, UI, and action regression tests     |
| AC-07 | Runtime Inspector displays the metrics as secondary accessible text and handles unavailable values without layout or state errors.                               | Frontend tests, responsive inspection, and read-only review   |
| AC-08 | macOS and Windows provider failures remain field-local, use no privilege escalation, and do not leak native handles or sensitive values.                         | Native tests, security review, and cross-platform CI          |

### Acceptance evidence

| ID    | Result | Evidence                                                                                                                                                                                                  |
| ----- | ------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| AC-01 | PASS   | Controlled-process provider tests passed locally on macOS and in macOS/Windows Native Validation on PR run `37576736850`; post-merge Native Validation passed on run `37577439695`.                       |
| AC-02 | PASS   | Deterministic sampler tests cover delta calculation, normalization, capping, first sample, invalid intervals, and unavailable inputs.                                                                     |
| AC-03 | PASS   | Tests cover PID reuse, stale sample removal, and superseded scans not committing a CPU sampling baseline. The latter test was added after review identified a missing assertion.                          |
| AC-04 | PASS   | DTO tests cover nullable/zero values; frontend formatting tests cover binary byte units and large values.                                                                                                 |
| AC-05 | PASS   | Domain tests cover missing/future start times; UI formatting tests cover zero through multi-day durations.                                                                                                |
| AC-06 | PASS   | Runtime composition, DTO, UI, and action-target regression tests passed; metrics remain separate from process identity and action authorization.                                                          |
| AC-07 | PASS   | Component tests cover secondary metric text and unavailable values; metadata styling wraps/truncates long content. Interactive visual testing at multiple window sizes was not performed.                 |
| AC-08 | PASS   | Provider tests and both platform CI jobs passed. The implementation adds no elevated rights; Windows process handles remain RAII-managed. Dedicated review found no remaining material security findings. |

“PASS” for AC-07 refers to automated UI and layout-rule coverage; it does not claim manual interactive visual validation.

## Non-goals

Process tree/parent metadata, project actions, per-thread or GPU metrics,
historical graphs, persistent sampling, sorting/filtering by metrics, a system
monitor, configurable refresh, and any release/version promotion are outside
W017.

## Validation and evidence

Validation ran against final reviewed PR head
`5d2de903864f1f19b4c676f696d59514fa88a6b3`. PR CI run `37576736850` passed
Shared Quality, macOS Native Validation, and Windows Native Validation. After
merge, all three checks passed again on run `37577439695`.

Local validation passed: `cargo test --manifest-path src-tauri/Cargo.toml
--all-targets` (95 Rust unit tests plus applicable native/provider/action,
contract, and runtime integration suites); strict Clippy; Rust formatting;
`pnpm format:check`; `pnpm lint`; `pnpm typecheck`; `pnpm test` (54 tests);
`pnpm build`; `pnpm docs:format:check`; `pnpm docs:check` (80 Markdown files);
macOS `pnpm tauri build --debug --no-bundle`; dependency audit; and
`git diff --check`.

Windows cross-target compilation was not available locally because the host
lacks `llvm-rc`; Windows Native Validation CI passed on the final PR head and
again after merge. `actionlint` was not run because it is not installed and no
workflow file changed. `cargo audit` completed for macOS arm64 and Windows
x64, reporting only the two previously documented allowed advisories:
RUSTSEC-2024-0370 (`proc-macro-error`, unmaintained) and RUSTSEC-2024-0429
(`glib` `VariantStrIter` unsoundness). No dependency was added.

The macOS host built the native app in debug/no-bundle mode. No manual
interactive runtime or visual test was performed on either platform; Windows
runtime/provider validation was performed by Windows Native Validation CI.
No standalone before/after scan benchmark was run, and no numeric performance
claim is made because NFR-006 defines no threshold.

The implementation adds no subprocesses or refresh loop. Metric work is
bounded by the number of distinct listener-owner processes already inspected:
one additional libproc query on macOS and CPU/memory native queries on the
existing Windows process handle. The CPU baseline retains at most one entry
per process in the latest accepted snapshot. No standalone before/after scan
benchmark was run, and no performance claim is made.

## Review and integration

PR #57 (`feat: add W017 CPU, memory, and uptime metrics`) targeted `develop`
and merged with the repository's merge-commit strategy. The dedicated
read-only review initially reported one medium finding: the superseded-scan
test did not assert that sampling state remained unchanged. The implementation
added that assertion in `superseded_scan_does_not_commit_cpu_sampling_baseline`;
the reviewer re-reviewed the final head and reported no remaining material
findings. The post-merge CI run passed on the resulting `develop` commit.

The release branch and release history were not changed. `main` remains
`3c1e9a53caf8ba048eaeeff3d77b26167b3140ff`; no version, tag, or release was
created or changed. W018 has not started.
