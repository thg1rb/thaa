# Thaa — Local Runtime Inspector

Thaa is a desktop utility in active development for understanding local ports, their owning processes, and development runtimes. The Tauri 2 application shell is now bootstrapped; runtime inspection functionality has not yet been implemented.

The current development shell displays application metadata through a local Tauri IPC smoke path. It does not inspect ports, processes, projects, or runtimes.

## Run locally

See [Local Setup](docs/development/LOCAL-SETUP.md) for prerequisites, then:

```sh
pnpm install --frozen-lockfile
pnpm tauri dev
```

See [Development](docs/development/DEVELOPMENT.md) for quality commands and contributor workflow. Product scope and engineering guidance live in [docs](docs/README.md).
