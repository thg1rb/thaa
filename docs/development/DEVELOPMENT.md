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
| `pnpm docs:check`                                                                | Check local Markdown link targets                             |

Rust commands can also be run from `src-tauri/` without the manifest argument. Use `pnpm tauri build --no-bundle` for a host desktop build; packaging is intentionally disabled during W005.

GitHub Actions runs the shared frontend/docs gates and native macOS/Windows
Rust checks and Tauri builds. See [CI](CI.md) for runner versions, action
security, dependency auditing, and platform evidence. CI complements the
read-only PR review; branch protection is not yet configured.

## Tests and validation

Vitest, jsdom, and Testing Library cover user-visible bootstrap states; Tauri's official mock IPC API supplies controlled responses. Rust's built-in test runner covers domain/provider behavior. W008 adds deterministic `lsof` parser tests and controlled macOS native listener integration tests. Windows native provider behavior remains unimplemented and unvalidated. See [test strategy](../testing/TEST-STRATEGY.md) and [platform testing](../testing/PLATFORM-TESTING.md) for current evidence and remaining expectations.

For code PRs, run formatting, lint, typecheck, Rust Clippy/tests, frontend tests/build, dependency review, and a native Tauri build on the available platform. Provider work also requires the relevant real OS integration tests. Report unavailable-platform checks as `NOT RUN`; do not infer parity from compilation on the other OS.

## Source formatting

See [Coding Standards](CODING-STANDARDS.md) for formatting ownership and
language guidance. Prettier formats frontend sources and the curated maintained
documentation/configuration paths in the package scripts; `rustfmt` formats
Rust. The pinned Rust toolchain includes rustfmt and Clippy. Avoid running
broad auto-format commands over frozen requirement documents or copied Project
Skills. `.prettierrc.json`, `.prettierignore`, and `.editorconfig` define the
repository defaults. ESLint checks code quality; it does not own formatting.
