# Architecture

GridPilot is built so that features can keep being added without the code
turning into a ball of special cases. The research behind these choices is in
[RESEARCH.md](RESEARCH.md).

## Crates

```
crates/
├── gridpilot-core      pure Rust, no OS calls, runs and tests anywhere
│   ├── geometry.rs     Rect, Direction, nearest_in_direction()
│   ├── grid.rs         Divvy: Grid, GridSelection, GridSession (overlay state machine)
│   ├── keys.rs         Chord parsing with modifier aliases (mod, grid, hyper)
│   ├── layout/         Layout trait + dwindle, master, scrolling, floating
│   ├── world.rs        displays, workspaces, tiles, windows; World::plan()
│   ├── command.rs      the command language and registry
│   ├── feature.rs      Feature trait, Registrar, EngineEvent hooks
│   ├── features/       built-in features (grid, focus, tiling, workspaces, scratchpad, groups, system)
│   ├── engine.rs       runs commands, syncs with the platform, applies the plan
│   ├── platform.rs     the Platform trait a backend implements
│   ├── config.rs       gridpilot.toml schema
│   ├── theme.rs        colour themes
│   └── testing.rs      MockPlatform
├── gridpilot-macos     Accessibility, Carbon hotkeys, AppKit overlay/HUD/menu bar, run loop
└── gridpilot           the binary: daemon, CLI client (Unix socket), init/check/keymap/simulate
```

## Data flow

```
 Carbon hotkey ─┐
 menu bar item ─┤
 grid overlay  ─┼─► command line ──► Engine::dispatch ──► feature handler ──► mutates World
 gridpilot CLI ─┘   "focus left"                                                   │
                                                                                   ▼
 Platform ◄── set_frame / hide / focus (only what changed) ◄── diff ◄── World::plan()
     │
     └── windows(), displays(), focused_window(), events ──► Engine::refresh / handle
```

1. **Input** of every kind becomes a command line. There is exactly one way
   to make GridPilot do something, so bindings, scripts and UI never drift.
2. **Commands mutate the model** (`World`) through engine helpers such as
   `show_workspace`, `move_window_to_workspace`, `set_floating`, `set_layout`.
   They never move windows themselves.
3. **`World::plan()`** computes the desired `Placement` (`Show(rect)` or
   `Hide`) of every window from: the active workspace of each display, its
   layout's arrangement of tiles, the active tab of each group, floating
   frames, fullscreen, pinned windows and the scratchpad.
4. **`Engine::flush()`** diffs the plan against what it last applied and
   sends only the changes to the platform, then moves keyboard focus if a
   command changed it. Hooks for queued `EngineEvent`s run first.
5. **Refresh** (`Engine::refresh`, polled by the macOS run loop) reconciles
   the model with reality: new windows are added (window rules apply), gone
   windows removed, windows the user dragged become the new floating frame
   (or snap back if tiled), and OS focus changes are adopted. Focusing a
   window that lives on a hidden workspace (⌘-Tab, Dock) brings its
   workspace forward.

## The model

* **Display** → shows exactly one **workspace** (`World::active`).
* **Workspace** → has a `Box<dyn Layout>`, a map of **tiles**, and a list of
  floating windows. Workspaces are global names (`"1"`…`"10"`, or any name)
  assigned to a display; empty hidden ones are garbage-collected so the number
  can reappear on whichever display wants it.
* **Tile** → one window, or several (`grouped`) shown as tabs.
* **ManagedWindow** → which workspace and tile, `floating`, `float_frame`,
  `pinned`, `fullscreen`, free-form `tags`.
* **Scratchpad** → the special workspace `"scratchpad"`, shown over the
  current display when `World::scratchpad_on` is set.

## Extending GridPilot

### Add a feature

A feature is one file. It registers commands, default bindings and hooks:

```rust
use gridpilot_core::{Engine, EngineEvent, Feature, Registrar};

pub struct AutoCenterDialogs;

impl Feature for AutoCenterDialogs {
    fn id(&self) -> &'static str { "auto-center" }
    fn summary(&self) -> &'static str { "Centre new floating windows" }

    fn install(&self, r: &mut Registrar<'_>) {
        r.command("center-all", "center-all", "Centre every floating window", |e, _| {
            let ws = e.world.current_workspace().ok_or("no workspace")?;
            let area = e.workspace_area(&ws);
            for id in e.world.workspaces[&ws].floating.clone() {
                let w = e.world.windows.get_mut(&id).unwrap();
                w.float_frame = area.centered(w.float_frame.size());
            }
            Ok(None)
        });
        r.bind("mod+shift+c", "center-all", "Centre floating windows");
        r.on(|e: &mut Engine, ev| {
            if let EngineEvent::WindowAdded(id) = ev {
                if e.world.windows[id].floating {
                    let _ = e.run("center-all");
                }
            }
        });
    }
}
```

Then add it to `features::builtin()` (or pass your own list to
`Engine::new`). Users can switch it off with
`[features] disabled = ["auto-center"]`, which also removes its bindings.

Per-feature state goes in `engine.ext::<MyState>()` (any `Default` type).

### Add a layout

Implement `layout::Layout` (insert, remove, tiles, arrange, swap, and
optionally toggle_split, resize, on_focus) and register it, either in
`LayoutRegistry::default()` or from a feature with
`r.layout("spiral", || Box::new(Spiral::default()))`. It immediately works
with `layout spiral`, `tiling.cycle`, per-workspace `[workspaces.layout]`,
groups, swap and directional focus, because those operate on tiles and on
the geometry the layout returns.

### Add a backend

Implement `platform::Platform` (≈10 methods). `MockPlatform` in
`testing.rs` is a complete reference in 150 lines. Likely next backends:

* **Windows** — `EnumWindows`/`SetWindowPos`, `RegisterHotKey`, a layered
  window for the overlay. Divvy itself ships on Windows.
* **Hyprland** — talk to `hyprctl` over its socket; useful for running the
  same config on an Omarchy machine.

### Add a command to an existing feature

Add `r.command(...)` in that feature's `install`. Commands receive `Args`
with helpers (`direction`, `parse`, `has_flag`, `rest`). Return
`Ok(Some(text))` to print something in the CLI.

## Testing

* Unit tests sit next to the code (grid math, layouts, parsing, config).
* `crates/gridpilot-core/tests/engine.rs` drives the full engine with every
  built-in feature through command lines against `MockPlatform`: tiling,
  Divvy shortcuts with display cycling, the overlay drag flow, workspaces,
  scratchpad, groups, pop-out, rules, display unplugging.
* `gridpilot simulate` is the same engine interactively.
* The macOS crate is type-checked on Linux with
  `cargo check --target aarch64-apple-darwin`, and CI also builds it on a macOS runner.

## Conventions

* Coordinates are global, top-left origin, `y` down, in points.
* Commands are lowercase, words separated by spaces, flags as `--flag`.
* Every binding has a human description; it is what `mod+k` shows.
* A feature never calls the platform to move a window; it changes the model
  and lets the plan do it.
