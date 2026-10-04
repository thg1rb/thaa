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
