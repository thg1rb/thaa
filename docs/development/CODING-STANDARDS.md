# Coding Standards

These standards keep Thaa's code clear, testable, and consistent across macOS
and Windows. The accepted requirements, ADRs, architecture, security baseline,
and testing strategy take precedence over examples in external Project Skills.

## General

- Use descriptive names and small, cohesive modules with clear responsibilities.
- Handle errors explicitly; do not silently discard failures.
- Avoid duplicated logic and abstractions without a current use case.
- Document non-obvious decisions and safety invariants close to the code.
- Treat operating-system process metadata and paths as untrusted input.

## Rust

- Prefer idiomatic ownership, borrowing, enums, and explicit `Result` handling.
- Avoid `unwrap` and `expect` in production paths; tests may use them when the
  failure is the intended assertion.
- Keep unsafe code minimal and document each unsafe block's safety invariant.
- Keep platform-specific behavior in platform adapters, not shared domain or
  application logic.
- Keep `cargo clippy` clean; resolve warnings rather than suppressing them
  broadly.

## TypeScript

- Keep strict TypeScript checks enabled and use explicit types at system and
  Rust/TypeScript boundaries.
- Prefer `unknown`, type guards, and discriminated unions over `any` or unsafe
  casts.
- Keep errors explicit and represent unavailable data without fabricated
  values.
- Use advanced type utilities only when they improve a real interface; avoid
  clever types that obscure ordinary data.

## React

- Use functional components and keep rendering free of side effects.
- Keep presentation separate from system inspection and Tauri IPC behavior.
- Prefer local state until a demonstrated shared-state need exists.
- Use semantic HTML, keyboard-accessible interactions, visible focus, and
  accessible state/error announcements.
- Test user-observable behavior rather than component implementation details.

## Security and Testing

- Follow the engineering rules in [Security](../security/SECURITY.md) and the
  threat-specific controls in [the threat model](../security/THREAT-MODEL.md).
- Follow the layers, provider contracts, isolation, and evidence rules in [the
  testing strategy](../testing/TEST-STRATEGY.md).
- Do not interpolate untrusted values into shell commands, executable
  arguments, URLs, or filesystem actions.
- Do not log command arguments or sensitive local paths by default.

## Formatting and Quality Commands

Prettier owns formatting for TypeScript, TSX, JavaScript, CSS, HTML, JSON, and
the maintained Markdown files selected by the package scripts. `rustfmt` owns
Rust formatting. TOML files retain conventional Cargo formatting; no second
formatter is configured for them.

```sh
pnpm format
pnpm format:check
pnpm lint
pnpm typecheck
pnpm test
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

See [Development](DEVELOPMENT.md) for command purposes and [Agent Workflow](../agent/AGENT-WORKFLOW.md) for Project Skill usage.
