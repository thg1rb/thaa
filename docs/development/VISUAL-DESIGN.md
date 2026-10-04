# Thaa Visual Design

W013.1 establishes a restrained technical visual language for the desktop
runtime inspector. It takes cues from technical HUD interfaces without copying
a specific reference or letting ornament compete with endpoint data.

## Interaction and state colors

| Token            | Value     | Use                                       |
| ---------------- | --------- | ----------------------------------------- |
| Background       | `#080E13` | Main graphite canvas                      |
| Surface          | `#101920` | Runtime cards and panels                  |
| Elevated surface | `#15232B` | Dialogs and raised controls               |
| Primary          | `#28E7D8` | User interaction, selection, hover, focus |
| Primary hover    | `#63F2E5` | Interactive hover                         |
| Danger           | `#FF707C` | Force Stop and destructive confirmation   |
| Warning          | `#F2BD67` | Partial inspection state                  |
| Success          | `#65D69A` | Positive state                            |
| Primary text     | `#E7F0F2` | Main content                              |
| Secondary text   | `#A5B7BD` | Supporting content                        |
| Muted text       | `#758991` | Secondary labels                          |
| Border           | `#24343D` | Quiet structure                           |
| Focus            | `#28E7D8` | High-contrast keyboard focus              |

Primary indicates an interactive element. It is not used as a general heading,
port, or decorative-border color. Danger, warning, and success remain separate
semantic colors and are also distinguished by text/shape, not color alone.

## Type, shape, and motion

Interface copy uses the system sans-serif stack. Monospace is limited to ports,
PIDs, and short technical values. Cards use subtle angular details, small
uppercase labels, restrained borders, and a quiet background grid. Interaction
transitions are short; the scan indicator stops animating when the user
requests reduced motion. No perpetual glitch or scanline effect is used.

Runtime cards prioritize process/application name and port, then owner/address
and binding, then optional metadata, then actions. Long paths are truncated in
the compact view. Process icons are decorative; visible text always identifies
the process. Missing native icons use Thaa's generic process-node mark, while
unresolved owners use a distinct outlined mark.

## User-action feedback

Transient user-action results use a shared Toast viewport. Success is green,
failure is danger red, warnings are amber, and informational updates use the
interactive primary accent. A successful process action is described as a
request; it is not described as an exited process until a later scan observes
that state. The viewport shows at most three notifications at once and keeps a
bounded FIFO queue. Notifications can be dismissed, expire after a readable
interval, and pause expiration while hovered or focused. Passive and periodic
refreshes do not create notifications. The viewport is anchored at the
bottom-right with the newest notification at the bottom. Both timeout and
manual dismissal transition through a short exit animation before the slot is
released and the next queued notification is promoted. A fallback timer removes
the item if the browser does not deliver an animation-end event; reduced-motion
preferences shorten the motion rather than changing the notification meaning.

## Loading placeholders

Initial runtime discovery uses three noninteractive Runtime Card Skeletons
with approximate card geometry. A single polite status announces the initial
scan; individual placeholder blocks are hidden from assistive technology. The
header count and observation time remain placeholders until a real snapshot is
available. Initial errors and empty results replace the Skeletons with their
normal states. A manual or background refresh keeps the last usable snapshot
visible and uses only the Refresh control's in-progress state.

Runtime snapshots do not wait for optional icon enrichment. For a known process
with an unresolved icon reference, a fixed-size icon Skeleton is shown while
the generation-scoped icon request is pending. A resolved native icon replaces
it; missing or failed icon data resolves to the Thaa fallback. Unknown owners
use their fallback immediately because no icon request is pending for them.
Skeletons use muted surfaces, a CSS-only subtle scan, and no animation when
reduced motion is requested.

## Window chrome and canvas

The macOS window uses Tauri's supported `Overlay` title-bar style with native
traffic lights and a separate top drag region. The content starts below the
traffic-light safe area; interactive controls are outside the drag region. The
native window, webview, `html`, `body`, and React root all use the graphite
canvas color. The document is the single vertical scroll owner, with vertical
overscroll suppressed while retaining a matching background at every layer.
Other platforms retain their native window decorations.

## Native Thaa branding

The canonical application and tray icon masters live in
`src-tauri/icons/source/`. Tauri-generated application assets and small tray
derivatives live in `src-tauri/icons/`. The native Rust tray composition uses
a 44×34 transparent monochrome template image on macOS and a 32×32 transparent
cyan image on Windows. The macOS asset has a 32px-high glyph; its aspect-matched
canvas renders close to the pinned tray adapter's 18pt native status-image
height. Windows sizing is independent and remains at 32×32. These are Thaa
application-branding assets and remain
separate from the per-process icons shown in runtime cards. Bundle/executable
icon metadata does not participate in process identity or action authorization.

The WebView root clips unintended horizontal overflow and suppresses horizontal
overscroll while retaining normal document vertical scrolling. App content and
notifications do not permit webpage-style text selection; text entry controls
restore it. Runtime Stop/Force and confirmation Cancel/Force pairs use equal
CSS grid columns so the longer action label determines both control widths.

## Frontend responsibilities

`App.tsx` composes shared providers and the runtime-inspector page. The
`features/runtime-inspector` feature owns its transport client, DTO/model types,
refresh and action hooks, and focused runtime-card components. Shared Toast
presentation lives in `shared/ui/toast`; Tauri command names and `invoke`
transport stay in the feature API client rather than React presentation
components. A development-only visual fixture can preview toasts and cards
without invoking native process actions.
