//! # GridPilot core
//!
//! The platform-independent heart of GridPilot: Divvy's grid placement and
//! Omarchy's tiling, workspaces, scratchpad and groups, written once against
//! the small [`Platform`] trait.
//!
//! ```text
//!  hotkey / CLI / overlay ──► command line ──► Engine ──► World::plan ──► Platform
//!                                 ▲                │
//!                                 └── features ◄───┘ (commands, bindings, hooks, layouts)
//! ```
//!
//! Start with [`Engine::new`] and [`features::builtin`]. See `docs/ARCHITECTURE.md`.

pub mod command;
pub mod config;
pub mod engine;
pub mod feature;
pub mod features;
pub mod geometry;
pub mod grid;
pub mod keys;
pub mod layout;
pub mod platform;
pub mod testing;
pub mod theme;
pub mod world;

pub use command::{Args, CmdResult};
pub use config::Config;
pub use engine::Engine;
pub use feature::{EngineEvent, Feature, Registrar};
pub use geometry::{Direction, Rect};
pub use platform::{DisplayId, DisplayInfo, Platform, PlatformEvent, UiRequest, WindowId, WindowInfo};
