//! A horizontally scrolling strip of columns (Hyprland `scrolling`, niri,
//! PaperWM). Columns keep their width; focusing one scrolls it into view.

use super::{Layout, LayoutOptions, TileId};
use crate::geometry::{Axis, Rect};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Scrolling {
    columns: Vec<TileId>,
    widths: HashMap<TileId, f64>,
    /// Horizontal scroll position in pixels from the start of the strip.
    offset: f64,
}

impl Scrolling {
    fn width(&self, t: TileId, area: Rect, opts: &LayoutOptions) -> f64 {
        let frac = self.widths.get(&t).copied().unwrap_or(opts.column_width);
        (area.w * frac).round()
    }

    fn column_x(&self, idx: usize, area: Rect, opts: &LayoutOptions) -> f64 {
        self.columns[..idx].iter().map(|t| self.width(*t, area, opts) + opts.gaps_inner).sum()
    }
}

impl Layout for Scrolling {
    fn name(&self) -> &'static str {
        "scrolling"
    }

    fn insert(&mut self, tile: TileId, focused: Option<TileId>, _area: Rect) {
        if self.columns.contains(&tile) {
            return;
        }
        let at = focused
            .and_then(|f| self.columns.iter().position(|t| *t == f))
            .map(|i| i + 1)
            .unwrap_or(self.columns.len());
        self.columns.insert(at, tile);
    }

    fn remove(&mut self, tile: TileId) {
        self.columns.retain(|t| *t != tile);
        self.widths.remove(&tile);
    }

    fn tiles(&self) -> Vec<TileId> {
        self.columns.clone()
    }

    fn arrange(&self, area: Rect, opts: &LayoutOptions) -> Vec<(TileId, Rect)> {
        let mut x = area.x - self.offset;
        self.columns
            .iter()
            .map(|t| {
                let w = self.width(*t, area, opts);
                let r = Rect::new(x, area.y, w, area.h);
                x += w + opts.gaps_inner;
                (*t, r)
            })
            .collect()
    }

    fn swap(&mut self, a: TileId, b: TileId) {
        let ia = self.columns.iter().position(|t| *t == a);
        let ib = self.columns.iter().position(|t| *t == b);
        if let (Some(ia), Some(ib)) = (ia, ib) {
            self.columns.swap(ia, ib);
        }
    }

    fn resize(&mut self, tile: TileId, axis: Axis, delta: f64, area: Rect, opts: &LayoutOptions) {
        if axis != Axis::Horizontal || area.w <= 0.0 {
            return;
        }
        let cur = self.widths.get(&tile).copied().unwrap_or(opts.column_width);
        self.widths.insert(tile, (cur + delta / area.w).clamp(0.1, 1.0));
        self.on_focus(tile, area, opts);
    }

    fn on_focus(&mut self, tile: TileId, area: Rect, opts: &LayoutOptions) {
        let Some(idx) = self.columns.iter().position(|t| *t == tile) else {
            return;
        };
        let start = self.column_x(idx, area, opts);
        let end = start + self.width(tile, area, opts);
        let off = self.offset;
        if start < off {
            self.offset = start;
        } else if end > off + area.w {
            self.offset = end - area.w;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_scrolls_column_into_view() {
        let area = Rect::new(0.0, 0.0, 1000.0, 800.0);
        let opts = LayoutOptions { gaps_inner: 0.0, column_width: 0.5, ..Default::default() };
        let mut s = Scrolling::default();
        let mut prev = None;
        for i in 1..=4 {
            s.insert(TileId(i), prev, area);
            prev = Some(TileId(i));
        }
        s.on_focus(TileId(4), area, &opts);
        let r = s.arrange(area, &opts);
        assert_eq!(r[3].1, Rect::new(500.0, 0.0, 500.0, 800.0));
        assert_eq!(r[0].1.x, -1000.0);
        s.on_focus(TileId(1), area, &opts);
        assert_eq!(s.arrange(area, &opts)[0].1.x, 0.0);
    }
}
