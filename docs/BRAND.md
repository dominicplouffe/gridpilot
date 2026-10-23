# GridPilot brand

<p align="center"><img src="../assets/brand/gridpilot-wordmark.svg" width="420" alt="GridPilot"></p>

## Idea

**You pilot windows onto a grid.** The mark is a 3×3 grid in which a window
(Beacon amber) has been placed on a 2×2 selection, and a heading arrow
(Horizon cyan) in the last cell is steering it there. It reads at 16 px as
"grid + one highlighted block + pointer", and the aviation theme gives the
palette, names and tone.

## Palette: "Cockpit"

Dark instrument panel, amber annunciators, a cyan horizon line.

| Token | Name | Dark (default) | Light ("Daylight") | Use |
|---|---|---|---|---|
| `background` | Cockpit | `#0B1120` | `#F5F7FB` | App backgrounds, overlay scrim |
| `surface` | Fuselage | `#141C2F` | `#FFFFFF` | Panels, cards, the grid panel |
| `line` | Rivet | `#26324D` | `#D5DBE7` | Grid cells, dividers, inactive borders |
| `accent` | Beacon | `#FFB020` | `#C97A00` | Selection, active border, brand word "Pilot" |
| `accent_alt` | Horizon | `#3EC6FF` | `#0284C7` | Pointer, selection outline, links |
| `text` | Contrail | `#E7ECF5` | `#0B1120` | Primary text |
| `muted` | Altimeter | `#8893AB` | `#5A6478` | Secondary text, "Grid" in the wordmark |
| `danger` | Mayday | `#F2545B` | `#D62F3A` | Errors, destructive actions |
| — | Runway | `#34D399` | `#059669` | Success states (docs/site only) |

Rules of thumb:

* Amber is for **the thing being placed** (the selection, the focused
  window). Use it sparingly; one amber element per view.
* Cyan is for **guidance** (pointer, outlines, links), never large fills.
* Text never sits on amber except dark `#0B1120` text.

The palette ships as the `cockpit` and `daylight` themes in
`crates/gridpilot-core/src/theme.rs`. Omarchy-style community themes
(`tokyo-night`, `catppuccin`, `gruvbox`, `nord`) are built in too, and users
can add `~/.config/gridpilot/themes/<name>.toml`.

## Logo files

| File | Use |
|---|---|
| `assets/brand/gridpilot-icon.svg` / `gridpilot-icon-1024.png` | App icon (macOS squircle-ish, 1024 px). |
| `assets/brand/gridpilot-mark.svg` | Mark without background, for docs and the website. |
| `assets/brand/gridpilot-wordmark.svg` | Mark + "GridPilot" for headers. |
| `assets/brand/gridpilot-menubar-template.svg` | 18 px monochrome **template** image for the menu bar (macOS tints it). |

Keep clear space of one grid cell around the mark. Don't recolour the
selection anything but Beacon, don't rotate the arrow (it always points
up-left, toward the selection), and don't add a fourth row or column.

## Typography

* Headings and wordmark: **Space Grotesk** 700 (fallback Inter, SF Pro Display).
* UI: the system font (SF Pro on macOS) — GridPilot's own UI should feel native.
* Code, key chords, cheat sheet: **JetBrains Mono** or SF Mono.

Key chords are written lowercase with `+`: `mod+shift+left`. In prose, `mod`
is always code-formatted.

## Voice

Calm, precise, a little bit of flight deck. Short imperative labels
("Left half", "Pop window out", "Move window to workspace 3"). Avoid jargon
in UI strings; aviation words are for names and the website, not error
messages.

Taglines:

* *Take the controls of your windows.*
* *Divvy's grid. Omarchy's tiling. One cockpit.*
* *Draw it. Snap it. Fly.*
