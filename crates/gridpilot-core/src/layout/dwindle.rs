//! Hyprland's default layout: a binary space partition where every new tile
//! splits the focused one along its longer side.

use super::{Layout, LayoutOptions, TileId};
use crate::geometry::{Axis, Rect};

#[derive(Debug, Clone, PartialEq)]
enum Node {
    Leaf(TileId),
    Split { axis: Axis, ratio: f64, a: Box<Node>, b: Box<Node> },
}

impl Node {
    fn leaves(&self, out: &mut Vec<TileId>) {
        match self {
            Node::Leaf(t) => out.push(*t),
            Node::Split { a, b, .. } => {
                a.leaves(out);
                b.leaves(out);
            }
        }
    }

    fn contains(&self, tile: TileId) -> bool {
        match self {
            Node::Leaf(t) => *t == tile,
            Node::Split { a, b, .. } => a.contains(tile) || b.contains(tile),
        }
    }

    fn arrange(&self, area: Rect, gap: f64, out: &mut Vec<(TileId, Rect)>) {
        match self {
            Node::Leaf(t) => out.push((*t, area)),
            Node::Split { axis, ratio, a, b } => {
                let (ra, rb) = area.split(*axis, *ratio, gap);
                a.arrange(ra, gap, out);
                b.arrange(rb, gap, out);
            }
        }
    }

    /// Replace the leaf `target` by a split holding it and `tile`.
    fn split_leaf(&mut self, target: TileId, tile: TileId, area: Rect, gap: f64) -> bool {
        match self {
            Node::Leaf(t) if *t == target => {
                // Split along the longer side, new tile right/below
                // (Hyprland `force_split = 2`).
                let axis = if area.w >= area.h { Axis::Horizontal } else { Axis::Vertical };
                *self =
                    Node::Split { axis, ratio: 0.5, a: Box::new(Node::Leaf(target)), b: Box::new(Node::Leaf(tile)) };
                true
            }
            Node::Leaf(_) => false,
            Node::Split { axis, ratio, a, b } => {
                let (ra, rb) = area.split(*axis, *ratio, gap);
                a.split_leaf(target, tile, ra, gap) || b.split_leaf(target, tile, rb, gap)
            }
        }
    }

    /// Remove `tile`; a split left with one child collapses into that child.
    fn remove(self, tile: TileId) -> Option<Node> {
        match self {
            Node::Leaf(t) if t == tile => None,
            leaf @ Node::Leaf(_) => Some(leaf),
            Node::Split { axis, ratio, a, b } => match (a.remove(tile), b.remove(tile)) {
                (Some(a), Some(b)) => Some(Node::Split { axis, ratio, a: Box::new(a), b: Box::new(b) }),
                (Some(only), None) | (None, Some(only)) => Some(only),
                (None, None) => None,
            },
        }
    }

    fn rename(&mut self, from: TileId, to: TileId) {
        match self {
            Node::Leaf(t) if *t == from => *t = to,
            Node::Leaf(_) => {}
            Node::Split { a, b, .. } => {
                a.rename(from, to);
                b.rename(from, to);
            }
        }
    }

    /// Flip the axis of the split directly holding `tile`.
    fn toggle_parent(&mut self, tile: TileId) -> bool {
        if let Node::Split { axis, a, b, .. } = self {
            let direct = matches!(**a, Node::Leaf(t) if t == tile) || matches!(**b, Node::Leaf(t) if t == tile);
            if direct {
                *axis = axis.flip();
                return true;
            }
            return a.toggle_parent(tile) || b.toggle_parent(tile);
        }
        false
    }

    /// Move the nearest enclosing divider on `want` so `tile` grows by `delta`.
    fn resize(&mut self, tile: TileId, want: Axis, delta: f64, area: Rect, gap: f64) -> bool {
        let Node::Split { axis, ratio, a, b } = self else {
            return false;
        };
        let (ra, rb) = area.split(*axis, *ratio, gap);
        let in_a = a.contains(tile);
        if !in_a && !b.contains(tile) {
            return false;
        }
        // Prefer the innermost matching split.
        let handled = if in_a { a.resize(tile, want, delta, ra, gap) } else { b.resize(tile, want, delta, rb, gap) };
        if handled {
            return true;
        }
        if *axis != want {
            return false;
        }
        let span = match want {
            Axis::Horizontal => area.w,
            Axis::Vertical => area.h,
        };
        if span <= 0.0 {
            return false;
        }
        let d = delta / span;
        *ratio = (*ratio + if in_a { d } else { -d }).clamp(0.1, 0.9);
        true
    }
}

#[derive(Debug, Default)]
pub struct Dwindle {
    root: Option<Node>,
}

impl Layout for Dwindle {
    fn name(&self) -> &'static str {
        "dwindle"
    }

    fn insert(&mut self, tile: TileId, focused: Option<TileId>, area: Rect) {
        if self.contains(tile) {
            return;
        }
        let Some(root) = self.root.as_mut() else {
            self.root = Some(Node::Leaf(tile));
            return;
        };
        let target = focused.filter(|f| root.contains(*f)).unwrap_or_else(|| {
            let mut v = Vec::new();
            root.leaves(&mut v);
            *v.last().expect("non-empty tree has a leaf")
        });
        root.split_leaf(target, tile, area, LayoutOptions::default().gaps_inner);
    }

    fn remove(&mut self, tile: TileId) {
        self.root = self.root.take().and_then(|r| r.remove(tile));
    }

    fn tiles(&self) -> Vec<TileId> {
        let mut v = Vec::new();
        if let Some(r) = &self.root {
            r.leaves(&mut v);
        }
        v
    }

    fn arrange(&self, area: Rect, opts: &LayoutOptions) -> Vec<(TileId, Rect)> {
        let mut out = Vec::new();
        if let Some(r) = &self.root {
            r.arrange(area, opts.gaps_inner, &mut out);
        }
        out
    }

    fn swap(&mut self, a: TileId, b: TileId) {
        if let Some(r) = self.root.as_mut() {
            let tmp = TileId(u64::MAX);
            r.rename(a, tmp);
            r.rename(b, a);
            r.rename(tmp, b);
        }
    }

    fn toggle_split(&mut self, tile: TileId) {
        if let Some(r) = self.root.as_mut() {
            r.toggle_parent(tile);
        }
    }

    fn resize(&mut self, tile: TileId, axis: Axis, delta: f64, area: Rect, opts: &LayoutOptions) {
        if let Some(r) = self.root.as_mut() {
            r.resize(tile, axis, delta, area, opts.gaps_inner);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::ids;

    fn area() -> Rect {
        Rect::new(0.0, 0.0, 1610.0, 1000.0)
    }

    fn opts() -> LayoutOptions {
        LayoutOptions { gaps_inner: 10.0, ..Default::default() }
    }

    fn build(n: u64) -> Dwindle {
        let mut d = Dwindle::default();
        let mut last = None;
        for t in ids(n) {
            d.insert(t, last, area());
            last = Some(t);
        }
        d
    }

    #[test]
    fn spirals_inward() {
        let d = build(3);
        let r = d.arrange(area(), &opts());
        assert_eq!(r[0], (TileId(1), Rect::new(0.0, 0.0, 800.0, 1000.0)));
        assert_eq!(r[1], (TileId(2), Rect::new(810.0, 0.0, 800.0, 495.0)));
        assert_eq!(r[2], (TileId(3), Rect::new(810.0, 505.0, 800.0, 495.0)));
    }

    #[test]
    fn remove_collapses_split() {
        let mut d = build(3);
        d.remove(TileId(2));
        let r = d.arrange(area(), &opts());
        assert_eq!(r.len(), 2);
        assert_eq!(r[1], (TileId(3), Rect::new(810.0, 0.0, 800.0, 1000.0)));
        d.remove(TileId(1));
        d.remove(TileId(3));
        assert!(d.tiles().is_empty());
    }

    #[test]
    fn swap_toggle_and_resize() {
        let mut d = build(2);
        d.swap(TileId(1), TileId(2));
        assert_eq!(d.tiles(), vec![TileId(2), TileId(1)]);
        d.toggle_split(TileId(1));
        let r = d.arrange(area(), &opts());
        assert_eq!(r[0].1.w, 1610.0);
        d.resize(TileId(2), Axis::Vertical, 99.5, area(), &opts());
        let r = d.arrange(area(), &opts());
        assert!(r[0].1.h > 550.0, "first tile grew: {:?}", r[0].1);
    }
}
