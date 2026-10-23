//! Divvy: draw a region on a grid and the window snaps to it, plus named
//! shortcuts for favourite regions.

use crate::command::Args;
use crate::engine::Engine;
use crate::feature::{EngineEvent, Feature, Registrar};
use crate::grid::{Cell, GridSelection, GridSession};
use crate::platform::{DisplayId, GridOverlay, UiRequest, WindowId};

pub struct GridFeature;

#[derive(Default)]
struct GridState {
    session: Option<Session>,
    /// The last shortcut applied, for "press again to move to the next display".
    last: Option<(String, WindowId, DisplayId)>,
}

struct Session {
    grid: GridSession,
    target: WindowId,
    display: DisplayId,
}

impl Feature for GridFeature {
    fn id(&self) -> &'static str {
        "grid"
    }

    fn summary(&self) -> &'static str {
        "Divvy-style grid overlay and placement shortcuts"
    }

    fn install(&self, r: &mut Registrar<'_>) {
        r.command(
            "grid",
            "grid show|hide|toggle | apply <col,row WxH / CxR> [--display ID] | shortcut <name> | key <dir> [--extend] | press|drag|release <col> <row> [--display ID] | subdivide | size <dcols> <drows> | all | commit [--display ID] | local <chord> | list",
            "Open the grid overlay, drive it, or place the focused window on a grid region",
            grid,
        );
        r.bind("mod+space", "grid toggle", "Show the grid overlay");
        for s in r.config().grid.shortcuts.clone() {
            if let (Some(keys), true) = (s.keys.as_deref(), s.global) {
                r.bind_from_config(keys, &format!("grid shortcut \"{}\"", s.name), &s.name);
            }
        }
        r.on(|e, ev| {
            if let EngineEvent::WindowRemoved(id) = ev {
                let st = e.ext::<GridState>();
                if st.session.as_ref().is_some_and(|s| s.target == *id) {
                    st.session = None;
                    e.present(UiRequest::HideGrid);
                }
            }
        });
    }
}

fn grid(e: &mut Engine, a: &Args) -> crate::command::CmdResult {
    let sub = a.get(0).unwrap_or("toggle");
    match sub {
        "show" => open(e)?,
        "hide" | "cancel" => {
            // The overlay took keyboard focus; hand it back.
            let target = e.ext::<GridState>().session.as_ref().map(|s| s.target);
            close(e);
            if let Some(t) = target {
                e.focus_window(t);
            }
        }
        "toggle" => {
            if e.ext::<GridState>().session.is_some() {
                return e.run("grid cancel");
            }
            open(e)?
        }
        "apply" => {
            let spec: Vec<&str> =
                a.words[1..].iter().take_while(|w| !w.starts_with("--")).map(|s| s.as_str()).collect();
            let sel: GridSelection = spec.join(" ").parse()?;
            let display = display_flag(a)?;
            apply(e, sel, display)?;
        }
        "shortcut" => {
            let name = a.rest(1);
            let sc = e
                .config
                .grid
                .shortcuts
                .iter()
                .find(|s| s.name.eq_ignore_ascii_case(&name))
                .cloned()
                .ok_or_else(|| format!("no grid shortcut named `{name}`"))?;
            let target = target(e)?;
            let here = window_display(e, target)?;
            let display = match e.ext::<GridState>().last.clone() {
                Some((n, w, d)) if e.config.grid.cycle_displays && n == sc.name && w == target && d == here => {
                    next_display(e, d)
                }
                _ => here,
            };
            apply(e, sc.cells, Some(display))?;
            e.ext::<GridState>().last = Some((sc.name, target, display));
        }
        "local" => {
            let chord = crate::keys::Chord::parse(a.require(1, "a key chord")?, &e.config.modifiers)?;
            let aliases = e.config.modifiers.clone();
            let hit =
                e.config.grid.shortcuts.iter().find(|s| {
                    s.keys.as_deref().and_then(|k| crate::keys::Chord::parse(k, &aliases).ok()) == Some(chord)
                });
            if let Some(sc) = hit.cloned() {
                apply(e, sc.cells, None)?;
            }
        }
        "list" => {
            let lines: Vec<String> = e
                .config
                .grid
                .shortcuts
                .iter()
                .map(|s| {
                    format!(
                        "{:<20} {:<16} {}{}",
                        s.name,
                        s.keys.clone().unwrap_or_default(),
                        s.cells,
                        if s.global { "" } else { "  (local)" }
                    )
                })
                .collect();
            return Ok(Some(lines.join("\n")));
        }
        "key" | "press" | "drag" | "release" | "subdivide" | "size" | "all" | "commit" => {
            let display_override = display_flag(a)?;
            let st = e.ext::<GridState>();
            let Some(s) = st.session.as_mut() else {
                return Err("the grid overlay is not open".into());
            };
            let mut finished = None;
            match sub {
                "key" => s.grid.key_move(a.direction(1)?, a.has_flag("--extend")),
                "press" => s.grid.mouse_down(cell(a)?),
                "drag" => s.grid.mouse_drag(cell(a)?),
                "release" => finished = s.grid.mouse_up(cell(a)?),
                "subdivide" => s.grid.toggle_subdivide(),
                "size" => s.grid.resize_grid(a.parse(1, "column delta")?, a.parse(2, "row delta")?),
                "all" => s.grid.select_all(),
                "commit" => finished = Some(s.grid.selection()),
                _ => unreachable!(),
            }
            if let Some(d) = display_override {
                s.display = d;
            }
            match finished {
                Some(sel) => {
                    let d = s.display;
                    apply(e, sel, Some(d))?;
                }
                None => redraw(e),
            }
        }
        other => return Err(format!("unknown `grid {other}`")),
    }
    Ok(None)
}

fn cell(a: &Args) -> Result<Cell, String> {
    Ok(Cell { col: a.parse(1, "column")?, row: a.parse(2, "row")? })
}

fn display_flag(a: &Args) -> Result<Option<DisplayId>, String> {
    match a.words.iter().position(|w| w == "--display") {
        Some(i) => Ok(Some(DisplayId(a.parse(i + 1, "display id")?))),
        None => Ok(None),
    }
}

/// The window the overlay was opened for, else the focused one.
fn target(e: &mut Engine) -> Result<WindowId, String> {
    if let Some(s) = e.ext::<GridState>().session.as_ref() {
        return Ok(s.target);
    }
    e.require_focused()
}

fn window_display(e: &Engine, id: WindowId) -> Result<DisplayId, String> {
    let w = e.world.windows.get(&id).ok_or("window is gone")?;
    e.world
        .workspaces
        .get(&w.workspace)
        .map(|ws| ws.display)
        .filter(|_| w.workspace != crate::world::SCRATCHPAD)
        .or_else(|| e.world.current_display())
        .ok_or_else(|| "no display".into())
}

/// Displays ordered left-to-right, then top-to-bottom.
fn next_display(e: &Engine, d: DisplayId) -> DisplayId {
    let mut ds: Vec<_> = e.world.displays.iter().map(|x| (x.frame.x, x.frame.y, x.id)).collect();
    ds.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
    let i = ds.iter().position(|x| x.2 == d).unwrap_or(0);
    ds[(i + 1) % ds.len()].2
}

fn open(e: &mut Engine) -> Result<(), String> {
    let target = e.require_focused()?;
    let display = window_display(e, target)?;
    let visible = e.world.display(display).map(|d| d.visible).unwrap_or_default();
    let frame = e.frame_of(target).unwrap_or_default();
    let grid = GridSession::around(e.config.grid.size, frame, visible);
    e.ext::<GridState>().session = Some(Session { grid, target, display });
    redraw(e);
    Ok(())
}

fn close(e: &mut Engine) {
    if e.ext::<GridState>().session.take().is_some() {
        e.present(UiRequest::HideGrid);
    }
}

fn redraw(e: &mut Engine) {
    let margins = e.config.grid_margins();
    let theme = e.theme.clone();
    let shortcuts = e
        .config
        .grid
        .shortcuts
        .iter()
        .map(|s| (s.name.clone(), s.keys.clone().unwrap_or_default(), format!("grid apply {}", s.cells)))
        .collect();
    let Some(s) = e.ext::<GridState>().session.as_ref() else { return };
    let overlay =
        GridOverlay { target: s.target, session: s.grid.clone(), display: s.display, margins, theme, shortcuts };
    e.present(UiRequest::ShowGrid(overlay));
}

/// Place the target window on `sel` on `display` (default: its own).
fn apply(e: &mut Engine, sel: GridSelection, display: Option<DisplayId>) -> Result<(), String> {
    let id = target(e)?;
    let display = match display {
        Some(d) => d,
        None => window_display(e, id)?,
    };
    let visible = e.world.display(display).ok_or("unknown display")?.visible;
    let frame = sel.frame(visible, &e.config.grid_margins());

    let w = e.world.windows.get(&id).ok_or("window is gone")?;
    let ws = w.workspace.clone();
    if window_display(e, id)? != display {
        let to = e.world.active_workspace(display).ok_or("display shows no workspace")?.to_string();
        e.move_window_to_workspace(id, &to, false);
    }
    let ws = e.world.windows.get(&id).map(|w| w.workspace.clone()).unwrap_or(ws);
    let tiled = e.world.workspaces.get(&ws).is_some_and(|w| w.layout.manages_geometry());
    if tiled && !e.world.windows[&id].floating {
        if !e.config.grid.float_on_apply {
            return Err("window is tiled; set grid.float_on_apply = true to float it on apply".into());
        }
        e.set_floating(id, true);
    }
    if let Some(w) = e.world.windows.get_mut(&id) {
        w.float_frame = frame;
        w.fullscreen = None;
    }
    e.focus_window(id);
    if e.config.grid.keep_open {
        if let Some(s) = e.ext::<GridState>().session.as_mut() {
            s.display = display;
        }
        redraw(e);
    } else {
        close(e);
    }
    Ok(())
}
