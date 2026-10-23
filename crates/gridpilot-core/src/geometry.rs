//! Screen geometry.
//!
//! GridPilot uses one global coordinate space: origin at the top-left corner
//! of the primary display, `y` growing downwards. This matches the macOS
//! Accessibility API and Quartz display bounds, Win32 and X11/Wayland
//! compositors, so backends rarely have to flip anything.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Size {
    pub w: f64,
    pub h: f64,
}

impl Size {
    pub const fn new(w: f64, h: f64) -> Self {
        Self { w, h }
    }
}

/// Spacing on each side of a rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Insets {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl Insets {
    pub const fn uniform(v: f64) -> Self {
        Self { top: v, right: v, bottom: v, left: v }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    pub const fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self { x, y, w, h }
    }

    pub fn min_x(&self) -> f64 {
        self.x
    }
    pub fn min_y(&self) -> f64 {
        self.y
    }
    pub fn max_x(&self) -> f64 {
        self.x + self.w
    }
    pub fn max_y(&self) -> f64 {
        self.y + self.h
    }

    pub fn origin(&self) -> Point {
        Point::new(self.x, self.y)
    }

    pub fn size(&self) -> Size {
        Size::new(self.w, self.h)
    }

    pub fn center(&self) -> Point {
        Point::new(self.x + self.w / 2.0, self.y + self.h / 2.0)
    }

    pub fn area(&self) -> f64 {
        self.w.max(0.0) * self.h.max(0.0)
    }

    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.x && p.x < self.max_x() && p.y >= self.y && p.y < self.max_y()
    }

    pub fn inset(&self, i: Insets) -> Rect {
        Rect::new(
            self.x + i.left,
            self.y + i.top,
            (self.w - i.left - i.right).max(0.0),
            (self.h - i.top - i.bottom).max(0.0),
        )
    }

    pub fn inset_by(&self, v: f64) -> Rect {
        self.inset(Insets::uniform(v))
    }

    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let mx = self.max_x().min(other.max_x());
        let my = self.max_y().min(other.max_y());
        (mx > x && my > y).then(|| Rect::new(x, y, mx - x, my - y))
    }

    /// A rectangle of `size` centered inside `self`, clamped to fit.
    pub fn centered(&self, size: Size) -> Rect {
        let w = size.w.min(self.w);
        let h = size.h.min(self.h);
        Rect::new((self.x + (self.w - w) / 2.0).round(), (self.y + (self.h - h) / 2.0).round(), w, h)
    }

    /// Split along `axis` at `ratio` (0..1), leaving `gap` between the halves.
    pub fn split(&self, axis: Axis, ratio: f64, gap: f64) -> (Rect, Rect) {
        let ratio = ratio.clamp(0.05, 0.95);
        match axis {
            Axis::Horizontal => {
                let usable = (self.w - gap).max(0.0);
                let a = (usable * ratio).round();
                (Rect::new(self.x, self.y, a, self.h), Rect::new(self.x + a + gap, self.y, usable - a, self.h))
            }
            Axis::Vertical => {
                let usable = (self.h - gap).max(0.0);
                let a = (usable * ratio).round();
                (Rect::new(self.x, self.y, self.w, a), Rect::new(self.x, self.y + a + gap, self.w, usable - a))
            }
        }
    }

    /// Map `self`, expressed relative to `from`, into the same relative
    /// position inside `to`. Used to carry floating windows across displays.
    pub fn reproject(&self, from: &Rect, to: &Rect) -> Rect {
        let sx = if from.w > 0.0 { to.w / from.w } else { 1.0 };
        let sy = if from.h > 0.0 { to.h / from.h } else { 1.0 };
        Rect::new(to.x + (self.x - from.x) * sx, to.y + (self.y - from.y) * sy, self.w * sx, self.h * sy)
    }

    pub fn approx_eq(&self, other: &Rect, tolerance: f64) -> bool {
        (self.x - other.x).abs() <= tolerance
            && (self.y - other.y).abs() <= tolerance
            && (self.w - other.w).abs() <= tolerance
            && (self.h - other.h).abs() <= tolerance
    }
}

impl fmt::Display for Rect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}+{}+{}", self.w, self.h, self.x, self.y)
    }
}

/// The axis a container is split along. `Horizontal` places children side by
/// side (a vertical divider line), `Vertical` stacks them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Axis {
    Horizontal,
    Vertical,
}

impl Axis {
    pub fn flip(self) -> Self {
        match self {
            Axis::Horizontal => Axis::Vertical,
            Axis::Vertical => Axis::Horizontal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    pub const ALL: [Direction; 4] = [Direction::Left, Direction::Right, Direction::Up, Direction::Down];

    pub fn axis(self) -> Axis {
        match self {
            Direction::Left | Direction::Right => Axis::Horizontal,
            Direction::Up | Direction::Down => Axis::Vertical,
        }
    }

    pub fn opposite(self) -> Self {
        match self {
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
        }
    }

    /// Unit step in grid/screen space.
    pub fn delta(self) -> (i32, i32) {
        match self {
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
        }
    }
}

impl FromStr for Direction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "left" | "l" | "west" => Ok(Direction::Left),
            "right" | "r" | "east" => Ok(Direction::Right),
            "up" | "u" | "north" => Ok(Direction::Up),
            "down" | "d" | "south" => Ok(Direction::Down),
            other => Err(format!("unknown direction `{other}` (expected left/right/up/down)")),
        }
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Direction::Left => "left",
            Direction::Right => "right",
            Direction::Up => "up",
            Direction::Down => "down",
        })
    }
}

/// Pick the rectangle that best continues a move from `from` in `dir`.
///
/// Candidates must lie strictly on that side of `from`'s center. Among them we
/// prefer ones that overlap `from` on the perpendicular axis, then the nearest
/// edge, then the nearest center. This is the rule used for directional focus,
/// swap and "move to display", so every layout gets consistent navigation for
/// free without implementing its own neighbour logic.
pub fn nearest_in_direction<'a, T>(
    from: &Rect,
    dir: Direction,
    candidates: impl IntoIterator<Item = (T, &'a Rect)>,
) -> Option<T> {
    let c = from.center();
    let mut best: Option<(T, (u8, f64, f64))> = None;
    for (item, r) in candidates {
        let rc = r.center();
        let ahead = match dir {
            Direction::Left => rc.x < c.x - 1.0,
            Direction::Right => rc.x > c.x + 1.0,
            Direction::Up => rc.y < c.y - 1.0,
            Direction::Down => rc.y > c.y + 1.0,
        };
        if !ahead {
            continue;
        }
        let overlaps = match dir.axis() {
            Axis::Horizontal => r.y < from.max_y() && r.max_y() > from.y,
            Axis::Vertical => r.x < from.max_x() && r.max_x() > from.x,
        };
        let edge_gap = match dir {
            Direction::Left => (from.x - r.max_x()).abs(),
            Direction::Right => (r.x - from.max_x()).abs(),
            Direction::Up => (from.y - r.max_y()).abs(),
            Direction::Down => (r.y - from.max_y()).abs(),
        };
        let center_dist = ((rc.x - c.x).powi(2) + (rc.y - c.y).powi(2)).sqrt();
        let score = (if overlaps { 0 } else { 1 }, edge_gap, center_dist);
        let better = match &best {
            None => true,
            Some((_, s)) => score.partial_cmp(s) == Some(std::cmp::Ordering::Less),
        };
        if better {
            best = Some((item, score));
        }
    }
    best.map(|(item, _)| item)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_leaves_gap() {
        let r = Rect::new(0.0, 0.0, 1010.0, 500.0);
        let (a, b) = r.split(Axis::Horizontal, 0.5, 10.0);
        assert_eq!(a, Rect::new(0.0, 0.0, 500.0, 500.0));
        assert_eq!(b, Rect::new(510.0, 0.0, 500.0, 500.0));
    }

    #[test]
    fn nearest_prefers_overlap() {
        let from = Rect::new(0.0, 0.0, 100.0, 100.0);
        let right_aligned = Rect::new(200.0, 0.0, 100.0, 100.0);
        let right_offset = Rect::new(110.0, 300.0, 100.0, 100.0);
        let pick =
            nearest_in_direction(&from, Direction::Right, [("aligned", &right_aligned), ("offset", &right_offset)]);
        assert_eq!(pick, Some("aligned"));
        assert_eq!(nearest_in_direction(&from, Direction::Left, [("x", &right_aligned)]), None);
    }

    #[test]
    fn reproject_keeps_relative_position() {
        let from = Rect::new(0.0, 0.0, 1000.0, 1000.0);
        let to = Rect::new(1000.0, 0.0, 2000.0, 1000.0);
        let r = Rect::new(500.0, 0.0, 500.0, 500.0).reproject(&from, &to);
        assert_eq!(r, Rect::new(2000.0, 0.0, 1000.0, 500.0));
    }
}
