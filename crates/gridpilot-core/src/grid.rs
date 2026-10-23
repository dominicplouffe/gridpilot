//! The Divvy-style grid: cell math, selections and the interactive session
//! that the overlay UI drives.
//!
//! Everything here is pure so that the overlay on each platform only has to
//! draw [`GridSession`] state and forward mouse/keyboard input to it.

use crate::geometry::{Direction, Insets, Rect};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Largest grid the session lets you build by adding rows/columns. Divvy caps
/// this at 10x10, which subdivides to 20x20.
pub const MAX_BASE_CELLS: u16 = 10;
pub const MAX_CELLS: u16 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Grid {
    pub cols: u16,
    pub rows: u16,
}

impl Grid {
    pub fn new(cols: u16, rows: u16) -> Self {
        Self { cols: cols.clamp(1, MAX_CELLS), rows: rows.clamp(1, MAX_CELLS) }
    }

    pub fn subdivided(self) -> Self {
        Grid::new(self.cols * 2, self.rows * 2)
    }

    pub fn full(self) -> GridSelection {
        GridSelection { grid: self, col: 0, row: 0, cols: self.cols, rows: self.rows }
    }

    /// Frame of a single cell, see [`GridSelection::frame`].
    pub fn cell_frame(self, col: u16, row: u16, area: Rect, margins: &Margins) -> Rect {
        GridSelection { grid: self, col, row, cols: 1, rows: 1 }.frame(area, margins)
    }

    /// Which cell a point falls into (clamped to the grid).
    pub fn cell_at(self, area: Rect, x: f64, y: f64) -> Cell {
        let fx = ((x - area.x) / area.w).clamp(0.0, 0.999_999);
        let fy = ((y - area.y) / area.h).clamp(0.0, 0.999_999);
        Cell { col: (fx * self.cols as f64) as u16, row: (fy * self.rows as f64) as u16 }
    }
}

impl Default for Grid {
    fn default() -> Self {
        Grid::new(6, 4)
    }
}

impl FromStr for Grid {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (c, r) = s.split_once(['x', 'X']).ok_or_else(|| format!("grid `{s}` must look like 6x4"))?;
        let cols: u16 = c.trim().parse().map_err(|_| format!("bad column count in `{s}`"))?;
        let rows: u16 = r.trim().parse().map_err(|_| format!("bad row count in `{s}`"))?;
        if cols == 0 || rows == 0 || cols > MAX_CELLS || rows > MAX_CELLS {
            return Err(format!("grid `{s}` must be between 1x1 and {MAX_CELLS}x{MAX_CELLS}"));
        }
        Ok(Grid::new(cols, rows))
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.cols, self.rows)
    }
}

/// Divvy's two margin settings: space kept free along the screen edges, and
/// space left between windows that were placed on neighbouring cells.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Margins {
    pub screen: Insets,
    pub window: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Cell {
    pub col: u16,
    pub row: u16,
}

/// A rectangular block of cells, remembered together with the grid it was made
/// on, so a shortcut recorded on 6x4 still means "left half" on a 12x8 grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GridSelection {
    pub grid: Grid,
    pub col: u16,
    pub row: u16,
    pub cols: u16,
    pub rows: u16,
}

impl GridSelection {
    /// The selection spanned by two corner cells, in any order.
    pub fn spanning(grid: Grid, a: Cell, b: Cell) -> Self {
        let (c0, c1) = (a.col.min(b.col), a.col.max(b.col));
        let (r0, r1) = (a.row.min(b.row), a.row.max(b.row));
        GridSelection { grid, col: c0, row: r0, cols: c1 - c0 + 1, rows: r1 - r0 + 1 }.clamped()
    }

    pub fn clamped(mut self) -> Self {
        self.col = self.col.min(self.grid.cols - 1);
        self.row = self.row.min(self.grid.rows - 1);
        self.cols = self.cols.clamp(1, self.grid.cols - self.col);
        self.rows = self.rows.clamp(1, self.grid.rows - self.row);
        self
    }

    /// The same region expressed on another grid, snapping outwards.
    pub fn on_grid(self, grid: Grid) -> Self {
        if grid == self.grid {
            return self;
        }
        let map = |v: u16, from: u16, to: u16, round_up: bool| -> u16 {
            let exact = v as f64 * to as f64 / from as f64;
            (if round_up { exact.ceil() } else { exact.floor() }) as u16
        };
        let c0 = map(self.col, self.grid.cols, grid.cols, false);
        let r0 = map(self.row, self.grid.rows, grid.rows, false);
        let c1 = map(self.col + self.cols, self.grid.cols, grid.cols, true).max(c0 + 1);
        let r1 = map(self.row + self.rows, self.grid.rows, grid.rows, true).max(r0 + 1);
        GridSelection { grid, col: c0, row: r0, cols: c1 - c0, rows: r1 - r0 }.clamped()
    }

    /// The window frame for this selection inside a display's visible area.
    ///
    /// Cells are laid out so that two selections that touch are separated by
    /// exactly `margins.window`, and selections on the outer edge sit exactly
    /// `margins.screen` from the screen edge.
    pub fn frame(&self, visible: Rect, margins: &Margins) -> Rect {
        let area = visible.inset(margins.screen);
        let gap = margins.window;
        let cw = (area.w - gap * (self.grid.cols as f64 - 1.0)) / self.grid.cols as f64;
        let ch = (area.h - gap * (self.grid.rows as f64 - 1.0)) / self.grid.rows as f64;
        let x = area.x + self.col as f64 * (cw + gap);
        let y = area.y + self.row as f64 * (ch + gap);
        let w = self.cols as f64 * cw + (self.cols as f64 - 1.0) * gap;
        let h = self.rows as f64 * ch + (self.rows as f64 - 1.0) * gap;
        Rect::new(x.round(), y.round(), w.round(), h.round())
    }

    /// The smallest selection on `grid` that covers `frame`. Used to start the
    /// interactive session on the window's current position.
    pub fn covering(grid: Grid, frame: Rect, visible: Rect) -> Self {
        let a = grid.cell_at(visible, frame.x + 1.0, frame.y + 1.0);
        let b = grid.cell_at(visible, frame.max_x() - 1.0, frame.max_y() - 1.0);
        GridSelection::spanning(grid, a, b)
    }
}

/// `col,row WxH` with an optional `/ CxR` grid, e.g. `0,0 3x4 / 6x4`.
impl FromStr for GridSelection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (body, grid) = match s.split_once('/') {
            Some((b, g)) => (b.trim(), Some(g.trim().parse::<Grid>()?)),
            None => (s.trim(), None),
        };
        let mut parts = body.split_whitespace();
        let pos = parts.next().ok_or("empty selection")?;
        let span = parts.next().unwrap_or("1x1");
        let (c, r) = pos.split_once(',').ok_or_else(|| format!("position `{pos}` must look like col,row"))?;
        let col: u16 = c.trim().parse().map_err(|_| format!("bad column in `{pos}`"))?;
        let row: u16 = r.trim().parse().map_err(|_| format!("bad row in `{pos}`"))?;
        let (w, h) = span.split_once(['x', 'X']).ok_or_else(|| format!("span `{span}` must look like 3x4"))?;
        let cols: u16 = w.trim().parse().map_err(|_| format!("bad width in `{span}`"))?;
        let rows: u16 = h.trim().parse().map_err(|_| format!("bad height in `{span}`"))?;
        let grid = grid.unwrap_or(Grid::new((col + cols).max(1), (row + rows).max(1)));
        if col + cols > grid.cols || row + rows > grid.rows || cols == 0 || rows == 0 {
            return Err(format!("selection `{s}` does not fit in a {grid} grid"));
        }
        Ok(GridSelection { grid, col, row, cols, rows })
    }
}

impl fmt::Display for GridSelection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{} {}x{} / {}", self.col, self.row, self.cols, self.rows, self.grid)
    }
}

/// Live state of the grid overlay while the user is choosing a region.
///
/// * Mouse: press on a cell, drag, release to apply (Divvy).
/// * Keyboard: arrows move the cursor, shift+arrows grow the selection from
///   the anchor, return applies, escape cancels.
/// * Option toggles a subdivided (2x finer) grid; +/- change the grid size.
#[derive(Debug, Clone, PartialEq)]
pub struct GridSession {
    base: Grid,
    subdivided: bool,
    anchor: Cell,
    cursor: Cell,
    dragging: bool,
}

impl GridSession {
    pub fn new(grid: Grid) -> Self {
        Self { base: grid, subdivided: false, anchor: Cell::default(), cursor: Cell::default(), dragging: false }
    }

    /// Start with the selection that covers the window's current frame.
    pub fn around(grid: Grid, frame: Rect, visible: Rect) -> Self {
        let sel = GridSelection::covering(grid, frame, visible);
        let mut s = Self::new(grid);
        s.anchor = Cell { col: sel.col, row: sel.row };
        s.cursor = Cell { col: sel.col + sel.cols - 1, row: sel.row + sel.rows - 1 };
        s
    }

    pub fn grid(&self) -> Grid {
        if self.subdivided {
            self.base.subdivided()
        } else {
            self.base
        }
    }

    pub fn base_grid(&self) -> Grid {
        self.base
    }

    pub fn is_subdivided(&self) -> bool {
        self.subdivided
    }

    pub fn is_dragging(&self) -> bool {
        self.dragging
    }

    pub fn selection(&self) -> GridSelection {
        GridSelection::spanning(self.grid(), self.anchor, self.cursor)
    }

    pub fn mouse_down(&mut self, cell: Cell) {
        self.anchor = self.clamp(cell);
        self.cursor = self.anchor;
        self.dragging = true;
    }

    pub fn mouse_drag(&mut self, cell: Cell) {
        if self.dragging {
            self.cursor = self.clamp(cell);
        }
    }

    /// Finishing a drag yields the selection to apply.
    pub fn mouse_up(&mut self, cell: Cell) -> Option<GridSelection> {
        if !self.dragging {
            return None;
        }
        self.cursor = self.clamp(cell);
        self.dragging = false;
        Some(self.selection())
    }

    /// Arrow keys. Without `extend` the whole selection collapses to a single
    /// cell and moves; with `extend` the anchor stays and the far corner moves.
    pub fn key_move(&mut self, dir: Direction, extend: bool) {
        let (dx, dy) = dir.delta();
        let step = |v: u16, d: i32, max: u16| -> u16 { (v as i32 + d).clamp(0, max as i32 - 1) as u16 };
        let g = self.grid();
        self.cursor = Cell { col: step(self.cursor.col, dx, g.cols), row: step(self.cursor.row, dy, g.rows) };
        if !extend {
            self.anchor = self.cursor;
        }
    }

    pub fn select_all(&mut self) {
        let g = self.grid();
        self.anchor = Cell::default();
        self.cursor = Cell { col: g.cols - 1, row: g.rows - 1 };
    }

    pub fn toggle_subdivide(&mut self) {
        let before = self.grid();
        self.subdivided = !self.subdivided;
        let sel = self.selection_on(before).on_grid(self.grid());
        self.set_selection(sel);
    }

    pub fn resize_grid(&mut self, d_cols: i32, d_rows: i32) {
        let before = self.grid();
        let sel = self.selection_on(before);
        let clamp = |v: u16, d: i32| (v as i32 + d).clamp(1, MAX_BASE_CELLS as i32) as u16;
        self.base = Grid::new(clamp(self.base.cols, d_cols), clamp(self.base.rows, d_rows));
        self.set_selection(sel.on_grid(self.grid()));
    }

    fn selection_on(&self, grid: Grid) -> GridSelection {
        GridSelection::spanning(grid, self.anchor, self.cursor)
    }

    fn set_selection(&mut self, sel: GridSelection) {
        self.anchor = Cell { col: sel.col, row: sel.row };
        self.cursor = Cell { col: sel.col + sel.cols - 1, row: sel.row + sel.rows - 1 };
    }

    fn clamp(&self, c: Cell) -> Cell {
        let g = self.grid();
        Cell { col: c.col.min(g.cols - 1), row: c.row.min(g.rows - 1) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn screen() -> Rect {
        Rect::new(0.0, 25.0, 1200.0, 800.0)
    }

    #[test]
    fn halves_meet_with_window_margin() {
        let m = Margins { screen: Insets::uniform(10.0), window: 10.0 };
        let left: GridSelection = "0,0 3x4 / 6x4".parse().unwrap();
        let right: GridSelection = "3,0 3x4 / 6x4".parse().unwrap();
        let l = left.frame(screen(), &m);
        let r = right.frame(screen(), &m);
        assert_eq!(l.x, 10.0);
        assert_eq!(l.y, 35.0);
        assert_eq!(r.max_x(), 1190.0);
        assert_eq!(r.x - l.max_x(), 10.0);
        assert_eq!(l.h, 780.0);
    }

    #[test]
    fn selection_parses_and_displays() {
        let s: GridSelection = "1,2 3x1 / 6x4".parse().unwrap();
        assert_eq!(s, GridSelection { grid: Grid::new(6, 4), col: 1, row: 2, cols: 3, rows: 1 });
        assert_eq!(s.to_string(), "1,2 3x1 / 6x4");
        assert!("5,0 3x1 / 6x4".parse::<GridSelection>().is_err());
    }

    #[test]
    fn on_grid_maps_half_to_half() {
        let left: GridSelection = "0,0 3x4 / 6x4".parse().unwrap();
        let mapped = left.on_grid(Grid::new(2, 1));
        assert_eq!((mapped.col, mapped.cols, mapped.rows), (0, 1, 1));
        let fine = left.on_grid(Grid::new(12, 8));
        assert_eq!((fine.cols, fine.rows), (6, 8));
    }

    #[test]
    fn drag_selects_in_any_direction() {
        let mut s = GridSession::new(Grid::new(6, 4));
        s.mouse_down(Cell { col: 4, row: 3 });
        s.mouse_drag(Cell { col: 1, row: 1 });
        let sel = s.mouse_up(Cell { col: 1, row: 0 }).unwrap();
        assert_eq!((sel.col, sel.row, sel.cols, sel.rows), (1, 0, 4, 4));
    }

    #[test]
    fn keyboard_extend_and_subdivide() {
        let mut s = GridSession::new(Grid::new(6, 4));
        s.key_move(Direction::Right, true);
        s.key_move(Direction::Right, true);
        s.key_move(Direction::Down, true);
        assert_eq!(s.selection(), "0,0 3x2 / 6x4".parse().unwrap());
        s.toggle_subdivide();
        assert_eq!(s.grid(), Grid::new(12, 8));
        assert_eq!(s.selection(), "0,0 6x4 / 12x8".parse().unwrap());
        s.toggle_subdivide();
        assert_eq!(s.selection(), "0,0 3x2 / 6x4".parse().unwrap());
    }

    #[test]
    fn covering_round_trips_a_frame() {
        let m = Margins::default();
        let sel: GridSelection = "2,1 2x2 / 6x4".parse().unwrap();
        let f = sel.frame(screen(), &m);
        assert_eq!(GridSelection::covering(sel.grid, f, screen()), sel);
    }
}
