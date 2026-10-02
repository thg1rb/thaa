# Development

Use Node.js 24.x, pnpm 10.29.2, and Rust 1.99.0 as pinned by repository configuration. Install dependencies with `pnpm install --frozen-lockfile`.

## Common commands

| Command                                                                          | Purpose                                                       |
| -------------------------------------------------------------------------------- | ------------------------------------------------------------- |
| `pnpm dev`                                                                       | Start the Vite frontend dev server                            |
| `pnpm tauri dev`                                                                 | Launch the desktop application with the frontend              |
| `pnpm build`                                                                     | Typecheck and build the frontend                              |
| `pnpm tauri build --no-bundle`                                                   | Build the host application without creating installers        |
| `pnpm format` / `pnpm format:check`                                              | Apply/check Prettier formatting on maintained source and docs |
| `pnpm lint`                                                                      | Run ESLint on frontend and build configuration                |
| `pnpm typecheck`                                                                 | Run TypeScript project checks                                 |
| `pnpm test`                                                                      | Run frontend behavior tests with Vitest                       |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check`                         | Check Rust formatting                                         |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | Run Rust lint checks                                          |
| `cargo test --manifest-path src-tauri/Cargo.toml`                                | Run Rust tests                                                |

Rust commands can also be run from `src-tauri/` without the manifest argument. Use `pnpm tauri build --no-bundle` for a host desktop build; packaging is intentionally disabled during W005.

## Tests and validation

Vitest, jsdom, and Testing Library cover user-visible bootstrap states; Tauri's official mock IPC API supplies controlled responses. Rust's built-in test runner checks serialized DTO shape. This does not test OS inspection: no provider or product feature exists yet. See [test strategy](../testing/TEST-STRATEGY.md) for future contracts and native platform evidence expectations.

Before a W005-equivalent code PR, run formatting, lint, typecheck, Rust Clippy/tests, frontend tests/build, dependency review, and a native Tauri build on the available platform. Report unavailable-platform checks as `NOT RUN`; do not infer parity from compilation on the other OS.

## Source formatting

Prettier formats the frontend and selected maintained documentation; rustfmt formats Rust. The pinned Rust toolchain includes rustfmt and Clippy. Avoid running broad auto-format commands over frozen requirement documents.
