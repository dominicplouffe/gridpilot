//! Master/stack: one large tile on the left, the rest stacked on the right.
//! New tiles take the master slot, as in Omarchy (`new_status = master`).

use super::{Layout, LayoutOptions, TileId};
use crate::geometry::{Axis, Rect};

#[derive(Debug)]
pub struct Master {
    order: Vec<TileId>,
    ratio: Option<f64>,
    /// When false, new tiles join the bottom of the stack instead.
    pub new_is_master: bool,
}

impl Default for Master {
    fn default() -> Self {
        Self { order: Vec::new(), ratio: None, new_is_master: true }
    }
}

impl Layout for Master {
    fn name(&self) -> &'static str {
        "master"
    }

    fn insert(&mut self, tile: TileId, _focused: Option<TileId>, _area: Rect) {
        if self.order.contains(&tile) {
            return;
        }
        if self.new_is_master {
            self.order.insert(0, tile);
        } else {
            self.order.push(tile);
        }
    }

    fn remove(&mut self, tile: TileId) {
        self.order.retain(|t| *t != tile);
    }

    fn tiles(&self) -> Vec<TileId> {
        self.order.clone()
    }

    fn arrange(&self, area: Rect, opts: &LayoutOptions) -> Vec<(TileId, Rect)> {
        let gap = opts.gaps_inner;
        match self.order.as_slice() {
            [] => vec![],
            [only] => vec![(*only, area)],
            [master, stack @ ..] => {
                let ratio = self.ratio.unwrap_or(opts.master_ratio);
                let (m, rest) = area.split(Axis::Horizontal, ratio, gap);
                let mut out = vec![(*master, m)];
                let n = stack.len() as f64;
                let h = (rest.h - gap * (n - 1.0)) / n;
                for (i, t) in stack.iter().enumerate() {
                    let y = rest.y + i as f64 * (h + gap);
                    out.push((*t, Rect::new(rest.x, y.round(), rest.w, h.round())));
                }
                out
            }
        }
    }

    fn swap(&mut self, a: TileId, b: TileId) {
        let ia = self.order.iter().position(|t| *t == a);
        let ib = self.order.iter().position(|t| *t == b);
        if let (Some(ia), Some(ib)) = (ia, ib) {
            self.order.swap(ia, ib);
        }
    }

    fn resize(&mut self, tile: TileId, axis: Axis, delta: f64, area: Rect, opts: &LayoutOptions) {
        if axis != Axis::Horizontal || self.order.len() < 2 || area.w <= 0.0 {
            return;
        }
        let d = delta / area.w;
        let is_master = self.order.first() == Some(&tile);
        let r = self.ratio.unwrap_or(opts.master_ratio) + if is_master { d } else { -d };
        self.ratio = Some(r.clamp(0.1, 0.9));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newest_is_master() {
        let mut m = Master::default();
        let area = Rect::new(0.0, 0.0, 1000.0, 1000.0);
        for i in 1..=3 {
            m.insert(TileId(i), None, area);
        }
        let r = m.arrange(area, &LayoutOptions { gaps_inner: 0.0, master_ratio: 0.6, ..Default::default() });
        assert_eq!(r[0], (TileId(3), Rect::new(0.0, 0.0, 600.0, 1000.0)));
        assert_eq!(r[2], (TileId(1), Rect::new(600.0, 500.0, 400.0, 500.0)));
    }
}
