# Thaa icon assets

These are the canonical, user-approved Thaa branding sources. Keep generated
application and tray assets outside this directory.

## Files

- `thaa-app-icon.png` — 1024×1024 RGBA application icon master.
- `thaa-tray-template-macos.png` — 512×512 RGBA monochrome foreground on transparency; use as a macOS template image.
- `thaa-tray-windows.png` — 512×512 RGBA cyan foreground on transparency for Windows tray use.

First generate deterministic production-sized derivatives from these masters:

```sh
swift scripts/generate-branding-assets.swift
pnpm tauri icon src-tauri/icons/derived/thaa-app-icon-production.png
```

`scripts/generate-branding-assets.swift` preserves the 1024×1024 app master,
scales its full artwork to 84% occupancy (860×860, centered with 82px
transparent margins), and crops tray sources to nontransparent alpha bounds
before fitting them to compact canvases. The macOS derivative is 36×36 with a
26px-high glyph; the Windows derivative is 32×32 with a 23px-high glyph.

Tauri writes official application outputs into `src-tauri/icons/`, including
desktop application files and additional iOS/Android outputs. Commit the
generated files and retain the additional official CLI outputs; mobile targets
are not configured by this work item. Production derivatives live in
`src-tauri/icons/derived/`. Do not place generated files under `source/` or
replace these canonical masters. The Rust tray builder selects the appropriate
derived PNG by compile-time platform: macOS enables native template rendering
for its monochrome asset; Windows uses its cyan asset without template
semantics. The original tray masters remain unchanged.
