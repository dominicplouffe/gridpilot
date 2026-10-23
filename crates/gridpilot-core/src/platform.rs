//! The boundary between the engine and an operating system.
//!
//! A backend (macOS Accessibility, Win32, a Wayland compositor's IPC, or the
//! in-memory [`crate::testing::MockPlatform`]) implements [`Platform`]. The
//! engine never talks to the OS any other way, which keeps every feature
//! testable and portable.

use crate::geometry::{Point, Rect};
use crate::grid::{GridSession, Margins};
use crate::keys::Chord;
use crate::theme::Theme;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Stable identifier of a window for as long as it exists (CGWindowID on
/// macOS, HWND on Windows, the compositor address on Hyprland).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WindowId(pub u64);

impl fmt::Display for WindowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DisplayId(pub u32);

impl fmt::Display for DisplayId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowInfo {
    pub id: WindowId,
    /// Bundle id or executable name, used by rules (`com.apple.Safari`).
    pub app: String,
    pub title: String,
    pub frame: Rect,
    /// Dialogs, sheets, panels, fixed-size windows: never tiled.
    pub is_dialog: bool,
    pub is_minimized: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisplayInfo {
    pub id: DisplayId,
    pub name: String,
    /// Whole display in global coordinates.
    pub frame: Rect,
    /// Minus the menu bar, dock, task bar or panels.
    pub visible: Rect,
    pub scale: f64,
}

/// Things the OS tells us about. Backends may also just call
/// [`crate::Engine::refresh`] periodically; events make it snappier.
#[derive(Debug, Clone, PartialEq)]
pub enum PlatformEvent {
    WindowCreated(WindowInfo),
    WindowDestroyed(WindowId),
    WindowFocused(WindowId),
    /// The user moved or resized a window by hand.
    WindowMoved {
        id: WindowId,
        frame: Rect,
    },
    DisplaysChanged,
    Hotkey(Chord),
}

/// Requests for UI that the platform draws. The engine owns the state
/// (e.g. the [`GridSession`]); the backend only renders it and reports input
/// back as commands such as `grid apply ...`.
#[derive(Debug, Clone)]
pub enum UiRequest {
    ShowGrid(GridOverlay),
    HideGrid,
    /// The Omarchy-style key cheat sheet (`mod+k`).
    ShowKeys(Vec<KeyHelp>),
    Notify {
        title: String,
        body: String,
    },
    ThemeChanged(Theme),
}

#[derive(Debug, Clone)]
pub struct GridOverlay {
    pub target: WindowId,
    pub session: GridSession,
    /// Display the target window is on; the overlay opens on every display.
    pub display: DisplayId,
    pub margins: Margins,
    pub theme: Theme,
    /// Shown under the grid like Divvy's shortcut list: (label, chord, selection command).
    pub shortcuts: Vec<(String, String, String)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyHelp {
    pub chord: String,
    pub command: String,
    pub description: String,
    pub feature: String,
}

pub type PlatformResult<T> = Result<T, String>;

pub trait Platform {
    fn displays(&self) -> PlatformResult<Vec<DisplayInfo>>;

    /// Manageable windows across all displays, front-most first.
    fn windows(&self) -> PlatformResult<Vec<WindowInfo>>;

    fn focused_window(&self) -> PlatformResult<Option<WindowId>>;

    fn set_frame(&mut self, id: WindowId, frame: Rect) -> PlatformResult<()>;

    /// Make a window invisible without closing it, for windows on inactive
    /// workspaces, in the scratchpad or behind the active tab of a group.
    /// macOS has no public API for this, so the backend parks the window just
    /// off the corner of its display (the trick AeroSpace uses).
    fn hide(&mut self, id: WindowId, on: &DisplayInfo) -> PlatformResult<()>;

    fn focus(&mut self, id: WindowId) -> PlatformResult<()>;

    fn close(&mut self, id: WindowId) -> PlatformResult<()>;

    fn minimize(&mut self, id: WindowId) -> PlatformResult<()> {
        let _ = id;
        Err("minimize is not supported by this backend".into())
    }

    /// Move the mouse pointer (for `general.mouse_follows_focus`).
    fn warp_pointer(&mut self, to: Point) {
        let _ = to;
    }

    /// Replace the set of global hotkeys.
    fn register_hotkeys(&mut self, chords: &[Chord]) -> PlatformResult<()>;

    fn present(&mut self, request: UiRequest) {
        let _ = request;
    }

    fn spawn(&mut self, command: &str) -> PlatformResult<()> {
        std::process::Command::new("/bin/sh")
            .arg("-c")
            .arg(command)
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("failed to run `{command}`: {e}"))
    }
}
