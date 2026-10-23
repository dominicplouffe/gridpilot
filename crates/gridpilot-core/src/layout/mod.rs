//! Tiling layouts.
//!
//! A layout arranges *tiles* inside a workspace's usable area. A tile is a
//! slot that holds one window, or several when they are grouped into tabs, so
//! layouts never need to know about grouping. Layouts are registered by name
//! in a [`LayoutRegistry`]; adding a new one (spiral, grid, centered-master,
//! ...) means implementing [`Layout`] and registering a constructor.

mod dwindle;
mod floating;
mod master;
mod scrolling;

pub use dwindle::Dwindle;
pub use floating::Floating;
pub use master::Master;
pub use scrolling::Scrolling;

use crate::geometry::{Axis, Rect};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TileId(pub u64);

/// Tunables shared by layouts, from the `[tiling]` config section.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LayoutOptions {
    /// Space between tiles (Hyprland `gaps_in`, doubled: Hyprland applies it per side).
    pub gaps_inner: f64,
    /// Space between tiles and the screen edge (`gaps_out`).
    pub gaps_outer: f64,
    /// Fraction of the area the master column takes.
    pub master_ratio: f64,
    /// Column width of the scrolling layout as a fraction of the area.
    pub column_width: f64,
}

impl Default for LayoutOptions {
    fn default() -> Self {
        Self { gaps_inner: 10.0, gaps_outer: 10.0, master_ratio: 0.55, column_width: 0.49 }
    }
}

pub trait Layout: std::fmt::Debug {
    fn name(&self) -> &'static str;

    /// `false` for layouts that leave geometry to the user (Divvy mode).
    fn manages_geometry(&self) -> bool {
        true
    }

    /// Add `tile`, next to `focused` if the layout cares about position.
    fn insert(&mut self, tile: TileId, focused: Option<TileId>, area: Rect);

    fn remove(&mut self, tile: TileId);

    /// Tiles in a stable reading order. Used to rebuild a workspace in a new
    /// layout when the user cycles layouts, and for `focus next`.
    fn tiles(&self) -> Vec<TileId>;

    fn arrange(&self, area: Rect, opts: &LayoutOptions) -> Vec<(TileId, Rect)>;

    fn swap(&mut self, a: TileId, b: TileId);

    /// Hyprland `togglesplit`: flip the split the tile sits in.
    fn toggle_split(&mut self, tile: TileId) {
        let _ = tile;
    }

    /// Grow (positive `delta`) or shrink the tile along `axis` by about
    /// `delta` pixels, like Hyprland's `resizeactive`.
    fn resize(&mut self, tile: TileId, axis: Axis, delta: f64, area: Rect, opts: &LayoutOptions) {
        let _ = (tile, axis, delta, area, opts);
    }

    /// Lets the scrolling layout bring the focused column into view.
    fn on_focus(&mut self, tile: TileId, area: Rect, opts: &LayoutOptions) {
        let _ = (tile, area, opts);
    }

    fn contains(&self, tile: TileId) -> bool {
        self.tiles().contains(&tile)
    }
}

pub type LayoutCtor = fn() -> Box<dyn Layout>;

/// Name -> constructor. Features and plugins may register more.
#[derive(Clone)]
pub struct LayoutRegistry {
    ctors: BTreeMap<String, LayoutCtor>,
}

impl LayoutRegistry {
    pub fn empty() -> Self {
        Self { ctors: BTreeMap::new() }
    }

    pub fn register(&mut self, name: &str, ctor: LayoutCtor) {
        self.ctors.insert(name.to_string(), ctor);
    }

    pub fn create(&self, name: &str) -> Option<Box<dyn Layout>> {
        self.ctors.get(name).map(|c| c())
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.ctors.keys().map(|s| s.as_str())
    }
}

impl Default for LayoutRegistry {
    fn default() -> Self {
        let mut r = Self::empty();
        r.register("dwindle", || Box::new(Dwindle::default()));
        r.register("master", || Box::new(Master::default()));
        r.register("scrolling", || Box::new(Scrolling::default()));
        r.register("floating", || Box::new(Floating::default()));
        r
    }
}

impl std::fmt::Debug for LayoutRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.ctors.keys()).finish()
    }
}

/// Move `tiles` from one layout into a freshly built one, keeping order.
pub fn rebuild(from: &dyn Layout, mut into: Box<dyn Layout>, area: Rect) -> Box<dyn Layout> {
    let mut prev = None;
    for t in from.tiles() {
        into.insert(t, prev, area);
        prev = Some(t);
    }
    into
}

#[cfg(test)]
pub(crate) fn ids(n: u64) -> Vec<TileId> {
    (1..=n).map(TileId).collect()
}
