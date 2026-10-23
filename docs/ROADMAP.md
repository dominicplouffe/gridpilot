# Roadmap

GridPilot 0.1 is the framework plus the first set of features. Each item
below is meant to land as one feature module or one backend improvement.

## 0.1 — foundation (this release)

- [x] Engine/platform split, command language, feature plug-ins, event hooks
- [x] Divvy grid: overlay on every display, drag/keyboard selection, subdivide, resize grid, named global/local shortcuts, cycle displays, margins
- [x] Layouts: floating (Divvy), dwindle, master, scrolling; per-workspace, cycled with `mod+l`
- [x] Omarchy window controls: focus/swap, float, fullscreen/maximize, pop-out (float+pin+centre), split toggle, resize, saved width, close, minimize
- [x] Emulated workspaces 1–10, move with/without follow, former workspace, move window/workspace between displays
- [x] Scratchpad, tabbed groups, window rules, themes, gaps toggle, key cheat sheet
- [x] macOS backend: Accessibility, Carbon hotkeys, AppKit overlay, HUD, menu bar
- [x] CLI over a Unix socket, `init`, `check`, `keymap`, `simulate`

## 0.2 — feel native

- [ ] `AXObserver` + `NSWorkspace` notifications instead of polling
- [ ] Focus border window in the theme's accent (Omarchy's gradient border)
- [ ] Group tab bar drawn above grouped tiles
- [ ] Shortcut list in the grid overlay, clickable (Divvy)
- [ ] Launch at login, signed and notarized `GridPilot.app`, Homebrew cask
- [ ] Settings window for grid size, margins and shortcuts (record a shortcut by drawing it, like Divvy)

## 0.3 — more Omarchy

- [ ] Mouse bindings: `mod+drag` to move, `mod+right-drag` to resize, `mod+wheel` for workspaces (CGEventTap)
- [ ] Drag a tiled window onto another to swap; drop on an edge to split there
- [ ] Pseudo-tiling (`SUPER + P`), tiled fullscreen (`SUPER + CTRL + F`)
- [ ] Workspace layouts remembered across restarts
- [ ] Per-display workspace sets (Omarchy's "each monitor has 1–10")
- [ ] Animations for workspace switches and snaps

## Later

- [ ] Windows backend (Divvy ships on Windows too)
- [ ] Hyprland backend over `hyprctl` so one config drives an Omarchy machine
- [ ] Scriptable features (Lua or WASM) loaded from `~/.config/gridpilot/features/`
- [ ] Theme sync: generate terminal/editor colours from the active GridPilot theme
