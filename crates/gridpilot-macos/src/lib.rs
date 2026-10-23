//! # GridPilot for macOS
//!
//! * [`ax`] / [`platform`]: windows are read and moved through the
//!   Accessibility API (the same API Divvy, Rectangle and AeroSpace use), so
//!   GridPilot needs the Accessibility permission and nothing else; SIP stays on.
//! * [`hotkeys`]: global shortcuts through Carbon's `RegisterEventHotKey`,
//!   which needs no Input Monitoring permission.
//! * [`overlay`], [`hud`], [`statusbar`]: AppKit UI drawn from engine state.
//! * [`capslock`]: Caps Lock as the `mod` key, using macOS's own `hidutil`.
//! * [`app`]: the main-thread run loop that owns the engine.
//!
//! On other targets only [`hidmap`] is built, so the workspace still builds.

// Pure helpers, built and tested on every OS.
pub mod hidmap;

#[cfg(target_os = "macos")]
pub mod app;
#[cfg(target_os = "macos")]
pub mod ax;
#[cfg(target_os = "macos")]
pub mod capslock;
#[cfg(target_os = "macos")]
mod ffi;
#[cfg(target_os = "macos")]
pub mod hotkeys;
#[cfg(target_os = "macos")]
pub mod hud;
#[cfg(target_os = "macos")]
pub mod keycodes;
#[cfg(target_os = "macos")]
pub mod overlay;
#[cfg(target_os = "macos")]
pub mod platform;
#[cfg(target_os = "macos")]
pub mod statusbar;

#[cfg(target_os = "macos")]
pub use app::{run, MainThreadPoster};
#[cfg(target_os = "macos")]
pub use platform::MacPlatform;
