# Application Directory Structure

This records the actual source structure established in W005 and the
repository/tooling boundary confirmed in W005.1. A single Tauri application
conventionally keeps the JavaScript frontend project and its configuration at
the repository root while nesting the Rust/Cargo application under
`src-tauri/`. This is one application repository, not an accidental frontend /
backend split or a Cargo workspace. The layout matches the [official Tauri
project structure](https://v2.tauri.app/start/project-structure/).

```text
.
  package.json            # frontend scripts and dependencies
  pnpm-lock.yaml          # reproducible frontend dependency graph
  tsconfig*.json          # TypeScript project configuration
  vite.config.ts          # frontend build/dev configuration
  eslint.config.js        # frontend/build-config code-quality rules
  .prettierrc.json        # repository Prettier policy
  .prettierignore         # generated and vendored content exclusions
  .editorconfig           # editor defaults; formatter tools remain authoritative
src/
  App.tsx                 # application root and shared provider composition
  main.tsx                # React entry point and global style import
  styles/global.css       # design tokens and root/window canvas styles
  features/runtime-inspector/
    RuntimeInspectorPage.tsx # page composition and user-action wiring
    api/runtimeClient.ts  # typed Tauri DTO client and command names
    hooks/                # refresh, process-action, and entry-action state
    components/           # header, runtime list/cards, icons, dialogs
    model/types.ts        # frontend transport/state types
    runtime-inspector.css # feature presentation styles
  shared/ui/toast/        # bounded accessible notification system
  dev/                    # development-only synthetic visual fixture
  App.test.tsx            # runtime-inspector behavioral tests
  test/setup.ts           # Vitest matcher setup
src-tauri/
  Cargo.toml              # the application's Rust package
  Cargo.lock              # reproducible Rust dependency graph
  build.rs                # Tauri build and command permission manifest
  tauri.conf.json         # application identity, window, CSP, build settings
  capabilities/
    main-window.json      # runtime commands and scoped event-listen capability
  src/
    lib.rs                # Tauri composition root
    main.rs               # desktop entry point
    application/           # shared use-case orchestration
      mod.rs
      process_inspection.rs # per-PID inspection using ProcessProvider
      process_icons.rs      # bounded, snapshot-scoped icon presentation contract
      runtime_inspection.rs # listener/process snapshot and refresh coordinator
    domain/               # shared, platform-neutral domain values and rules
      mod.rs
      capabilities.rs
      metadata.rs
      network.rs
      port_provider.rs   # stable listening-port contract and query outcomes
      process.rs
      process_action.rs   # identity-bound target and shared action values
      process_controller.rs # platform-neutral action and capability ports
      process_provider.rs # read-only one-process inspection contract and errors
    platform/             # OS-specific adapters; conditionals stop here
      mod.rs
      macos/               # compiled only for macOS
        mod.rs
        lsof.rs            # bounded invocation helper and private byte parser
        port_provider.rs   # MacOSPortProvider implementing domain contract
        process_provider.rs # MacOSProcessProvider; lsof metadata normalization
        process_controller.rs # identity-bound SIGTERM/SIGKILL requests and capabilities
        process_identity.rs # shared SDK-backed KERN_PROC_PID identity query
        process_identity.c # SDK-defined start-time query and fixed-signal shim
        process_icon.rs   # AppKit icon wrapper
        process_icon.m    # public NSRunningApplication icon bridge
      windows/             # compiled only for Windows
        mod.rs
        port_provider.rs   # GetExtendedTcpTable adapter and checked row parsing
        process_native.rs  # shared RAII HANDLE, FILETIME/path/lifetime helpers
        process_provider.rs # documented Win32 metadata through shared helpers
        process_controller.rs # same-HANDLE identity-checked Force Stop
        process_icon.rs   # Shell/GDI icon extraction and owned resources
    commands/
      mod.rs
      runtime.rs          # runtime/action commands and transport DTOs
  tests/
    common/
      mod.rs
      port_provider_contract.rs # reusable native-provider assertions
      process_provider_contract.rs # reusable native process assertions
    port_provider_contract.rs   # deterministic public-contract tests
    macos_process_controller.rs # controlled-child SIGTERM/SIGKILL tests (macOS only)
    process_provider_contract.rs # deterministic process contract tests
    macos_port_provider.rs       # controlled native listener tests (macOS only)
    runtime_snapshot_native.rs # controlled listener + native process snapshot
  icons/source/           # canonical approved Thaa application and tray masters
  icons/                  # generated application icons and small native tray images
docs/
  architecture/           # architecture baseline and actual directory shape
  development/            # setup, workflow, and coding standards
  agent/                  # agent policy and project-scoped skill inventory
```

The application module owns shared orchestration and depends on domain
contracts; W011 process inspection deduplicates requested PIDs per invocation
and preserves each per-PID outcome. The domain module contains W006 shared
models and pure binding classification, plus the W007 `PortProvider` and W010
`ProcessProvider` contracts. The W010 process contract returns `ProcessInfo`
and keeps platform-wide capabilities separate. W008's listener `lsof` field
parser and W011.1's process metadata parsing stay private to the macOS adapter;
W009's Win32 types, bounded aligned buffers, and table parser stay private to
the Windows adapter. W011.2/W012.2's process HANDLE, Win32 errors, FILETIME
conversion, and UTF-16 buffer handling stay private to that adapter.
Inspection and action share the same internal RAII and identity-query helpers.
Rust integration tests under `src-tauri/tests/` exercise the public contract as an external crate
would. `tests/common/` assertions are reused by both native provider targets.

The root also owns `rust-toolchain.toml` because it pins the Rust toolchain
used by this repository's nested Tauri project. Frontend TypeScript/Vite/ESLint
configuration stays at the root so those tools resolve from `src/`; Rust
manifests and source stay together under `src-tauri/`. A Cargo workspace is
not needed for the current single Rust application. Future domain/application
modules belong under `src-tauri/src/` unless later evidence justifies a
workspace.

W013.1 adds `application/process_icons.rs` for the presentation-only icon port,
PNG bounds, and snapshot asset values. Platform implementations live in
`platform/macos/process_icon.rs` and `process_icon.m` (public AppKit bridge),
and `platform/windows/process_icon.rs` (Shell/GDI extraction with local RAII
resource owners). The runtime snapshot returns icon references; the separate
`get_runtime_process_icons` command resolves bounded assets on a blocking worker
for the still-current snapshot. Icons do not modify domain process identity or
action-target models.
