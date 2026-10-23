//! The engine's model of the desktop: displays, workspaces, tiles and
//! windows, plus [`World::plan`], which turns that model into the frame every
//! window should have. Nothing here talks to the OS.

use crate::geometry::{Rect, Size};
use crate::layout::{Layout, LayoutOptions, TileId};
use crate::platform::{DisplayId, DisplayInfo, WindowId, WindowInfo};
use std::collections::{BTreeMap, BTreeSet};

pub const SCRATCHPAD: &str = "scratchpad";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FullscreenMode {
    /// Cover the whole visible area, no gaps (`SUPER + F`).
    Full,
    /// Cover the area inside the outer gaps (`SUPER + ALT + F`).
    Maximized,
}

#[derive(Debug, Clone)]
pub struct ManagedWindow {
    pub info: WindowInfo,
    pub workspace: String,
    /// The tile holding this window while it is tiled.
    pub tile: Option<TileId>,
    pub floating: bool,
    /// Where a floating window lives.
    pub float_frame: Rect,
    /// Visible on every workspace of its display (Omarchy's pop-out).
    pub pinned: bool,
    pub fullscreen: Option<FullscreenMode>,
    pub tags: BTreeSet<String>,
}

/// A slot in a layout: one window, or a tabbed group of them.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Tile {
    pub windows: Vec<WindowId>,
    pub active: usize,
    /// Groups accept new windows moved into them; plain tiles do not.
    pub grouped: bool,
}

impl Tile {
    pub fn active_window(&self) -> Option<WindowId> {
        self.windows.get(self.active.min(self.windows.len().saturating_sub(1))).copied()
    }
}

#[derive(Debug)]
pub struct Workspace {
    pub name: String,
    pub display: DisplayId,
    pub layout: Box<dyn Layout>,
    pub tiles: BTreeMap<TileId, Tile>,
    /// Floating windows, bottom to top.
    pub floating: Vec<WindowId>,
    pub last_focused: Option<WindowId>,
}

impl Workspace {
    pub fn windows(&self) -> impl Iterator<Item = WindowId> + '_ {
        self.tiles.values().flat_map(|t| t.windows.iter().copied()).chain(self.floating.iter().copied())
    }

    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty() && self.floating.is_empty()
    }
}

/// What the engine wants the platform to do with a window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Placement {
    Show(Rect),
    Hide(DisplayId),
}

#[derive(Debug, Default)]
pub struct World {
    pub displays: Vec<DisplayInfo>,
    pub windows: BTreeMap<WindowId, ManagedWindow>,
    pub workspaces: BTreeMap<String, Workspace>,
    /// The workspace shown on each display.
    pub active: BTreeMap<DisplayId, String>,
    pub focused: Option<WindowId>,
    pub focused_display: Option<DisplayId>,
    /// The display the scratchpad is shown on, if it is shown.
    pub scratchpad_on: Option<DisplayId>,
    /// The workspace focused before the current one, per display.
    pub previous: BTreeMap<DisplayId, String>,
    next_tile: u64,
}

/// Order workspaces 1, 2, ..., 10, then named ones alphabetically.
pub fn workspace_order(a: &str, b: &str) -> std::cmp::Ordering {
    match (a.parse::<u32>(), b.parse::<u32>()) {
        (Ok(x), Ok(y)) => x.cmp(&y),
        (Ok(_), Err(_)) => std::cmp::Ordering::Less,
        (Err(_), Ok(_)) => std::cmp::Ordering::Greater,
        _ => a.cmp(b),
    }
}

impl World {
    pub fn new_tile_id(&mut self) -> TileId {
        self.next_tile += 1;
        TileId(self.next_tile)
    }

    pub fn display(&self, id: DisplayId) -> Option<&DisplayInfo> {
        self.displays.iter().find(|d| d.id == id)
    }

    pub fn primary_display(&self) -> Option<DisplayId> {
        self.displays.first().map(|d| d.id)
    }

    /// The display holding most of `frame`, falling back to the primary one.
    pub fn display_for_frame(&self, frame: &Rect) -> Option<DisplayId> {
        self.displays
            .iter()
            .map(|d| (d.id, d.frame.intersection(frame).map_or(0.0, |r| r.area())))
            .filter(|(_, a)| *a > 0.0)
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(id, _)| id)
            .or_else(|| {
                let c = frame.center();
                self.displays.iter().find(|d| d.frame.contains(c)).map(|d| d.id)
            })
            .or_else(|| self.primary_display())
    }

    pub fn current_display(&self) -> Option<DisplayId> {
        self.focused
            .and_then(|w| self.windows.get(&w))
            .and_then(|w| self.workspaces.get(&w.workspace))
            .map(|ws| ws.display)
            .filter(|_| self.scratchpad_on.is_none())
            .or(self.scratchpad_on)
            .or(self.focused_display)
            .or_else(|| self.primary_display())
    }

    pub fn active_workspace(&self, display: DisplayId) -> Option<&str> {
        self.active.get(&display).map(|s| s.as_str())
    }

    pub fn current_workspace(&self) -> Option<String> {
        self.current_display().and_then(|d| self.active.get(&d).cloned())
    }

    pub fn is_visible_workspace(&self, name: &str) -> bool {
        self.active.values().any(|n| n == name) || (name == SCRATCHPAD && self.scratchpad_on.is_some())
    }

    pub fn workspace_names(&self) -> Vec<String> {
        let mut v: Vec<String> = self.workspaces.keys().filter(|n| *n != SCRATCHPAD).cloned().collect();
        v.sort_by(|a, b| workspace_order(a, b));
        v
    }

    /// Tile of a window and the workspace it is in.
    pub fn tile_of(&self, id: WindowId) -> Option<(&Workspace, TileId, &Tile)> {
        let w = self.windows.get(&id)?;
        let ws = self.workspaces.get(&w.workspace)?;
        let t = w.tile?;
        Some((ws, t, ws.tiles.get(&t)?))
    }

    /// Unlink a window from its tile or floating list. Empty tiles are removed
    /// from the layout. The window stays in `windows`.
    pub fn detach(&mut self, id: WindowId) {
        let Some(w) = self.windows.get_mut(&id) else { return };
        let tile = w.tile.take();
        let Some(ws) = self.workspaces.get_mut(&w.workspace) else { return };
        ws.floating.retain(|f| *f != id);
        if ws.last_focused == Some(id) {
            ws.last_focused = None;
        }
        if let Some(t) = tile {
            if let Some(tile) = ws.tiles.get_mut(&t) {
                tile.windows.retain(|x| *x != id);
                if tile.active >= tile.windows.len() {
                    tile.active = tile.windows.len().saturating_sub(1);
                }
                if tile.windows.is_empty() {
                    ws.tiles.remove(&t);
                    ws.layout.remove(t);
                }
            }
        }
    }

    /// Link a window into `workspace`, tiled next to the focused tile or floating.
    pub fn attach(&mut self, id: WindowId, workspace: &str, floating: bool) {
        let tile_id = self.new_tile_id();
        let area = self
            .workspaces
            .get(workspace)
            .and_then(|ws| self.display(ws.display))
            .map(|d| d.visible)
            .unwrap_or_default();
        let focus_tile =
            self.focused.and_then(|f| self.tile_of(f)).filter(|(ws, ..)| ws.name == workspace).map(|(_, t, _)| t);
        let Some(w) = self.windows.get_mut(&id) else { return };
        w.workspace = workspace.to_string();
        let Some(ws) = self.workspaces.get_mut(workspace) else { return };
        let floating = floating || w.info.is_dialog;
        if floating {
            w.floating = true;
            w.tile = None;
            ws.floating.push(id);
        } else {
            w.floating = false;
            w.tile = Some(tile_id);
            ws.tiles.insert(tile_id, Tile { windows: vec![id], active: 0, grouped: false });
            ws.layout.insert(tile_id, focus_tile, area);
        }
    }

    /// Where every managed window should be right now.
    pub fn plan(&self, opts: &LayoutOptions) -> Vec<(WindowId, Placement)> {
        let mut out: BTreeMap<WindowId, Placement> = BTreeMap::new();
        let hide_on = |out: &mut BTreeMap<WindowId, Placement>, id: WindowId, d: DisplayId| {
            out.entry(id).or_insert(Placement::Hide(d));
        };

        for d in &self.displays {
            let Some(ws) = self.active.get(&d.id).and_then(|n| self.workspaces.get(n)) else { continue };
            let area = d.visible.inset_by(opts.gaps_outer);

            if let Some((id, mode)) = ws.windows().find_map(|id| self.windows[&id].fullscreen.map(|m| (id, m))) {
                let frame = match mode {
                    FullscreenMode::Full => d.visible,
                    FullscreenMode::Maximized => area,
                };
                out.insert(id, Placement::Show(frame));
            }

            if ws.layout.manages_geometry() {
                for (tile_id, rect) in ws.layout.arrange(area, opts) {
                    let Some(tile) = ws.tiles.get(&tile_id) else { continue };
                    let active = tile.active_window();
                    for w in &tile.windows {
                        if Some(*w) == active {
                            out.entry(*w).or_insert(Placement::Show(rect));
                        } else {
                            hide_on(&mut out, *w, d.id);
                        }
                    }
                }
            }
            for tile in ws.tiles.values().filter(|_| !ws.layout.manages_geometry()) {
                // Tiles of a floating layout stay where the user put them;
                // only the active member of a group is shown.
                let active = tile.active_window();
                for w in &tile.windows {
                    if Some(*w) == active {
                        out.entry(*w).or_insert(Placement::Show(self.windows[w].float_frame));
                    } else {
                        hide_on(&mut out, *w, d.id);
                    }
                }
            }
            for w in &ws.floating {
                out.entry(*w).or_insert(Placement::Show(self.windows[w].float_frame));
            }
        }

        if let Some(d) = self.scratchpad_on.and_then(|d| self.display(d)) {
            if let Some(ws) = self.workspaces.get(SCRATCHPAD) {
                for w in ws.windows() {
                    let mw = &self.windows[&w];
                    let size = Size::new(mw.float_frame.w.max(400.0), mw.float_frame.h.max(300.0));
                    out.insert(w, Placement::Show(d.visible.centered(size)));
                }
            }
        }

        for (id, w) in &self.windows {
            if out.contains_key(id) {
                continue;
            }
            let ws_display = self.workspaces.get(&w.workspace).map(|ws| ws.display);
            if w.pinned && w.workspace != SCRATCHPAD {
                out.insert(*id, Placement::Show(w.float_frame));
                continue;
            }
            let d = ws_display.or_else(|| self.primary_display());
            if let Some(d) = d {
                out.insert(*id, Placement::Hide(d));
            }
        }
        out.into_iter().collect()
    }
}
