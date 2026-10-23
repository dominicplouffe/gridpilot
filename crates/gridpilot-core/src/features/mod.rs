//! Built-in features. Each file is one feature; see [`crate::feature`] for
//! how to write another.

pub mod focus;
pub mod grid;
pub mod groups;
pub mod scratchpad;
pub mod system;
pub mod tiling;
pub mod workspaces;

use crate::feature::Feature;

/// Everything GridPilot ships with, in install order. Later features win
/// binding conflicts, and the engine reports them as warnings.
pub fn builtin() -> Vec<Box<dyn Feature>> {
    vec![
        Box::new(system::SystemFeature),
        Box::new(grid::GridFeature),
        Box::new(focus::FocusFeature),
        Box::new(tiling::TilingFeature),
        Box::new(workspaces::WorkspacesFeature),
        Box::new(scratchpad::ScratchpadFeature),
        Box::new(groups::GroupsFeature),
    ]
}
