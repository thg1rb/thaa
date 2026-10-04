# Thaa icon assets

These are the canonical, user-approved Thaa branding sources. Keep generated
application and tray assets outside this directory.

## Files

- `thaa-app-icon.png` — 1024×1024 RGBA application icon master.
- `thaa-tray-template-macos.png` — 512×512 RGBA monochrome foreground on transparency; use as a macOS template image.
- `thaa-tray-windows.png` — 512×512 RGBA cyan foreground on transparency for Windows tray use.

Generate application assets from the application master with:

```sh
pnpm tauri icon src-tauri/icons/source/thaa-app-icon.png
```

Tauri writes generated assets into `src-tauri/icons/`, including desktop
application files and additional official iOS/Android outputs. Commit the
generated files and retain the additional CLI outputs; mobile targets are not
configured by this work item. Do not place generated files under `source/` or
replace these masters. The Rust tray builder uses
small derived PNGs (`thaa-tray-macos.png`, 36×36, and
`thaa-tray-windows.png`, 32×32) in `src-tauri/icons/`. macOS enables native
template rendering for its monochrome asset; Windows uses its cyan asset
without template semantics. The original tray masters remain unchanged.
