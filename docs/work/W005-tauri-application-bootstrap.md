# W005 — Tauri Application Bootstrap

## Status

Done — merged into `develop` by [PR #9](https://github.com/thg1rb/thaa/pull/9) at merge commit `179f93c`.

## Objective

Establish the minimum functioning Tauri 2, Rust, React, and TypeScript application shell and verify a safe frontend-to-Rust smoke interaction without implementing runtime inspection features.

## Scope

- Bootstrap the Tauri 2 / React / TypeScript shell and Rust composition root.
- Add one typed, side-effect-free application-information IPC command.
- Establish strict formatting, lint, typecheck, and focused test commands.
- Document verified local setup, development workflows, actual source structure, dependency/tool choices, and platform limitations.

## Out of Scope

Listener discovery, process inspection or control, project/Git/runtime detection, tray behavior, polling, persistence, telemetry, shell/native inspection plugins, signing, installer/release work, CI workflows, and requirement/ADR changes.

## Requirements

Supports bootstrap prerequisites for NFR-001, NFR-008, NFR-009, and NFR-010, and PR-001 through local-only operation, explicit boundaries, pinned dependency management, and cross-platform portable source. It implements no frozen functional requirement and does not change `THAA-REQ-0.1`.

## Dependencies

W001–W004 are Done and integrated into `develop`; ADR-001 through ADR-005 and the W003/W004 architecture, security, and testing baselines apply.

## Design Notes

Use the official Tauri 2 React/TypeScript scaffold as a reference/generated baseline without overwriting repository governance. Keep the Rust surface to the composition root and a thin `get_app_info` command; do not create empty future architecture modules. Use pnpm 10.29.2 with one lockfile, Rust 1.99.0, and Tauri 2.12.0-era dependencies resolved and locked during implementation. The bundle identifier is `io.github.thg1rb.thaa`; Windows 11 is the selected minimum validation target. The public repository has no project license; W005 will not choose or add one.

## Acceptance Criteria

- [x] Tauri 2, Rust backend, and React/TypeScript frontend build on the available macOS host; app identity and structure follow ADR-001/002.
- [x] The UI calls `get_app_info` through real Tauri IPC and renders the typed response; loading and safe error states are covered by behavior tests.
- [x] Rust serialization and frontend tests verify the smoke DTO contract without fake product/runtime data.
- [x] Tauri command access, CSP, permissions, and dependencies follow W004 least-privilege and local-first rules; no shell, remote content, telemetry, or system inspection is added.
- [x] Formatting, Clippy, Rust tests, frontend format/lint/typecheck/tests/build, host Tauri build, runtime smoke, docs checks, and dependency checks are recorded accurately.
- [x] Setup, development, testing, README, docs navigation, and actual directory structure are documented; generated build artifacts are ignored.
- [x] macOS results reflect actual host validation; Windows 11 remains explicitly `NOT RUN` because no Windows runner is available.
- [x] Frozen requirements and accepted ADRs remain unchanged; no P0 product capability is claimed implemented.

## Validation

PASS — frontend format/lint/typecheck, 3 Vitest behavior tests, production build, Rust format, Clippy with warnings denied, Rust serialization test, macOS Tauri release build with `--no-bundle`, `pnpm audit`, documentation/link/navigation checks, `git diff --check`, frozen requirement/ADR integrity, PR review, and integration. The Tauri dev application launched; temporary fixed-string diagnostic output confirmed the frontend reached the real `get_app_info` Rust command twice. That diagnostic was removed and is not in the source. BLOCKED — visual inspection of the native window, because the computer-use review denied access; frontend behavior tests independently verify loading/success/error rendering. `cargo audit` exits 0 with two target-specific warnings (`glib` unsound iterator, `proc-macro-error` unmaintained) from Tauri's Linux GTK dependency graph; target trees confirm both are absent from macOS and Windows. Windows 11 build/runtime is NOT RUN because no Windows runner is available. Post-merge frontend build/tests, Rust tests/Clippy, and diff integrity passed on `develop`.

## Security Considerations

The command returns only public application metadata and has no system side effects. Expose only the required app command to the main window; grant no plugin/system permissions. Do not load remote resources, execute subprocesses, collect local runtime data, or log sensitive values. Dependency license inventory cannot establish project-license compatibility until the owner selects a project license.

## Platform Considerations

The current host is macOS 27 arm64 with Xcode installed. Rust 1.99.0 was installed user-locally through official rustup without PATH profile changes. Node 24.13.1 and pnpm 10.29.2 are available. Windows 11 native build/runtime checks require a separate Windows environment and must not be inferred from macOS results.

## Documentation Impact

Update README and contributor navigation; add local setup, development commands, and actual directory-structure documentation; record selected frontend test tools in the testing strategy; add this work item. Do not modify frozen requirements, ADRs, or the established security architecture.

## Review Findings

The PR #9 reviewer reported one Low evidence-record finding, resolved by commit `e399ffd`; read-only re-review found no remaining findings.

## Known Limitations

No product inspection functionality is part of W005. Visual inspection of the native window was blocked by computer-use access review; real command invocation was verified from the running Tauri dev app. Windows execution is not available on the current host. The public repository has no declared license, so direct dependency licenses were inventoried but not assessed against a project license. CI and release signing remain future work.
