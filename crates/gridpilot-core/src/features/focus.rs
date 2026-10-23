//! Keyboard focus and swapping, from Omarchy's `SUPER + arrows` family.

use crate::command::{Args, CmdResult};
use crate::engine::Engine;
use crate::feature::{Feature, Registrar};
use crate::geometry::{nearest_in_direction, Direction};
use crate::platform::{DisplayId, WindowId};

pub struct FocusFeature;

impl Feature for FocusFeature {
    fn id(&self) -> &'static str {
        "focus"
    }

    fn summary(&self) -> &'static str {
        "Directional focus, swapping and window cycling"
    }

    fn install(&self, r: &mut Registrar<'_>) {
        r.command("focus", "focus <left|right|up|down|next|prev>", "Focus the window in a direction, or cycle", focus);
        r.command("swap", "swap <left|right|up|down>", "Swap the focused tile with its neighbour", swap);
        r.command(
            "focus-display",
            "focus-display <next|prev|left|right|up|down>",
            "Focus another display",
            focus_display,
        );

        for d in Direction::ALL {
            r.bind(&format!("mod+{d}"), &format!("focus {d}"), &format!("Focus window {d}"));
            r.bind(&format!("mod+shift+{d}"), &format!("swap {d}"), &format!("Swap window {d}"));
        }
        r.bind("alt+tab", "focus next", "Focus next window");
        r.bind("alt+shift+tab", "focus prev", "Focus previous window");
        // Omarchy uses CTRL+ALT+TAB, which is `mod+tab` here.
        r.bind("mod+period", "focus-display next", "Focus next display");
        r.bind("mod+comma", "focus-display prev", "Focus previous display");
    }
}

/// Windows of the current workspace in layout order, then floating ones.
fn cycle_order(e: &Engine) -> Vec<WindowId> {
    let Some(ws) = e.world.current_workspace().and_then(|n| e.world.workspaces.get(&n)) else { return vec![] };
    let mut v: Vec<WindowId> =
        ws.layout.tiles().iter().filter_map(|t| ws.tiles.get(t)).filter_map(|t| t.active_window()).collect();
    for t in ws.tiles.values() {
        if let Some(w) = t.active_window() {
            if !v.contains(&w) {
                v.push(w);
            }
        }
    }
    v.extend(ws.floating.iter().copied());
    v
}

fn focus(e: &mut Engine, a: &Args) -> CmdResult {
    let what = a.require(0, "a direction or next/prev")?;
    if what == "next" || what == "prev" {
        let order = cycle_order(e);
        if order.is_empty() {
            return Ok(None);
        }
        let i = e.focused().and_then(|f| order.iter().position(|x| *x == f));
        let n = order.len();
        let next = match (i, what) {
            (None, _) => 0,
            (Some(i), "next") => (i + 1) % n,
            (Some(i), _) => (i + n - 1) % n,
        };
        e.focus_window(order[next]);
        return Ok(None);
    }
    let dir: Direction = what.parse()?;
    match e.neighbour(dir) {
        Some(id) => e.focus_window(id),
        None if e.focused().is_none() => {
            if let Some(first) = cycle_order(e).first().copied() {
                e.focus_window(first);
            }
        }
        // Nothing that way on this display: try the next display over.
        None => {
            if let Some(d) = display_towards(e, dir) {
                focus_on_display(e, d);
            }
        }
    }
    Ok(None)
}

fn swap(e: &mut Engine, a: &Args) -> CmdResult {
    let dir = a.direction(0)?;
    let me = e.require_focused()?;
    let Some(other) = e.neighbour(dir) else { return Ok(None) };
    let (Some((ws_a, ta, _)), Some((ws_b, tb, _))) = (e.world.tile_of(me), e.world.tile_of(other)) else {
        return Ok(None);
    };
    if ws_a.name != ws_b.name {
        return Ok(None);
    }
    let name = ws_a.name.clone();
    if let Some(ws) = e.world.workspaces.get_mut(&name) {
        ws.layout.swap(ta, tb);
    }
    e.focus_window(me);
    Ok(None)
}

pub(crate) fn display_towards(e: &Engine, dir: Direction) -> Option<DisplayId> {
    let cur = e.world.current_display()?;
    let from = e.world.display(cur)?.frame;
    nearest_in_direction(&from, dir, e.world.displays.iter().filter(|d| d.id != cur).map(|d| (d.id, &d.frame)))
}

pub(crate) fn display_step(e: &Engine, forward: bool) -> Option<DisplayId> {
    let cur = e.world.current_display()?;
    let mut ds: Vec<_> = e.world.displays.iter().map(|d| (d.frame.x, d.frame.y, d.id)).collect();
    ds.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
    let i = ds.iter().position(|d| d.2 == cur)?;
    let n = ds.len();
    Some(ds[if forward { (i + 1) % n } else { (i + n - 1) % n }].2)
}

pub(crate) fn focus_on_display(e: &mut Engine, d: DisplayId) {
    e.world.focused_display = Some(d);
    e.world.scratchpad_on = None;
    match e.world.active_workspace(d).map(str::to_string) {
        Some(ws) => e.focus_fallback(&ws),
        None => e.world.focused = None,
    }
}

fn focus_display(e: &mut Engine, a: &Args) -> CmdResult {
    let target = match a.require(0, "next/prev or a direction")? {
        "next" => display_step(e, true),
        "prev" => display_step(e, false),
        dir => display_towards(e, dir.parse()?),
    };
    if let Some(d) = target {
        focus_on_display(e, d);
    }
    Ok(None)
}
