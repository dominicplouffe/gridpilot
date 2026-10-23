//! # GridPilot for macOS
//!
//! * [`ax`] / [`platform`]: windows are read and moved through the
//!   Accessibility API (the same API Divvy, Rectangle and AeroSpace use), so
//!   GridPilot needs the Accessibility permission and nothing else; SIP stays on.
//! * [`hotkeys`]: global shortcuts through Carbon's `RegisterEventHotKey`,
//!   which needs no Input Monitoring permission.
//! * [`overlay`], [`hud`], [`statusbar`]: AppKit UI drawn from engine state.
//! * [`app`]: the main-thread run loop that owns the engine.
//!
//! On other targets this crate is empty so the workspace still builds.

#![cfg(target_os = "macos")]

pub mod app;
pub mod ax;
mod ffi;
pub mod hotkeys;
pub mod hud;
pub mod keycodes;
pub mod overlay;
pub mod platform;
pub mod statusbar;

pub use app::{run, MainThreadPoster};
pub use platform::MacPlatform;
