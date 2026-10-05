# Thaa — Local Runtime Inspector

Thaa is a desktop utility for understanding local listening TCP ports and their owning processes. The current First Usable Build provides a runtime list, refresh, local URL/copy actions, and identity-checked process actions supported by the active platform.

Thaa runs locally. Git repository/branch context, resource metrics, and runtime/framework detection remain future work.

## Run locally

See [Local Setup](docs/development/LOCAL-SETUP.md) for prerequisites, then:

```sh
pnpm install --frozen-lockfile
pnpm tauri dev
```

See [Development](docs/development/DEVELOPMENT.md) for quality commands and contributor workflow. Product scope and engineering guidance live in [docs](docs/README.md).
