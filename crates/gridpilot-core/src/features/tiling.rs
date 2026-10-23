//! Per-window tiling controls from Omarchy: float, fullscreen, pop-out,
//! split direction, resize, layout cycling and saved widths.

use crate::command::{Args, CmdResult};
use crate::engine::Engine;
use crate::feature::{Feature, Registrar};
use crate::geometry::{Axis, Size};
use crate::platform::WindowId;
use crate::world::FullscreenMode;
use std::collections::HashMap;

pub struct TilingFeature;

#[derive(Default)]
struct SavedWidths(HashMap<WindowId, f64>);

impl Feature for TilingFeature {
    fn id(&self) -> &'static str {
        "tiling"
    }

    fn summary(&self) -> &'static str {
        "Float, fullscreen, pop-out, split, resize and layout cycling"
    }

    fn install(&self, r: &mut Registrar<'_>) {
        r.command("close", "close", "Close the focused window", |e, _| {
            let id = e.require_focused()?;
            e.platform().close(id)?;
            Ok(None)
        });
        r.command("minimize", "minimize", "Minimize the focused window", |e, _| {
            let id = e.require_focused()?;
            e.platform().minimize(id)?;
            Ok(None)
        });
        r.command("float", "float [toggle|on|off]", "Float or tile the focused window", float);
        r.command(
            "fullscreen",
            "fullscreen [full|maximize|off]",
            "Toggle fullscreen (no gaps) or maximized (inside gaps)",
            fullscreen,
        );
        r.command(
            "pop",
            "pop [width height]",
            "Pop the window out: float, pin to every workspace and centre (toggle)",
            pop,
        );
        r.command("pin", "pin [toggle|on|off]", "Show a floating window on every workspace of its display", pin);
        r.command("center", "center", "Centre a floating window on its display", center);
        r.command("split", "split toggle", "Flip the direction of the split the window sits in", |e, _| {
            let id = e.require_focused()?;
            let (ws, t) = e.world.tile_of(id).map(|(ws, t, _)| (ws.name.clone(), t)).ok_or("window is not tiled")?;
            if let Some(ws) = e.world.workspaces.get_mut(&ws) {
                ws.layout.toggle_split(t);
            }
            Ok(None)
        });
        r.command("resize", "resize <width|height> <+/-pixels>", "Grow or shrink the focused window", resize);
        r.command("width", "width save|restore", "Remember the window width, or go back to it", width);
        r.command("layout", "layout <name|next|prev>", "Set or cycle the layout of the current workspace", layout);

        r.bind("mod+w", "close", "Close window");
        r.bind("mod+t", "float toggle", "Toggle floating/tiling");
        r.bind("mod+f", "fullscreen full", "Full screen");
        r.bind("mod+shift+f", "fullscreen maximize", "Maximize inside gaps");
        r.bind("mod+o", "pop", "Pop window out (float and pin)");
        r.bind("mod+c", "center", "Centre floating window");
        r.bind("mod+j", "split toggle", "Toggle split direction");
        r.bind("mod+m", "minimize", "Minimize window");
        r.bind("mod+l", "layout next", "Next workspace layout");
        r.bind("mod+shift+l", "layout prev", "Previous workspace layout");
        r.bind("mod+shift+home", "width save", "Save window width");
        r.bind("mod+home", "width restore", "Restore window width");

        let step = r.config().tiling.resize_steps[1];
        r.bind("mod+minus", &format!("resize width -{step}"), "Shrink width");
        r.bind("mod+equal", &format!("resize width +{step}"), "Grow width");
        r.bind("mod+shift+minus", &format!("resize height -{step}"), "Shrink height");
        r.bind("mod+shift+equal", &format!("resize height +{step}"), "Grow height");
    }
}

fn toggle_arg(a: &Args, current: bool) -> Result<bool, String> {
    match a.get(0).unwrap_or("toggle") {
        "toggle" => Ok(!current),
        "on" | "true" | "yes" => Ok(true),
        "off" | "false" | "no" => Ok(false),
        other => Err(format!("`{}`: expected toggle/on/off, got `{other}`", a.name)),
    }
}

fn float(e: &mut Engine, a: &Args) -> CmdResult {
    let id = e.require_focused()?;
    let on = toggle_arg(a, e.world.windows[&id].floating)?;
    e.set_floating(id, on);
    Ok(None)
}

fn fullscreen(e: &mut Engine, a: &Args) -> CmdResult {
    let id = e.require_focused()?;
    let mode = match a.get(0).unwrap_or("full") {
        "full" | "fullscreen" => Some(FullscreenMode::Full),
        "maximize" | "maximized" => Some(FullscreenMode::Maximized),
        "off" => None,
        other => return Err(format!("unknown fullscreen mode `{other}`")),
    };
    let w = e.world.windows.get_mut(&id).ok_or("window is gone")?;
    w.fullscreen = if w.fullscreen == mode { None } else { mode };
    Ok(None)
}

fn pop(e: &mut Engine, a: &Args) -> CmdResult {
    let id = e.require_focused()?;
    let [dw, dh] = e.config.tiling.pop_size;
    let size = Size::new(a.parse(0, "width").unwrap_or(dw), a.parse(1, "height").unwrap_or(dh));
    let popped = e.world.windows[&id].tags.contains("pop");
    if popped {
        if let Some(w) = e.world.windows.get_mut(&id) {
            w.pinned = false;
            w.tags.remove("pop");
        }
        e.set_floating(id, false);
        return Ok(None);
    }
    e.set_floating(id, true);
    let area = e.workspace_area(&e.world.windows[&id].workspace);
    if let Some(w) = e.world.windows.get_mut(&id) {
        w.float_frame = area.centered(size);
        w.pinned = true;
        w.tags.insert("pop".into());
    }
    Ok(None)
}

fn pin(e: &mut Engine, a: &Args) -> CmdResult {
    let id = e.require_focused()?;
    let on = toggle_arg(a, e.world.windows[&id].pinned)?;
    if on {
        e.set_floating(id, true);
    }
    if let Some(w) = e.world.windows.get_mut(&id) {
        w.pinned = on;
    }
    Ok(None)
}

fn center(e: &mut Engine, _: &Args) -> CmdResult {
    let id = e.require_focused()?;
    let w = &e.world.windows[&id];
    let manages = e.world.workspaces.get(&w.workspace).is_some_and(|ws| ws.layout.manages_geometry());
    if manages && !w.floating {
        return Err("only floating windows can be centred".into());
    }
    let area = e.workspace_area(&w.workspace);
    let size = e.frame_of(id).unwrap_or(w.float_frame).size();
    if let Some(w) = e.world.windows.get_mut(&id) {
        w.float_frame = area.centered(size);
    }
    Ok(None)
}

fn resize(e: &mut Engine, a: &Args) -> CmdResult {
    let id = e.require_focused()?;
    let axis = match a.require(0, "width or height")? {
        "width" | "w" | "x" => Axis::Horizontal,
        "height" | "h" | "y" => Axis::Vertical,
        other => return Err(format!("resize: expected width/height, got `{other}`")),
    };
    let delta: f64 = a.parse(1, "pixel delta")?;
    resize_by(e, id, axis, delta);
    Ok(None)
}

fn resize_by(e: &mut Engine, id: WindowId, axis: Axis, delta: f64) {
    let opts = e.layout_options();
    let w = &e.world.windows[&id];
    let ws_name = w.workspace.clone();
    let tiled = w.tile.filter(|_| e.world.workspaces.get(&ws_name).is_some_and(|ws| ws.layout.manages_geometry()));
    match tiled {
        Some(t) if !w.floating => {
            let area = e.workspace_area(&ws_name);
            if let Some(ws) = e.world.workspaces.get_mut(&ws_name) {
                ws.layout.resize(t, axis, delta, area, &opts);
            }
        }
        _ => {
            let current = e.frame_of(id).unwrap_or(w.float_frame);
            if let Some(w) = e.world.windows.get_mut(&id) {
                w.float_frame = current;
                match axis {
                    Axis::Horizontal => w.float_frame.w = (w.float_frame.w + delta).max(100.0),
                    Axis::Vertical => w.float_frame.h = (w.float_frame.h + delta).max(80.0),
                }
            }
        }
    }
}

fn width(e: &mut Engine, a: &Args) -> CmdResult {
    let id = e.require_focused()?;
    let current = e.frame_of(id).map(|f| f.w).unwrap_or_default();
    match a.require(0, "save or restore")? {
        "save" => {
            e.ext::<SavedWidths>().0.insert(id, current);
            Ok(Some(format!("saved width {current}")))
        }
        "restore" => {
            let saved = e.ext::<SavedWidths>().0.get(&id).copied().ok_or("no saved width for this window")?;
            resize_by(e, id, Axis::Horizontal, saved - current);
            Ok(None)
        }
        other => Err(format!("width: expected save/restore, got `{other}`")),
    }
}

fn layout(e: &mut Engine, a: &Args) -> CmdResult {
    let ws = e.world.current_workspace().ok_or("no workspace")?;
    let current = e.world.workspaces[&ws].layout.name().to_string();
    let want = a.require(0, "a layout name or next/prev")?;
    let name = match want {
        "next" | "prev" => {
            let cycle = &e.config.tiling.cycle;
            if cycle.is_empty() {
                return Err("tiling.cycle is empty".into());
            }
            let i = cycle.iter().position(|n| *n == current);
            let n = cycle.len();
            let j = match (i, want) {
                (None, _) => 0,
                (Some(i), "next") => (i + 1) % n,
                (Some(i), _) => (i + n - 1) % n,
            };
            cycle[j].clone()
        }
        name => name.to_string(),
    };
    e.set_layout(&ws, &name)?;
    e.present(crate::platform::UiRequest::Notify { title: format!("Workspace {ws}"), body: format!("Layout: {name}") });
    Ok(Some(name))
}
