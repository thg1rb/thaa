# Local Setup

## Supported bootstrap environment

The application shell uses Tauri 2, Rust, React, TypeScript, Vite, and pnpm. Versions are constrained by the repository manifests:

- Node.js 24.x (`.node-version`; `package.json` requires `>=24 <25`).
- pnpm 10.29.2 (`packageManager` in `package.json`).
- Rust 1.99.0 with `rustfmt` and `clippy` (`rust-toolchain.toml`; rustup installs the toolchain when first used).
- Tauri CLI/API v2, resolved and locked in the package and Cargo lockfiles.

Install Node.js from [nodejs.org](https://nodejs.org/) and pnpm using its [official installation guidance](https://pnpm.io/installation). Install Rust through [rustup](https://rustup.rs/). Use the system's native development tools required by [Tauri's prerequisites](https://v2.tauri.app/start/prerequisites/).

## Platform prerequisites

- **macOS:** Xcode Command Line Tools and the WebKit development libraries provided by macOS. W005 was built on macOS 27 arm64 with Xcode installed.
- **Windows:** Windows 11 is the planned minimum. Install Microsoft C++ Build Tools with the Desktop development with C++ workload and WebView2 Runtime as described by Tauri's current Windows prerequisites. W005 has not yet been built or launched on Windows.

Check the upstream prerequisite pages before setup; platform tooling can change. Do not run Thaa with administrator/root privileges for normal development.

## Install and launch

From the repository root:

```sh
pnpm install --frozen-lockfile
pnpm tauri dev
```

The first Rust build can take several minutes while Cargo compiles dependencies. No signing credentials or elevated permissions are required for development.
