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
  App.tsx                 # bootstrap UI and typed IPC state
  App.css                 # minimal shell styling
  main.tsx                # React entry point
  App.test.tsx            # observable shell-state tests
  test/setup.ts           # Vitest matcher setup
src-tauri/
  Cargo.toml              # the application's Rust package
  Cargo.lock              # reproducible Rust dependency graph
  build.rs                # Tauri build and command permission manifest
  tauri.conf.json         # application identity, window, CSP, build settings
  capabilities/
    main-window.json      # narrow app-info command capability
  src/
    lib.rs                # Tauri composition root
    main.rs               # desktop entry point
    domain/               # shared, platform-neutral domain values and rules
      mod.rs
      capabilities.rs
      metadata.rs
      network.rs
      port_provider.rs   # stable listening-port contract and query outcomes
      process.rs
    platform/             # OS-specific adapters; conditionals stop here
      mod.rs
      macos/               # compiled only for macOS
        mod.rs
        lsof.rs            # bounded invocation helper and private byte parser
        port_provider.rs   # MacOSPortProvider implementing domain contract
      windows/             # compiled only for Windows
        mod.rs
        port_provider.rs   # GetExtendedTcpTable adapter and checked row parsing
    commands/
      mod.rs
      app_info.rs         # thin command and transport DTO
  tests/
    common/
      mod.rs
      port_provider_contract.rs # reusable native-provider assertions
    port_provider_contract.rs   # deterministic public-contract tests
    macos_port_provider.rs       # controlled native listener tests (macOS only)
  icons/icon.png          # minimal RGBA application icon required by Tauri context
docs/
  architecture/           # architecture baseline and actual directory shape
  development/            # setup, workflow, and coding standards
  agent/                  # agent policy and project-scoped skill inventory
```

The domain module contains W006 shared models and pure binding classification,
plus the W007 stable port-provider contract. W008's `lsof` field parser stays
private to the macOS adapter; W009's Win32 types, bounded aligned buffers, and
table parser stay private to the Windows adapter. Rust integration tests
under `src-tauri/tests/` exercise the public contract as an external crate
would. `tests/common/` assertions are reused by both native provider targets.

The root also owns `rust-toolchain.toml` because it pins the Rust toolchain
used by this repository's nested Tauri project. Frontend TypeScript/Vite/ESLint
configuration stays at the root so those tools resolve from `src/`; Rust
manifests and source stay together under `src-tauri/`. A Cargo workspace is
not needed for the current single Rust application. Future domain/application
modules belong under `src-tauri/src/` unless later evidence justifies a
workspace.
