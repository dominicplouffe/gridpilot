# Research: Divvy, Omarchy, and what it takes to combine them on macOS

This document records how the two products GridPilot draws from work, what
macOS lets a window manager do, and how each behaviour maps onto GridPilot's
design. It is the "why" behind [ARCHITECTURE.md](ARCHITECTURE.md).

Sources are listed at the end. Where a detail could not be confirmed from a
primary source it is marked *(unconfirmed)*.

---

## 1. Divvy (Mizage)

Divvy is a commercial window manager for macOS and Windows. Its whole model
fits in one sentence: **draw a rectangle on a grid and the front window
snaps to it.**

### 1.1 Interaction model

| Aspect | Behaviour |
|---|---|
| Activation | Click the menu bar icon, or press a global hotkey. |
| Target | The front-most window at the moment Divvy is activated. |
| Panel | A small floating panel with a grid of cells appears **on every screen**, so a window can be sent to any display, even one it isn't on. |
| Selection | Press on a cell, drag across cells, release: the window moves and resizes to exactly the area covered. |
| Dismissal | The panel closes after one resize, unless *Keep panel open until dismissed* is on. |
| Subdivide | Pressing ⌥ Option subdivides the grid for finer placement (default 6×6 → 12×12); pressing it again reverts. |
| Grid size | Rows and columns can be added up to 10×10 (20×20 subdivided) and removed down to 1×1. |
| Shortcuts | Unlimited named shortcuts. A shortcut is a saved grid selection plus a key combination. |
| Shortcut scope | **Global** shortcuts work at any time in any app; **local** shortcuts only work while the Divvy panel is open. |
| Multi-display | *Cycle screens when using shortcuts*: pressing the same shortcut again moves the window to the next screen. For local shortcuts this needs *Keep panel open* too (documented for Windows; the Mac option has the same name). |
| Margins | Space around the screen edges and between windows, in pixels, with a live preview *(documented in the Windows guide; the Mac app has the equivalent)*. |
| Footprint | Menu bar utility, no Dock icon, optional launch at login. |

### 1.2 How it works underneath

* **Geometry**: each display's *visible* frame (minus menu bar and Dock) is
  divided into `cols × rows` cells. A selection `(col, row, w, h)` maps to a
  rectangle: `x = left + col × cellW`, `w = w × cellW`, and the same for `y`.
  Margins inset the screen and leave a gap between adjacent selections.
* **Moving windows** on macOS is only possible through the **Accessibility
  API**: get the app's `AXUIElement`, read `AXFocusedWindow`, then set
  `AXPosition` and `AXSize`. That is why Divvy (and every tool like it) asks
  for the Accessibility permission. On Windows the same step is
  `SetWindowPos`.
* **Hotkeys**: global key combinations registered with the system so the
  front app never sees them (on macOS, Carbon `RegisterEventHotKey`).
* **State**: almost none. Divvy does not track windows; it acts on one
  window at a time and forgets it. This is its strength (predictable, cannot
  "fight" the user) and its limit (no workspaces, no auto-arrangement).

### 1.3 What GridPilot keeps

Everything above, as the `grid` feature:

* `GridSelection` stores the grid it was made on (`0,0 3x4 / 6x4`), so a
  shortcut keeps meaning "left half" when the grid size changes.
* `GridSession` is the overlay's state machine: drag, arrows, shift+arrows
  to extend, ⌥ to subdivide, `+`/`-` to change size, return/escape. It lives
  in the platform-neutral core and is unit-tested; the AppKit panel only draws
  it and forwards input as commands.
* Shortcuts have `global = true|false` exactly like Divvy.
* `grid.cycle_displays` implements "press again for the next screen".
* `grid.screen_margin` and `grid.window_margin` mirror Divvy's margins.
* The default layout is `floating`, which *is* Divvy: GridPilot does not
  rearrange anything until you ask it to.

---

## 2. Omarchy (Basecamp / DHH) on Hyprland

Omarchy is an opinionated Arch Linux setup whose window management is
[Hyprland](https://hypr.land), a tiling Wayland compositor, plus Omarchy's
bindings and helper scripts. The facts below come from the Omarchy source
(`default/hypr/bindings/tiling.lua`, `utilities.lua`, `looknfeel.lua`,
`workspace-modes.lua`, `bin/omarchy-hyprland-window-*`), read for this document.

### 2.1 Window management features

| Feature | Omarchy binding | What it does |
|---|---|---|
| Directional focus | `SUPER + ←↑→↓` | Focus the neighbour in that direction. |
| Swap | `SUPER + SHIFT + ←↑→↓` | Swap the window with its neighbour. |
| Close | `SUPER + W` (or `Q`) | Close the window. |
| Float/tile | `SUPER + T` | Toggle the window between tiled and floating. |
| Fullscreen | `SUPER + F` | Real fullscreen. |
| Full width / maximize | `SUPER + ALT + F` | Maximized inside the layout. |
| Tiled fullscreen | `SUPER + CTRL + F` | Fullscreen inside the tile area. |
| Toggle split | `SUPER + J` | Flip the split direction (dwindle `togglesplit`). |
| Pseudo-tile | `SUPER + P` | Keep a window's size inside its tile. |
| Pop out | `SUPER + O` | Float, resize to 1300×900, centre, **pin** to every workspace; again to undo. |
| Layout cycle | `SUPER + L` / `SUPER + ALT + L` | Cycle the workspace layout: dwindle, scrolling, (master), floating. Saved per workspace. |
| Workspaces | `SUPER + 1..0` | Switch; each monitor has workspaces 1–10 of its own. |
| Move to workspace | `SUPER + SHIFT + 1..0` | Move the window and follow it. |
| Move silently | `SUPER + SHIFT + ALT + 1..0` | Move without following. |
| Next/prev workspace | `SUPER + TAB` / `SUPER + SHIFT + TAB` | Cycle workspaces on this monitor. |
| Former workspace | `SUPER + CTRL + TAB` | Jump back to the previous one. |
| Scratchpad | `SUPER + S` / `` SUPER + ` `` | Toggle a special workspace drawn over the current one. |
| To scratchpad | `SUPER + ALT + S` | Send the window there. |
| Minimize | `SUPER + M` | Minimize to the dock. |
| Window to monitor | `SUPER + SHIFT + CTRL + ←↑→↓` | Move the window to the monitor in that direction. |
| Workspace to monitor | `SUPER + SHIFT + ALT + ←↑→↓` | Move the whole workspace. |
| Resize | `SUPER + -` / `SUPER + =` (+SHIFT for height, ALT = small, CTRL = large) | Grow/shrink by 100 px (25 / 300). |
| Save/restore width | `SUPER + ALT + HOME` / `SUPER + HOME` | Remember a width and return to it. |
| Groups | `SUPER + G` toggle, `SUPER + ALT + ←↑→↓` join, `SUPER + ALT + G` leave, `SUPER + ALT + TAB` cycle, `SUPER + ALT + 1..5` pick | Tabbed groups sharing one tile. |
| Window cycling | `ALT + TAB` | Next window, brought to top. |
| Monitor cycling | `CTRL + ALT + TAB` | Focus next monitor. |
| Mouse | `SUPER + drag` move, `SUPER + right-drag` resize, `SUPER + wheel` workspaces | |
| Cheat sheet | `SUPER + K` | Show every binding. |
| Gaps | `SUPER + SHIFT + BACKSPACE` | Toggle window gaps. |
| Themes | `SUPER + SHIFT + CTRL + SPACE` or `SUPER + CTRL + T` | Theme menu; one switch restyles every app. |

### 2.2 Look and feel defaults

* `gaps_in = 5`, `gaps_out = 10`, `border_size = 2`, no rounding, no blur.
* Active border: a 45° gradient `#33ccff → #00ff99`; inactive `#595959aa`.
* `dwindle` with `preserve_split = true`, `force_split = 2` (new window goes
  right/bottom); `scrolling` with `column_width = 0.49`; `master` with
  `new_status = master` (new windows take the master slot).
* A **floating workspace mode** is emulated by floating each window and
  tagging it, so ending the mode re-tiles only the windows the mode floated.
  Newly floated windows open centred at ~64 % × 68 % of the monitor and
  cascade 24 px apart.
* Themes are a directory of small files (terminal, bar, borders, wallpaper);
  switching re-renders all of them.

### 2.3 Hyprland concepts worth borrowing

* **Dispatchers**: every action is a named command with arguments
  (`movefocus l`, `workspace 3`). Bindings, IPC (`hyprctl dispatch`) and
  scripts all speak the same language. GridPilot copies this exactly.
* **Layouts as plug-ins** operating on an abstract list/tree of windows.
* **Window rules** matched by class/title (float, workspace, size).
* **Special workspaces** (scratchpad) and **pinning**.

---

## 3. What macOS allows

| Need | macOS reality | GridPilot's choice |
|---|---|---|
| Move/resize other apps' windows | Only via Accessibility (`AXUIElement`), needs the user's permission. | `gridpilot-macos::ax`, the same API Divvy, Rectangle and AeroSpace use. |
| Stable window ids | `_AXUIElementGetWindow` (private, used by yabai, AeroSpace, Hammerspoon for years) gives the `CGWindowID`. | Used as `WindowId`. |
| Global hotkeys | Carbon `RegisterEventHotKey` (no extra permission, swallows the key) or a CGEventTap (needs Input Monitoring, can see every key). | Carbon. An event tap is on the roadmap for mouse bindings. |
| Workspaces | macOS *Spaces* have no public API; driving them needs SIP partly disabled (yabai) or flaky keyboard simulation. | **Emulate**: each display shows one GridPilot workspace; windows of hidden workspaces are parked just off a screen corner (AeroSpace's approach). Works with SIP on, instantly, and survives restarts. |
| Window events | `AXObserver` notifications per app, plus `NSWorkspace` app launch/quit notifications. | v0.1 polls (`general.poll_interval_ms`, default 250 ms); observers are on the roadmap. The engine already accepts events (`PlatformEvent`). |
| Drawing overlays | Borderless, non-activating `NSPanel`s at status-window level. | Grid overlay, HUD, key cheat sheet. |
| Coordinates | AppKit: origin bottom-left of the primary screen. AX/Quartz: top-left. | The core uses top-left everywhere; only the backend flips. |
| Modifier keys | Cmd is owned by apps (`cmd+w`, `cmd+t` …). There is no free "Super". | `mod` alias, default `ctrl+alt`. Omarchy's extra `CTRL` becomes `cmd`, its extra `ALT` becomes `shift`/`cmd` per binding (see §5). |

### Comparable macOS tools

| Tool | Model | Notes |
|---|---|---|
| Divvy | Grid, manual | Closed source, the UX reference. |
| Rectangle (Spectacle fork) | Snap shortcuts, manual | MIT, Swift. `ctrl+opt+arrows` halves. |
| AeroSpace | i3-style tiling, emulated workspaces | MIT, Swift. Proves the emulated-workspace approach. |
| yabai + skhd | BSP tiling, real Spaces | Needs SIP partly off for many features. |
| Amethyst | xmonad-style layouts | MIT, Swift. |

GridPilot's niche: **Divvy's grid and Omarchy's tiling in one tool**, chosen
per workspace, built as a feature framework.

---

## 4. Design decisions that follow

1. **Engine/platform split.** All behaviour lives in `gridpilot-core`, a
   pure-Rust crate with no OS calls, tested on any machine with an in-memory
   `MockPlatform`. A backend implements one small trait (`Platform`): list
   displays and windows, set a frame, hide, focus, close, register hotkeys,
   draw UI.
2. **One command language** (Hyprland dispatchers): key bindings, the
   `gridpilot` CLI, the menu bar menu and the overlay all send strings like
   `grid apply 0,0 3x4 / 6x4` or `move-to-workspace 3 --silent`.
3. **Features as plug-ins.** Each behaviour is a `Feature` that registers
   commands, default bindings, layouts and event hooks. Disable one in config
   and its keys disappear. New behaviours are new files, not edits to a
   central switch.
4. **Declarative placement.** Commands only change the model (`World`).
   `World::plan()` computes where every window should be; the engine diffs
   that against what it last applied and calls the platform for the
   difference. Layouts, fullscreen, groups, pinning, scratchpad and hidden
   workspaces compose without knowing about each other.
5. **Tiles, not windows, in layouts.** Layouts arrange *tiles*; a tile holds
   one window or a tabbed group. Groups came for free.
6. **Divvy by default, tiling on demand.** `default_layout = "floating"`;
   `mod+l` turns the current workspace into dwindle/scrolling/master.
   Switching back to floating freezes windows where they are.

## 5. Omarchy → GridPilot key translation

`mod` = `ctrl+alt` stands in for `SUPER`. Because `mod` already contains
Alt and Ctrl, Omarchy's *additional* modifiers are remapped:

| Omarchy | GridPilot | Example |
|---|---|---|
| `SUPER + x` | `mod + x` | `mod+w` close |
| `SUPER + SHIFT + x` | `mod + shift + x` | `mod+shift+left` swap |
| `SUPER + CTRL + x` | `mod + cmd + x` | `mod+cmd+f` maximize |
| `SUPER + ALT + arrows` (group) | `mod + cmd + arrows` | join group |
| `SUPER + SHIFT + CTRL + arrows` | `mod + shift + cmd + arrows` | window to display |
| `SUPER + SHIFT + ALT + N` | `mod + shift + cmd + N` | move silently |
| `SUPER + ALT + L` / `SUPER + ALT + S` / `SUPER + ALT + G` | `mod + shift + l / s / g` | |
| `SUPER + CTRL + TAB` | `mod + backslash` | former workspace |
| `CTRL + ALT + TAB` | `mod + period` / `mod + comma` | next/prev display |
| `SUPER + SPACE` (launcher) | `mod + space` → **grid overlay** | macOS has Spotlight |

With a Hyper key (Caps Lock → ctrl+alt+shift+cmd via Karabiner) set
`mod = "hyper"`; note that shift-based variants then collide, so prefer the
default unless you rebind those.

The complete generated table is in [KEYS.md](KEYS.md).

---

## Sources

* Mizage, *Divvy user guide* — <https://mizage.com/help/divvy/user_guide.html>
* Mizage, *Divvy for Mac OS X documentation* (PDF) — <https://mizage.com/downloads/DivvyMacHelp.pdf>
* Mizage, *Divvy for Windows documentation* (PDF) — <https://mizage.com/downloads/DivvyWinHelp.pdf>
* *Divvy – Window Manager*, Mac App Store — <https://apps.apple.com/app/id413857545>
* Macworld, *Divvy 1.2.3* review — <https://www.macworld.com/article/1159013/divvy.html>
* Basecamp, *Omarchy* source — <https://github.com/basecamp/omarchy> (`default/hypr/bindings/tiling.lua`, `default/hypr/bindings/utilities.lua`, `default/hypr/looknfeel.lua`, `default/hypr/workspace-modes.lua`, `bin/omarchy-hyprland-window-pop`)
* *The Omarchy Manual* — <https://learn.omacom.io/2/the-omarchy-manual>
* Hyprland wiki — <https://wiki.hypr.land>
* AeroSpace — <https://github.com/nikitabobko/AeroSpace> (emulated workspaces)
