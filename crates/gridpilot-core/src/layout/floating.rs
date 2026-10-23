//! No tiling at all: windows stay where the user (or the grid) puts them.
//! This is GridPilot's "Divvy mode" workspace layout.

use super::{Layout, LayoutOptions, TileId};
use crate::geometry::Rect;

#[derive(Debug, Default)]
pub struct Floating {
    order: Vec<TileId>,
}

impl Layout for Floating {
    fn name(&self) -> &'static str {
        "floating"
    }

    fn manages_geometry(&self) -> bool {
        false
    }

    fn insert(&mut self, tile: TileId, _focused: Option<TileId>, _area: Rect) {
        if !self.order.contains(&tile) {
            self.order.push(tile);
        }
    }

    fn remove(&mut self, tile: TileId) {
        self.order.retain(|t| *t != tile);
    }

    fn tiles(&self) -> Vec<TileId> {
        self.order.clone()
    }

    fn arrange(&self, _area: Rect, _opts: &LayoutOptions) -> Vec<(TileId, Rect)> {
        Vec::new()
    }

    fn swap(&mut self, a: TileId, b: TileId) {
        let ia = self.order.iter().position(|t| *t == a);
        let ib = self.order.iter().position(|t| *t == b);
        if let (Some(ia), Some(ib)) = (ia, ib) {
            self.order.swap(ia, ib);
        }
    }
}
