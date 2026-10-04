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
scales its full artwork to an 824×824 centered body with 100px transparent
margins, and crops tray sources to nontransparent alpha bounds before fitting
them into compact canvases. The macOS derivative is 44×44 with a proportional
32px-high glyph; the Windows derivative remains 32×32 with a 23px-high glyph.
The generator checks the expected app bounds and runs an asymmetric
top/bottom orientation fixture through drawing and PNG encode/decode before
writing outputs.

Tauri writes official application outputs into `src-tauri/icons/`, including
desktop application files and additional iOS/Android outputs. Commit the
generated files and retain the additional official CLI outputs; mobile targets
are not configured by this work item. Production derivatives live in
`src-tauri/icons/derived/`. Do not place generated files under `source/` or
replace these canonical masters. The Rust tray builder selects the appropriate
derived PNG by compile-time platform: macOS enables native template rendering
for its monochrome asset; Windows uses its cyan asset without template
semantics. The original tray masters remain unchanged.
