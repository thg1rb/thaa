# Application Directory Structure

This records the actual source structure established in W005. Future architecture areas are added when a work item introduces real content; no empty provider/domain trees are created.

```text
src/
  App.tsx                 # bootstrap UI and typed IPC state
  App.css                 # minimal shell styling
  main.tsx                # React entry point
  App.test.tsx            # observable shell-state tests
  test/setup.ts           # Vitest matcher setup
src-tauri/
  icons/icon.png          # minimal RGBA application icon required by Tauri context
  build.rs                # Tauri build and command permission manifest
  tauri.conf.json         # application identity, window, CSP, build settings
  capabilities/
    main-window.json      # narrow app-info command capability
  src/
    lib.rs                # Tauri composition root
    main.rs               # desktop entry point
    commands/
      mod.rs
      app_info.rs         # thin command and transport DTO
```

The application, domain, provider, and platform adapter modules described by the architecture baseline will be introduced with their corresponding implementation work, not scaffolded empty during W005.
